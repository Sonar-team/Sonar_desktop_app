//! Thread de capture : lit les paquets sur l'interface pcap, les copie dans
//! des buffers du pool et les pousse dans le canal borné vers le thread de
//! traitement (comptage des pertes applicatives quand le canal est plein).

use crossbeam::channel::{Sender, TrySendError};
use log::{debug, error, warn};
use pcap::{Active, Capture};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tauri::ipc::Channel;

use crate::{
    events::CaptureEvent,
    state::capture::capture_handle::{
        PipelineThreadGuard, TerminalState,
        messages::{
            CaptureMessage,
            stats::{AppDropCounters, SharedCaptureStats},
        },
        threads::packet_buffer::PacketBufferPool,
    },
};

const STATS_POLL_INTERVAL_MS: u64 = 250;
/// Cadence maximale des logs de pertes et de l'événement backpressure :
/// sous saturation, chaque paquet échoue — un log par paquet noierait tout.
const DROP_REPORT_INTERVAL: Duration = Duration::from_secs(1);

/// Suivi local des pertes pour le rate-limiting des logs.
#[derive(Default)]
struct DropReport {
    no_buffer: u64,
    channel_full: u64,
}

impl DropReport {
    fn total(&self) -> u64 {
        self.no_buffer + self.channel_full
    }
}

/// Copie un paquet capturé dans un buffer du pool et l'envoie au thread de
/// traitement. Retourne `true` si la boucle de capture doit continuer,
/// `false` si le consommateur (thread de traitement) est parti et que la
/// boucle doit s'arrêter.
fn forward_captured_packet(
    packet: &pcap::Packet,
    tx: &Sender<CaptureMessage>,
    buffer_pool: &PacketBufferPool,
    drop_counters: &AppDropCounters,
    pending_drops: &mut DropReport,
) -> bool {
    let Some(mut buffer) = buffer_pool.get(packet.header.caplen as usize) else {
        drop_counters.add_no_buffer();
        pending_drops.no_buffer += 1;
        return true;
    };
    // On copie les octets DANS UN SCOPE LIMITE
    buffer.write_from_parts(packet.header, packet.data);

    match tx.try_send(CaptureMessage::Packet(buffer)) {
        Ok(()) => {
            // Succès : le processing thread RENDRA le buffer au pool.
            true
        }
        Err(TrySendError::Full(message)) => {
            drop_counters.add_channel_full();
            pending_drops.channel_full += 1;
            // Échec d'envoi => on remet IMMÉDIATEMENT le buffer au pool
            let CaptureMessage::Packet(buffer) = message;
            buffer_pool.put(buffer);
            true
        }
        Err(TrySendError::Disconnected(message)) => {
            // Le consommateur est parti (arrêt en cours) : ce paquet est
            // arrivé après l'arrêt effectif, ce n'est pas une perte -- on
            // rend le buffer et on sort sans le compter en « canal plein ».
            let CaptureMessage::Packet(buffer) = message;
            buffer_pool.put(buffer);
            debug!("Canal fermé côté processing : fin du thread de capture");
            false
        }
    }
}

/// Journalise et signale au front (best-effort) les pertes applicatives
/// accumulées depuis le dernier rapport, au plus une fois par
/// [`DROP_REPORT_INTERVAL`]. Réinitialise le compteur si un rapport est émis.
#[allow(clippy::too_many_arguments)]
fn maybe_report_drops(
    pending_drops: &mut DropReport,
    last_drop_report: &mut Instant,
    on_event: &Channel<CaptureEvent<'static>>,
    session_id: u64,
    channel_capacity: i32,
    tx_len: usize,
) {
    if pending_drops.total() == 0 || last_drop_report.elapsed() < DROP_REPORT_INTERVAL {
        return;
    }
    warn!(
        "Capture : {} paquets perdus côté app depuis {:?} (pool épuisé : {}, canal plein : {})",
        pending_drops.total(),
        last_drop_report.elapsed(),
        pending_drops.no_buffer,
        pending_drops.channel_full
    );
    if let Err(e) = on_event.send(CaptureEvent::ChannelCapacityPayload {
        session_id,
        channel_size: channel_capacity as usize,
        current_size: tx_len,
        backpressure: true,
    }) {
        error!("Erreur send channel capacity payload: {}", e);
    }
    *pending_drops = DropReport::default();
    *last_drop_report = Instant::now();
}

