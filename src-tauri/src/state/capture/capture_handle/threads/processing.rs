//! Thread de traitement de la capture live : consomme les paquets du canal,
//! met à jour la matrice de flux et le graphe, et pousse au front les batches
//! de paquets, les updates graphe et les stats périodiques.

use crossbeam::channel::{Receiver, RecvTimeoutError};
use log::{debug, error, info};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, ipc::Channel};

use crate::{
    events::CaptureEvent,
    state::{
        capture::capture_handle::{
            messages::{
                CaptureMessage,
                capture::{CapturedPacket, CapturedPacketOwned},
                channel::ChannelCapacityPayload,
                stats::{AppDropCounters, SharedCaptureStats, StatsPayload, StatsSnapshot},
            },
            threads::packet_buffer::{PacketBuffer, PacketBufferPool},
        },
        flow_matrix::FlowMatrix,
        graph::{GraphData, GraphUpdateBatch},
    },
};
use packet_parser::LinkType;
#[cfg(feature = "capture_timing")]
use packet_parser::timing::ParseTiming;
use sonar_flows_core::link::LinkView;

#[cfg(feature = "capture_timing")]
use super::capture_timing::{
    CapturePipelineTiming, CaptureTimingLogger, elapsed_ns_since, parse_packet_flow_with_timing,
};

pub(super) const PACKET_BATCH_MAX: usize = 256;
pub(super) const PACKET_BATCH_INTERVAL_MS: u64 = 75;
const PACKET_BATCH_INTERVAL: Duration = Duration::from_millis(PACKET_BATCH_INTERVAL_MS);
/// Cadence d'émission des stats vers le frontend (dédupliquées par maybe_send).
const STATS_EMIT_INTERVAL_MS: u64 = 250;
/// Intervalle minimal entre deux émissions d'occupation du canal (max 4/s).
/// L'ancien tempo « à chaque changement de taille » émettait quasiment un
/// événement IPC par paquet sous saturation — précisément quand le pipeline
/// était déjà sous pression (#141).
const CHANNEL_EMIT_MIN_INTERVAL: Duration = Duration::from_millis(250);
/// Flush anticipé du batch graphe (rafales de nouveaux flux, ex. début de capture).
const GRAPH_BATCH_MAX: usize = 512;
/// Attente maximale entre deux paquets pendant le drainage d'arrêt : couvre
/// largement le timeout pcap par défaut (25 ms) le temps que le thread de
/// capture sorte et lâche l'émetteur.
const DRAIN_RECV_TIMEOUT: Duration = Duration::from_millis(250);
/// Nombre maximal de paquets d'un batch réellement sérialisés vers la
/// WebView : le front n'en affiche que les 5 derniers (journal de
/// `BottomLong.vue`), sérialiser les 256 du batch était du travail par
/// paquet inutile (#154). La matrice et le graphe, eux, reçoivent tout.
const PACKET_BATCH_UI_MAX: usize = 16;
/// Plafond de flux de la matrice en capture live (#147). Un trafic à forte
/// cardinalité (scan, adresses aléatoires) créerait des lignes sans fin ;
/// plutôt qu'évincer des données (le relevé mentirait), la capture s'arrête
/// proprement avec une raison explicite — le relevé reste fidèle à ce qui a
/// été observé jusque-là. ~250 k lignes ≈ quelques centaines de Mo.
const MAX_LIVE_FLOWS: u32 = 250_000;
/// Cadence maximale des logs d'erreur de parsing : sous trafic malformé
/// (volontaire ou non), un log par paquet saturerait le fichier de logs.
const PARSE_ERROR_LOG_INTERVAL: Duration = Duration::from_secs(1);

/// IPs source/destination d'un flux possédé, en chaînes (vides si absentes).
fn flow_ips(owned: &CapturedPacketOwned) -> (String, String) {
    let source_ip = owned
        .flow
        .internet
        .as_ref()
        .and_then(|i| i.source_ip)
        .map(|ip| ip.to_string())
        .unwrap_or_default();
    let destination_ip = owned
        .flow
        .internet
        .as_ref()
        .and_then(|i| i.destination_ip)
        .map(|ip| ip.to_string())
        .unwrap_or_default();
    (source_ip, destination_ip)
}

