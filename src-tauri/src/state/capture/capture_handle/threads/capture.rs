//! Thread de capture : lit les paquets sur l'interface pcap, les copie dans
//! des buffers du pool et les pousse dans le canal borné vers le thread de
//! traitement (comptage des pertes applicatives quand le canal est plein).

use crossbeam::channel::Sender;
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

#[allow(clippy::too_many_arguments)]
pub fn spawn_capture_thread_with_pool(
    tx: Sender<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    mut cap: Capture<Active>,
    stop_flag: Arc<AtomicBool>,
    channel_capacity: i32,
    buffer_pool: Arc<PacketBufferPool>,
    drop_counters: Arc<AppDropCounters>,
    shared_stats: Arc<SharedCaptureStats>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        debug!("Démarrage du thread de capture avec pool");
        let stats_poll_interval = Duration::from_millis(STATS_POLL_INTERVAL_MS);
        let mut last_stats_poll = Instant::now()
            .checked_sub(stats_poll_interval)
            .unwrap_or_else(Instant::now);
        let mut pending_drops = DropReport::default();
        let mut last_drop_report = Instant::now();

        while !stop_flag.load(Ordering::Relaxed) {
            if last_stats_poll.elapsed() >= stats_poll_interval {
                last_stats_poll = Instant::now();
                if let Ok(stats) = cap.stats() {
                    shared_stats.store(stats);
                }
            }

            match cap.next_packet() {
                Ok(packet) => {
                    if let Some(mut buffer) = buffer_pool.get(packet.header.caplen as usize) {
                        // On copie les octets DANS UN SCOPE LIMITE
                        buffer.write_from_parts(packet.header, packet.data);

                        match tx.try_send(CaptureMessage::Packet(buffer)) {
                            Ok(()) => {
                                // Succès : le processing thread RENDRA le buffer au pool.
                            }
                            Err(err) => {
                                drop_counters.add_channel_full();
                                pending_drops.channel_full += 1;
                                // Échec d'envoi => on remet IMMÉDIATEMENT le buffer au pool
                                let CaptureMessage::Packet(buffer) = err.into_inner();
                                buffer_pool.put(buffer);
                            }
                        }
                    } else {
                        drop_counters.add_no_buffer();
                        pending_drops.no_buffer += 1;
                    }

                    if pending_drops.total() > 0
                        && last_drop_report.elapsed() >= DROP_REPORT_INTERVAL
                    {
                        warn!(
                            "Capture : {} paquets perdus côté app depuis {:?} (pool épuisé : {}, canal plein : {})",
                            pending_drops.total(),
                            last_drop_report.elapsed(),
                            pending_drops.no_buffer,
                            pending_drops.channel_full
                        );
                        if let Err(e) = on_event.send(CaptureEvent::ChannelCapacityPayload {
                            channel_size: channel_capacity as usize,
                            current_size: tx.len(),
                            backpressure: true,
                        }) {
                            error!("Erreur send channel capacity payload: {}", e);
                        }
                        pending_drops = DropReport::default();
                        last_drop_report = Instant::now();
                    }
                }
                Err(pcap::Error::PcapError(e)) if e.contains("Packets are not available") => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(pcap::Error::TimeoutExpired) => continue,
                Err(e) => {
                    error!("Erreur capture : {:?}", e);
                    // Pipeline mort : on force l'arrêt et on prévient le
                    // frontend, sinon l'UI croit capturer indéfiniment.
                    stop_flag.store(true, Ordering::Relaxed);
                    if let Err(send_err) = on_event.send(CaptureEvent::Stopped {
                        reason: format!("erreur pcap : {e}"),
                    }) {
                        error!("Erreur send Stopped: {}", send_err);
                    }
                    break;
                }
            }
        }
        debug!("Thread de capture terminé.");
    })
}