/// Rafraîchit les stats pcap partagées si l'intervalle de sondage est
/// écoulé. Extrait de la boucle de capture pour éviter un if/if-let imbriqué
/// répété à chaque tour (complexité cognitive).
fn maybe_poll_capture_stats(
    cap: &mut Capture<Active>,
    shared_stats: &SharedCaptureStats,
    last_stats_poll: &mut Instant,
    stats_poll_interval: Duration,
) {
    if last_stats_poll.elapsed() < stats_poll_interval {
        return;
    }
    *last_stats_poll = Instant::now();
    if let Ok(stats) = cap.stats() {
        shared_stats.store(stats);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_capture_thread_with_pool(
    tx: Sender<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    mut cap: Capture<Active>,
    stop_flag: Arc<AtomicBool>,
    terminal: Arc<TerminalState>,
    completion: Sender<()>,
    channel_capacity: i32,
    buffer_pool: Arc<PacketBufferPool>,
    drop_counters: Arc<AppDropCounters>,
    shared_stats: Arc<SharedCaptureStats>,
    session_id: u64,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _thread_guard = PipelineThreadGuard::new(
            "capture pcap",
            completion,
            Arc::clone(&stop_flag),
            Arc::clone(&terminal),
        );
        debug!("Démarrage du thread de capture avec pool");
        let stats_poll_interval = Duration::from_millis(STATS_POLL_INTERVAL_MS);
        let mut last_stats_poll = Instant::now()
            .checked_sub(stats_poll_interval)
            .unwrap_or_else(Instant::now);
        let mut pending_drops = DropReport::default();
        let mut last_drop_report = Instant::now();

        while !stop_flag.load(Ordering::Acquire) {
            maybe_poll_capture_stats(
                &mut cap,
                &shared_stats,
                &mut last_stats_poll,
                stats_poll_interval,
            );

            match cap.next_packet() {
                Ok(packet) => {
                    let keep_going = forward_captured_packet(
                        &packet,
                        &tx,
                        &buffer_pool,
                        &drop_counters,
                        &mut pending_drops,
                    );
                    // Canal déconnecté : on sort immédiatement, sans passer
                    // par le rapport de pertes (même comportement que
                    // l'ancien `break` inline, qui sautait le reste du bras).
                    if !keep_going {
                        break;
                    }
                    maybe_report_drops(
                        &mut pending_drops,
                        &mut last_drop_report,
                        &on_event,
                        session_id,
                        channel_capacity,
                        tx.len(),
                    );
                }
                Err(pcap::Error::PcapError(e)) if e.contains("Packets are not available") => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(pcap::Error::TimeoutExpired) => continue,
                Err(e) => {
                    error!("Erreur capture : {:?}", e);
                    // Pipeline mort : on force l'arrêt et on mémorise la
                    // cause ; le coordinateur préviendra le frontend après
                    // jointure et normalisation de la phase.
                    stop_flag.store(true, Ordering::Release);
                    terminal.record_autonomous(format!("erreur pcap : {e}"));
                    break;
                }
            }
        }
        // Dernier relevé des stats pcap avant de sortir : sans lui, le
        // récapitulatif final émis après drainage lirait des compteurs
        // vieux d'un cycle de polling (jusqu'à 250 ms de trafic) (#158).
        if let Ok(stats) = cap.stats() {
            shared_stats.store(stats);
        }
        debug!("Thread de capture terminé.");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::bounded;

    /// Trame ARP request complète (42 octets), identique à celle utilisée
    /// dans les tests du thread de traitement.
    fn arp_frame() -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(&[0xff; 6]); // dst broadcast
        frame.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]); // src
        frame.extend_from_slice(&[0x08, 0x06]); // ethertype ARP
        frame.extend_from_slice(&[0x00, 0x01]); // hw type ethernet
        frame.extend_from_slice(&[0x08, 0x00]); // proto type IPv4
        frame.extend_from_slice(&[0x06, 0x04]); // hlen, plen
        frame.extend_from_slice(&[0x00, 0x01]); // opération request
        frame.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]); // sha
        frame.extend_from_slice(&[192, 168, 1, 10]); // spa
        frame.extend_from_slice(&[0x00; 6]); // tha
        frame.extend_from_slice(&[192, 168, 1, 1]); // tpa
        frame
    }

    fn test_header(data: &[u8]) -> pcap::PacketHeader {
        pcap::PacketHeader {
            ts: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            caplen: data.len() as u32,
            len: data.len() as u32,
        }
    }

    fn recording_channel() -> (
        Channel<CaptureEvent<'static>>,
        Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    ) {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let channel = Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                captured
                    .lock()
                    .unwrap()
                    .push(serde_json::from_str(&json).unwrap());
            }
            Ok(())
        });
        (channel, events)
    }

    #[test]
    fn forward_captured_packet_sends_on_success() {
        let pool = PacketBufferPool::new(4, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(4);
        let drop_counters = AppDropCounters::default();
        let mut pending_drops = DropReport::default();
        let data = arp_frame();
        let header = test_header(&data);
        let packet = pcap::Packet::new(&header, &data);

        let keep_going =
            forward_captured_packet(&packet, &tx, &pool, &drop_counters, &mut pending_drops);

        assert!(keep_going);
        assert_eq!(pending_drops.total(), 0);
        assert!(
            rx.try_recv().is_ok(),
            "le paquet est envoyé au thread de traitement"
        );
    }

    #[test]
    fn forward_captured_packet_counts_no_buffer_drop_when_pool_is_empty() {
        let pool = PacketBufferPool::new(0, 65_536);
        let (tx, _rx) = bounded::<CaptureMessage>(4);
        let drop_counters = AppDropCounters::default();
        let mut pending_drops = DropReport::default();
        let data = arp_frame();
        let header = test_header(&data);
        let packet = pcap::Packet::new(&header, &data);

        let keep_going =
            forward_captured_packet(&packet, &tx, &pool, &drop_counters, &mut pending_drops);

        assert!(keep_going, "pool épuisé : on continue, on compte la perte");
        assert_eq!(pending_drops.no_buffer, 1);
        assert_eq!(drop_counters.total(), 1);
    }

    #[test]
    fn forward_captured_packet_counts_channel_full_and_returns_buffer() {
        let pool = PacketBufferPool::new(4, 65_536);
        // Capacité nulle : try_send échoue systématiquement en Full.
        let (tx, _rx) = bounded::<CaptureMessage>(0);
        let drop_counters = AppDropCounters::default();
        let mut pending_drops = DropReport::default();
        let data = arp_frame();
        let header = test_header(&data);
        let packet = pcap::Packet::new(&header, &data);

        let keep_going =
            forward_captured_packet(&packet, &tx, &pool, &drop_counters, &mut pending_drops);

        assert!(keep_going);
        assert_eq!(pending_drops.channel_full, 1);
        // Pool à allocation paresseuse (aucun buffer dispo avant le premier
        // `get()`) : le buffer obtenu puis rendu sur l'échec d'envoi se
        // retrouve donc immédiatement disponible, pas perdu.
        assert_eq!(
            pool.len(),
            1,
            "le buffer est rendu au pool immédiatement sur échec d'envoi"
        );
    }

    #[test]
    fn forward_captured_packet_stops_the_loop_when_consumer_is_gone() {
        let pool = PacketBufferPool::new(4, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(4);
        drop(rx); // plus aucun récepteur : try_send échoue en Disconnected.
        let drop_counters = AppDropCounters::default();
        let mut pending_drops = DropReport::default();
        let data = arp_frame();
        let header = test_header(&data);
        let packet = pcap::Packet::new(&header, &data);

        let keep_going =
            forward_captured_packet(&packet, &tx, &pool, &drop_counters, &mut pending_drops);

        assert!(
            !keep_going,
            "consommateur parti -> la boucle de capture doit s'arrêter"
        );
        assert_eq!(
            pending_drops.total(),
            0,
            "un consommateur parti n'est pas compté comme une perte"
        );
    }

    #[test]
    fn maybe_report_drops_fires_and_resets_after_the_interval() {
        let mut pending_drops = DropReport {
            no_buffer: 2,
            channel_full: 1,
        };
        let mut last_drop_report = Instant::now() - DROP_REPORT_INTERVAL - Duration::from_millis(1);
        let (on_event, events) = recording_channel();

        maybe_report_drops(
            &mut pending_drops,
            &mut last_drop_report,
            &on_event,
            9,
            128,
            3,
        );

        assert_eq!(
            pending_drops.total(),
            0,
            "le compteur est remis à zéro après le rapport"
        );
        let events = events.lock().unwrap();
        assert!(
            events
                .iter()
                .any(|e| e["event"] == "channelCapacityPayload"),
            "l'occupation du canal est signalée au front"
        );
    }

    #[test]
    fn maybe_report_drops_does_nothing_without_pending_drops() {
        let mut pending_drops = DropReport::default();
        let mut last_drop_report = Instant::now() - DROP_REPORT_INTERVAL - Duration::from_millis(1);
        let (on_event, events) = recording_channel();

        maybe_report_drops(
            &mut pending_drops,
            &mut last_drop_report,
            &on_event,
            9,
            128,
            3,
        );

        assert!(
            events.lock().unwrap().is_empty(),
            "rien à signaler sans perte en attente"
        );
    }

    #[test]
    fn maybe_report_drops_respects_the_rate_limit() {
        let mut pending_drops = DropReport {
            no_buffer: 1,
            channel_full: 0,
        };
        let mut last_drop_report = Instant::now(); // rapport tout juste émis
        let (on_event, events) = recording_channel();

        maybe_report_drops(
            &mut pending_drops,
            &mut last_drop_report,
            &on_event,
            9,
            128,
            3,
        );

        assert_eq!(
            pending_drops.total(),
            1,
            "le compteur n'est pas remis à zéro avant l'intervalle minimal"
        );
        assert!(events.lock().unwrap().is_empty());
    }
}