/// État du pipeline par capture : accumulation des batches (paquets et
/// updates graphe) et accès à la matrice / au graphe partagés.
struct PacketWorker {
    on_event: Channel<CaptureEvent<'static>>,
    /// Session de capture émettrice, reprise dans chaque événement.
    session_id: u64,
    /// LINKTYPE canonique de l'interface capturée : chaque paquet est parsé
    /// avec le décodeur de ce type de liaison, jamais Ethernet supposé (#150).
    link_type: LinkType,
    flow_matrix: Arc<Mutex<FlowMatrix>>,
    graph: Arc<Mutex<GraphData>>,
    packet_batch: Vec<CapturedPacketOwned>,
    graph_batch: GraphUpdateBatch,
    last_batch_flush: Instant,
    /// Nombre de flux de la matrice, rafraîchi à chaque update (pour les stats).
    processed: u32,
    /// Paquets parsés et intégrés à la matrice depuis le début de la session
    /// (un par paquet capturé, niveaux de tunnel non recomptés) : la
    /// catégorie « intégré » du récapitulatif (#158).
    packets_integrated: u64,
    /// Total des paquets illisibles par le parseur depuis le début de la
    /// session : la catégorie « illisible » du récapitulatif (#158).
    parse_errors_total: u64,
    /// Erreurs de parsing accumulées depuis le dernier log (rate-limiting).
    parse_errors_pending: u64,
    last_parse_error_log: Instant,
    #[cfg(feature = "capture_timing")]
    timing_logger: Option<CaptureTimingLogger>,
}

impl PacketWorker {
    fn new(
        on_event: Channel<CaptureEvent<'static>>,
        session_id: u64,
        link_type: LinkType,
        flow_matrix: Arc<Mutex<FlowMatrix>>,
        graph: Arc<Mutex<GraphData>>,
    ) -> Self {
        Self {
            on_event,
            session_id,
            link_type,
            flow_matrix,
            graph,
            packet_batch: Vec::with_capacity(PACKET_BATCH_MAX),
            graph_batch: GraphUpdateBatch::default(),
            last_batch_flush: Instant::now(),
            processed: 0,
            packets_integrated: 0,
            parse_errors_total: 0,
            parse_errors_pending: 0,
            last_parse_error_log: Instant::now(),
            #[cfg(feature = "capture_timing")]
            timing_logger: match CaptureTimingLogger::new() {
                Ok(logger) => Some(logger),
                Err(e) => {
                    error!("Capture timing log disabled: {}", e);
                    None
                }
            },
        }
    }

    /// Compte une erreur de parsing et la journalise au plus une fois par
    /// [`PARSE_ERROR_LOG_INTERVAL`] : du trafic malformé en rafale ne doit
    /// pas amplifier les logs (#147).
    fn note_parse_error(&mut self, err: &dyn std::fmt::Display) {
        self.parse_errors_total += 1;
        self.parse_errors_pending += 1;
        if self.last_parse_error_log.elapsed() >= PARSE_ERROR_LOG_INTERVAL {
            error!(
                "{} paquet(s) illisible(s) depuis {:?} (dernier : {})",
                self.parse_errors_pending,
                self.last_parse_error_log.elapsed(),
                err
            );
            self.parse_errors_pending = 0;
            self.last_parse_error_log = Instant::now();
        }
    }

    /// Traite un paquet du canal : parsing, matrice, graphe, batches.
    /// Retourne `false` si le canal IPC vers le front est cassé (le thread
    /// doit s'arrêter).
    fn process_packet(&mut self, pkt: &PacketBuffer) -> bool {
        #[cfg(feature = "capture_timing")]
        let timing_sample = self
            .timing_logger
            .as_mut()
            .and_then(CaptureTimingLogger::next_sample);
        #[cfg(feature = "capture_timing")]
        let pipeline_start = timing_sample.map(|_| Instant::now());

        #[cfg(feature = "capture_timing")]
        let (flow, parse_timing) = if timing_sample.is_some() {
            match parse_packet_flow_with_timing(self.link_type, pkt.as_ref()) {
                Ok(parsed) => parsed,
                Err(e) => {
                    self.note_parse_error(&e);
                    return true;
                }
            }
        } else {
            match packet_parser::parse::parse(self.link_type, pkt.as_ref()) {
                Ok(flow) => (flow, ParseTiming::default()),
                Err(e) => {
                    self.note_parse_error(&e);
                    return true;
                }
            }
        };

        #[cfg(not(feature = "capture_timing"))]
        let flow = match packet_parser::parse::parse(self.link_type, pkt.as_ref()) {
            Ok(flow) => flow,
            Err(e) => {
                self.note_parse_error(&e);
                return true;
            }
        };

        self.packets_integrated += 1;
        let packet = CapturedPacket {
            ts_sec: pkt.header.ts.tv_sec,
            ts_usec: pkt.header.ts.tv_usec,
            caplen: pkt.header.caplen,
            len: pkt.header.len,
            flow,
        };

        #[cfg(feature = "capture_timing")]
        let packet_owned_start = timing_sample.map(|_| Instant::now());
        let record_owned = packet.to_owned_packet();
        #[cfg(feature = "capture_timing")]
        let packet_owned_ns = packet_owned_start.map(elapsed_ns_since).unwrap_or(0);

        // Un seul verrouillage de la matrice par paquet : lookup des labels
        // puis update du flux dans le même scope.
        #[cfg(feature = "capture_timing")]
        let mut label_lookup_ns = 0u64;
        #[cfg(feature = "capture_timing")]
        let mut matrix_update_ns = 0u64;
        let (source_label, destination_label) =
            if let Ok(mut locked_state) = self.flow_matrix.lock() {
                #[cfg(feature = "capture_timing")]
                let label_lookup_start = timing_sample.map(|_| Instant::now());
                let (source_ip, destination_ip) = flow_ips(&record_owned);
                let link = LinkView::of(&record_owned.flow.data_link);
                let labels = (
                    locked_state.get_label(&link.source_mac, &source_ip),
                    locked_state.get_label(&link.destination_mac, &destination_ip),
                );
                #[cfg(feature = "capture_timing")]
                {
                    label_lookup_ns = label_lookup_start.map(elapsed_ns_since).unwrap_or(0);
                }

                #[cfg(feature = "capture_timing")]
                let matrix_update_start = timing_sample.map(|_| Instant::now());
                locked_state.update_flow(&record_owned);
                self.processed = locked_state.row_count() as u32;
                #[cfg(feature = "capture_timing")]
                {
                    matrix_update_ns = matrix_update_start.map(elapsed_ns_since).unwrap_or(0);
                }

                labels
            } else {
                (None, None)
            };

        #[cfg(feature = "capture_timing")]
        let graph_update_start = timing_sample.map(|_| Instant::now());
        let graph_updates = if let Ok(mut g) = self.graph.lock() {
            g.add_packet_flow(
                &record_owned.flow,
                source_label,
                destination_label,
                1,
                record_owned.len as u64,
                record_owned.encap_id.as_slice(),
            )
        } else {
            Vec::new()
        };
        #[cfg(feature = "capture_timing")]
        let graph_update_ns = graph_update_start.map(elapsed_ns_since).unwrap_or(0);

        // Les updates graphe sont coalescées puis envoyées par lot, au même
        // rythme que le batch de paquets.
        #[cfg(feature = "capture_timing")]
        let graph_ipc_start = timing_sample.map(|_| Instant::now());
        #[cfg(feature = "capture_timing")]
        let graph_update_count = graph_updates.len();
        for update in graph_updates {
            self.graph_batch.push(update);
        }
        let graph_flush_ok = self.graph_batch.len() < GRAPH_BATCH_MAX || self.flush_graph_batch();
        #[cfg(feature = "capture_timing")]
        let graph_ipc_ns = graph_ipc_start.map(elapsed_ns_since).unwrap_or(0);
        if !graph_flush_ok {
            return false;
        }

        #[cfg(feature = "capture_timing")]
        if let (Some(sample), Some(start)) = (timing_sample, pipeline_start)
            && let Some(logger) = self.timing_logger.as_mut()
        {
            let pipeline_timing = CapturePipelineTiming {
                caplen: pkt.header.caplen,
                len: pkt.header.len,
                parse_l2_ns: parse_timing.l2_ns,
                parse_l3_ns: parse_timing.l3_ns,
                parse_l4_ns: parse_timing.l4_ns,
                parse_l7_ns: parse_timing.l7_ns,
                parse_total_ns: parse_timing.total_ns,
                packet_owned_ns,
                label_lookup_ns,
                matrix_update_ns,
                graph_update_ns,
                graph_ipc_ns,
                graph_updates: graph_update_count,
                graph_ipc_failures: 0,
                pipeline_total_ns: elapsed_ns_since(start),
            };

            if let Err(e) = logger.write_pipeline(sample, pipeline_timing) {
                error!("Capture timing log disabled after write error: {}", e);
                self.timing_logger = None;
            }
        }

        self.packet_batch.push(record_owned);

        // Tunnels (ex. CAPWAP) : chaque niveau interne devient une ligne de
        // flux supplémentaire. La garde `inner.is_some()` assure un coût nul
        // pour le trafic normal (non tunnelé).
        if packet.flow.inner.is_some() {
            self.process_inner_tunnels(&packet);
        }

        // Flush si le batch est plein ou si l'intervalle est écoulé.
        if (self.packet_batch.len() >= PACKET_BATCH_MAX
            || self.last_batch_flush.elapsed() >= PACKET_BATCH_INTERVAL)
            && !self.flush_batches()
        {
            return false;
        }

        true
    }

    /// Intègre un paquet à la matrice et au graphe sans rien émettre sur le
    /// canal IPC (chemin de drainage : le front est arrêté ou injoignable).
    fn ingest_packet_silently(&mut self, pkt: &PacketBuffer) {
        let Ok(flow) = packet_parser::parse::parse(self.link_type, pkt.as_ref()) else {
            // Même comptabilité que le chemin nominal : un paquet drainé
            // illisible reste un paquet classé, pas un paquet évaporé (#158).
            self.parse_errors_total += 1;
            return;
        };
        self.packets_integrated += 1;
        let packet = CapturedPacket {
            ts_sec: pkt.header.ts.tv_sec,
            ts_usec: pkt.header.ts.tv_usec,
            caplen: pkt.header.caplen,
            len: pkt.header.len,
            flow,
        };
        for owned in packet.to_owned_packets() {
            let (source_label, destination_label) = self.resolve_labels_and_update_matrix(&owned);
            if let Ok(mut g) = self.graph.lock() {
                g.add_packet_flow(
                    &owned.flow,
                    source_label,
                    destination_label,
                    1,
                    owned.len as u64,
                    owned.encap_id.as_slice(),
                );
            }
        }
    }

    /// Draine le canal après l'arrêt : les paquets déjà acceptés par le
    /// pipeline (jusqu'à `chan_capacity`) sont intégrés à la matrice et au
    /// graphe au lieu d'être jetés — le relevé exporté reste fidèle à ce qui
    /// a été capturé. Consomme jusqu'à la déconnexion (le thread de capture
    /// sort et lâche l'émetteur), pour couvrir aussi les paquets envoyés
    /// entre la levée du drapeau d'arrêt et la sortie effective de la
    /// capture. Retourne le nombre de paquets récupérés.
    fn drain_channel(
        &mut self,
        rx: &Receiver<CaptureMessage>,
        buffer_pool: &PacketBufferPool,
    ) -> usize {
        let mut drained = 0;
        loop {
            match rx.recv_timeout(DRAIN_RECV_TIMEOUT) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    self.ingest_packet_silently(&pkt);
                    buffer_pool.put(pkt);
                    drained += 1;
                }
                // Émetteur lâché et canal vide : drainage complet.
                Err(RecvTimeoutError::Disconnected) => break,
                // Garde-fou : l'émetteur vit encore mais plus rien n'arrive
                // (ne devrait pas se produire à l'arrêt, le thread de capture
                // sort dans son timeout pcap).
                Err(RecvTimeoutError::Timeout) => break,
            }
        }
        if drained > 0 {
            info!("Arrêt : {drained} paquet(s) drainés du canal vers la matrice");
        }
        drained
    }

    /// Niveaux internes d'un paquet tunnelé : matrice, graphe et batch pour
    /// chaque flux transporté (le niveau externe est déjà traité).
    fn process_inner_tunnels(&mut self, packet: &CapturedPacket<'_>) {
        for inner_owned in packet.to_owned_packets().into_iter().skip(1) {
            let (source_label, destination_label) =
                self.resolve_labels_and_update_matrix(&inner_owned);
            if let Ok(mut g) = self.graph.lock() {
                for update in g.add_packet_flow(
                    &inner_owned.flow,
                    source_label,
                    destination_label,
                    1,
                    inner_owned.len as u64,
                    inner_owned.encap_id.as_slice(),
                ) {
                    self.graph_batch.push(update);
                }
            }
            self.packet_batch.push(inner_owned);
        }
    }

    /// Un seul verrouillage de la matrice : résolution des labels puis update
    /// du flux dans le même scope.
    fn resolve_labels_and_update_matrix(
        &mut self,
        owned: &CapturedPacketOwned,
    ) -> (Option<String>, Option<String>) {
        let Ok(mut locked_state) = self.flow_matrix.lock() else {
            return (None, None);
        };
        let (source_ip, destination_ip) = flow_ips(owned);
        let link = LinkView::of(&owned.flow.data_link);
        let labels = (
            locked_state.get_label(&link.source_mac, &source_ip),
            locked_state.get_label(&link.destination_mac, &destination_ip),
        );
        locked_state.update_flow(owned);
        self.processed = locked_state.row_count() as u32;
        labels
    }

    /// Envoie le batch de paquets au front. Retourne false si le canal est cassé.
    fn flush_packet_batch(&mut self) -> bool {
        if self.packet_batch.is_empty() {
            return true;
        }
        #[cfg(feature = "capture_timing")]
        let batch_len = self.packet_batch.len();
        let mut packets = std::mem::take(&mut self.packet_batch);
        // Seule la queue du batch part vers la WebView (voir
        // PACKET_BATCH_UI_MAX) ; matrice et graphe ont déjà tout intégré.
        if packets.len() > PACKET_BATCH_UI_MAX {
            packets.drain(..packets.len() - PACKET_BATCH_UI_MAX);
        }
        self.last_batch_flush = Instant::now();
        #[cfg(feature = "capture_timing")]
        let ipc_start = Instant::now();
        let send_result = self.on_event.send(CaptureEvent::PacketBatch {
            session_id: self.session_id,
            packets,
        });
        #[cfg(feature = "capture_timing")]
        if let Some(logger) = self.timing_logger.as_mut()
            && let Err(e) = logger.write_packet_batch_ipc(
                batch_len,
                elapsed_ns_since(ipc_start),
                send_result.is_ok(),
            )
        {
            error!(
                "Capture timing log disabled after batch IPC write error: {}",
                e
            );
            self.timing_logger = None;
        }

        match send_result {
            Ok(_) => true,
            Err(e) => {
                error!("[TAURI] Erreur envoi PacketBatch: {}", e);
                false
            }
        }
    }

    /// Envoie les updates graphe coalescées. Retourne false si le canal est cassé.
    fn flush_graph_batch(&mut self) -> bool {
        if self.graph_batch.is_empty() {
            return true;
        }
        let updates = self.graph_batch.take();
        match self.on_event.send(CaptureEvent::GraphBatch {
            session_id: self.session_id,
            updates,
        }) {
            Ok(_) => true,
            Err(e) => {
                error!("[TAURI] Erreur envoi GraphBatch: {}", e);
                false
            }
        }
    }

    /// Flush les deux batches (paquets puis graphe). Retourne false si le
    /// canal est cassé.
    fn flush_batches(&mut self) -> bool {
        self.flush_packet_batch() && self.flush_graph_batch()
    }

    #[cfg(feature = "capture_timing")]
    fn write_run_summary(&mut self, buffer_pool: &PacketBufferPool) {
        if let Some(logger) = self.timing_logger.as_mut()
            && let Err(e) = logger.write_run_summary(buffer_pool)
        {
            error!("Capture timing summary write failed: {}", e);
        }
    }
}

/// Émission périodique vers le front des stats de capture et de l'occupation
/// du canal, dédupliquées par leurs payloads respectifs.
struct StatsEmitter {
    session_id: u64,
    /// Dernier snapshot émis : unité de déduplication des `Stats`.
    last: Option<StatsSnapshot>,
    emit_interval: Duration,
    last_emit: Instant,
    last_channel: ChannelCapacityPayload,
    last_channel_update: Instant,
    channel_capacity: usize,
    shared_stats: Arc<SharedCaptureStats>,
    drop_counters: Arc<AppDropCounters>,
}

impl StatsEmitter {
    fn new(
        channel_capacity: usize,
        shared_stats: Arc<SharedCaptureStats>,
        drop_counters: Arc<AppDropCounters>,
        session_id: u64,
    ) -> Self {
        Self {
            session_id,
            last: None,
            emit_interval: Duration::from_millis(STATS_EMIT_INTERVAL_MS),
            last_emit: Instant::now(),
            last_channel: ChannelCapacityPayload::default(),
            last_channel_update: Instant::now(),
            channel_capacity,
            shared_stats,
            drop_counters,
        }
    }

    /// Snapshot courant : stats pcap partagées (hors canal de données, donc
    /// fiables même sous saturation) + comptabilité du worker.
    fn snapshot(&self, worker: &PacketWorker) -> StatsSnapshot {
        let mut triple = self.shared_stats.load();
        triple.app_dropped = self.drop_counters.total();
        StatsSnapshot {
            triple,
            parse_errors: worker.parse_errors_total,
            processed: worker.processed,
        }
    }

    fn tick(&mut self, queue_len: usize, worker: &PacketWorker) {
        if self.last_emit.elapsed() >= self.emit_interval {
            self.last_emit = Instant::now();
            let current = self.snapshot(worker);
            if let Err(e) =
                StatsPayload::maybe_send(&mut self.last, current, self.session_id, &worker.on_event)
            {
                error!("[TAURI] Erreur envoi Stats: {}", e);
            }
        }

        if self.last_channel_update.elapsed() >= CHANNEL_EMIT_MIN_INTERVAL {
            self.last_channel_update = Instant::now();

            if let Err(e) = ChannelCapacityPayload::send_if_changed(
                &mut self.last_channel,
                queue_len,
                self.channel_capacity,
                self.session_id,
                &worker.on_event,
            ) {
                error!("[TAURI] Erreur émission canal : {}", e);
            }
        }
    }

    /// Récapitulatif final, émis inconditionnellement après le drainage
    /// d'arrêt ou de plafond (#158) : les derniers compteurs affichés
    /// incluent les paquets drainés, et la somme des catégories boucle avec
    /// les paquets reçus. Best-effort : le canal IPC peut être mort à ce
    /// stade, l'échec est seulement journalisé.
    fn send_final(&mut self, worker: &PacketWorker) {
        let current = self.snapshot(worker);
        self.last = Some(current);
        if let Err(e) = StatsPayload::new(current, self.session_id).send(&worker.on_event) {
            error!("[TAURI] Erreur envoi Stats final : {}", e);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_processing_thread(
    rx: Receiver<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
    drop_counters: Arc<AppDropCounters>,
    shared_stats: Arc<SharedCaptureStats>,
    stop_flag: Arc<AtomicBool>,
    session_id: u64,
    link_type: LinkType,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        // Résolus une seule fois : le lookup d'état Tauri par paquet est inutile.
        let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>().inner().clone();
        let graph = app.state::<Arc<Mutex<GraphData>>>().inner().clone();

        let mut worker = PacketWorker::new(on_event, session_id, link_type, flow_matrix, graph);
        let mut emitter = StatsEmitter::new(
            channel_capacity as usize,
            shared_stats,
            drop_counters,
            session_id,
        );

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                // Arrêt demandé : les paquets déjà acceptés dans le canal
                // sont intégrés à la matrice (fidélité du relevé), puis les
                // derniers batches et le récapitulatif final partent vers le
                // front en best-effort (#158).
                worker.drain_channel(&rx, &buffer_pool);
                let _ = worker.flush_batches();
                emitter.send_final(&worker);
                break;
            }

            let timeout = PACKET_BATCH_INTERVAL.saturating_sub(worker.last_batch_flush.elapsed());
            match rx.recv_timeout(timeout.max(Duration::from_millis(1))) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        // Arrêt reçu entre deux paquets : celui-ci et le reste
                        // du canal sont intégrés à la matrice avant de sortir.
                        worker.ingest_packet_silently(&pkt);
                        buffer_pool.put(pkt);
                        worker.drain_channel(&rx, &buffer_pool);
                        let _ = worker.flush_batches();
                        emitter.send_final(&worker);
                        break;
                    }

                    let keep_going = worker.process_packet(&pkt);
                    buffer_pool.put(pkt);
                    if !keep_going {
                        // Canal IPC vers le front cassé : sans arrêt explicite,
                        // le thread de capture continuerait seul à remplir le
                        // canal (capture fantôme). On stoppe tout le pipeline,
                        // en gardant les paquets déjà acceptés dans la matrice.
                        error!("Canal IPC frontend cassé : arrêt du pipeline de capture");
                        stop_flag.store(true, Ordering::Relaxed);
                        worker.drain_channel(&rx, &buffer_pool);
                        break;
                    }

                    if worker.processed >= MAX_LIVE_FLOWS {
                        // Plafond de flux atteint : arrêt propre et explicite
                        // plutôt qu'une éviction silencieuse (le relevé
                        // mentirait) ou un épuisement mémoire (#147). Comme à
                        // l'arrêt demandé, les paquets déjà acceptés dans le
                        // canal sont drainés vers la matrice : le dépassement
                        // du plafond est borné par la taille du canal, alors
                        // que les jeter serait une perte non comptée (#158).
                        error!("Plafond de {MAX_LIVE_FLOWS} flux atteint : arrêt de la capture");
                        stop_flag.store(true, Ordering::Relaxed);
                        worker.drain_channel(&rx, &buffer_pool);
                        let _ = worker.flush_batches();
                        emitter.send_final(&worker);
                        let _ = worker.on_event.send(CaptureEvent::Stopped {
                            session_id,
                            reason: format!(
                                "plafond de {MAX_LIVE_FLOWS} flux atteint : capture arrêtée \
                                 pour préserver la mémoire, le relevé reste fidèle jusqu'ici"
                            ),
                        });
                        break;
                    }
                }

                Err(RecvTimeoutError::Timeout) => {
                    // Flush les batches restants après inactivité.
                    if !worker.flush_batches() {
                        error!("Canal IPC frontend cassé : arrêt du pipeline de capture");
                        stop_flag.store(true, Ordering::Relaxed);
                        worker.drain_channel(&rx, &buffer_pool);
                        break;
                    }
                }

                Err(RecvTimeoutError::Disconnected) => {
                    // Le thread de capture est mort : on récupère ce qui reste
                    // dans le canal avant de sortir.
                    error!("Erreur réception canal : canal déconnecté");
                    worker.drain_channel(&rx, &buffer_pool);
                    let _ = worker.flush_batches();
                    emitter.send_final(&worker);
                    break;
                }
            }

            emitter.tick(rx.len(), &worker);
        }

        #[cfg(feature = "capture_timing")]
        worker.write_run_summary(&buffer_pool);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::bounded;

    /// Trame ARP request complète (42 octets) : parsée par packet_parser,
    /// elle produit une ligne de matrice.
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

    fn packet_message(pool: &PacketBufferPool, data: &[u8]) -> CaptureMessage {
        let mut buffer = pool.get(data.len()).expect("buffer disponible");
        let header = pcap::PacketHeader {
            ts: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            caplen: data.len() as u32,
            len: data.len() as u32,
        };
        buffer.write_from_parts(&header, data);
        CaptureMessage::Packet(buffer)
    }

    /// Fidélité à l'arrêt : les paquets encore dans le canal sont intégrés à
    /// la matrice au lieu d'être jetés.
    #[test]
    fn drain_channel_ingests_pending_packets_into_matrix() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let mut worker = PacketWorker::new(
            Channel::new(|_| Ok(())),
            1,
            LinkType::ETHERNET,
            flow_matrix.clone(),
            graph.clone(),
        );
        let pool = PacketBufferPool::new(8, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(8);

        for _ in 0..3 {
            tx.send(packet_message(&pool, &arp_frame())).unwrap();
        }
        // Le thread de capture sort et lâche l'émetteur : le drainage doit
        // consommer tout le buffer restant puis s'arrêter sur la déconnexion.
        drop(tx);

        let drained = worker.drain_channel(&rx, &pool);

        assert_eq!(drained, 3, "tous les paquets en attente sont consommés");
        assert!(rx.is_empty());
        assert_eq!(
            worker.packets_integrated, 3,
            "chaque paquet drainé est compté intégré (#158)"
        );
        assert_eq!(worker.parse_errors_total, 0);
        let matrix = flow_matrix.lock().unwrap();
        assert_eq!(matrix.row_count(), 1, "3 paquets du même flux -> 1 ligne");
        let packets: u64 = matrix
            .matrix
            .values()
            .flat_map(|entries| entries.iter())
            .map(|(_, stats)| stats.count)
            .sum();
        assert_eq!(packets, 3, "aucun paquet accepté n'est perdu");
        assert!(
            !graph.lock().unwrap().nodes.is_empty(),
            "le graphe reçoit les nœuds"
        );
    }

    /// Un paquet illisible ne bloque pas le drainage.
    #[test]
    fn drain_channel_skips_unparseable_packets() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let mut worker = PacketWorker::new(
            Channel::new(|_| Ok(())),
            1,
            LinkType::ETHERNET,
            flow_matrix.clone(),
            graph,
        );
        let pool = PacketBufferPool::new(8, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(8);

        tx.send(packet_message(&pool, &[0x00, 0x01, 0x02])).unwrap();
        tx.send(packet_message(&pool, &arp_frame())).unwrap();
        drop(tx);

        let drained = worker.drain_channel(&rx, &pool);

        assert_eq!(drained, 2, "le paquet illisible est consommé sans bloquer");
        assert_eq!(flow_matrix.lock().unwrap().row_count(), 1);
        // Comptabilité exhaustive (#158) : chaque paquet accepté par le
        // pipeline est classé — intégré ou illisible — jamais évaporé.
        assert_eq!(worker.packets_integrated, 1);
        assert_eq!(worker.parse_errors_total, 1);
        assert_eq!(
            worker.packets_integrated + worker.parse_errors_total,
            drained as u64,
            "la somme des catégories boucle avec les paquets drainés"
        );
    }

    /// Le chemin nominal tient la même comptabilité que le drainage : après
    /// un mélange de paquets lisibles et illisibles, la somme des catégories
    /// égale les paquets traités (#158).
    #[test]
    fn process_packet_categorizes_every_accepted_packet() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let mut worker = PacketWorker::new(
            Channel::new(|_| Ok(())),
            1,
            LinkType::ETHERNET,
            flow_matrix,
            graph,
        );
        let pool = PacketBufferPool::new(8, 65_536);

        let frames: [&[u8]; 5] = [
            &arp_frame(),
            &[0x00, 0x01, 0x02], // tronqué : illisible
            &arp_frame(),
            &[0xff; 4], // tronqué : illisible
            &arp_frame(),
        ];
        for data in frames {
            let CaptureMessage::Packet(buffer) = packet_message(&pool, data);
            assert!(worker.process_packet(&buffer), "canal IPC de test vivant");
            pool.put(buffer);
        }

        assert_eq!(worker.packets_integrated, 3);
        assert_eq!(worker.parse_errors_total, 2);
        assert_eq!(
            worker.packets_integrated + worker.parse_errors_total,
            frames.len() as u64,
            "chaque paquet accepté appartient à une catégorie explicite"
        );
    }
}
