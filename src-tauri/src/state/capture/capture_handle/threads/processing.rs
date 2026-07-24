//! Thread de traitement de la capture live : consomme les paquets du canal,
//! met à jour la matrice de flux et le graphe, et pousse au front les batches
//! de paquets, les updates graphe et les stats périodiques.

use crossbeam::channel::{Receiver, RecvTimeoutError, Sender};
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
            PipelineThreadGuard, TerminalState,
            messages::{
                CaptureMessage,
                capture::{CapturedPacket, CapturedPacketOwned},
                channel::ChannelCapacityPayload,
                stats::{AppDropCounters, SharedCaptureStats, StatsPayload, StatsSnapshot},
            },
            threads::packet_buffer::{PacketBuffer, PacketBufferPool},
        },
        flow_matrix::FlowMatrix,
        graph::{GraphData, GraphUpdate, GraphUpdateBatch},
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
/// Marge appliquée au timeout pcap configuré pour borner un silence pendant
/// le drainage (#166) : le timeout pcap ne borne qu'un seul appel
/// `next_packet`, pas toute la boucle du thread de capture (stats, envoi
/// canal…) — une marge évite un faux positif sur une machine chargée.
const DRAIN_STALL_MARGIN: u32 = 4;
/// Plancher de la borne de silence : avec un timeout pcap très court
/// (config minimale 1 ms), la marge seule couvrirait à peine la sortie du
/// thread de capture (join, drop du sender).
const DRAIN_STALL_MIN_BOUND: Duration = Duration::from_secs(2);

/// Borne de silence tolérée pendant le drainage avant de considérer le
/// producteur injoignable. Un producteur mort de façon normale lâche son
/// `Sender` bien avant cette borne (`Disconnected`) ; elle ne protège que
/// contre un producteur qui ne se déconnecte jamais (ex. thread de capture
/// réellement bloqué dans un appel pcap natif — risque distinct, hors
/// périmètre de #166, cf. la décision Npcap #138).
fn drain_stall_bound(pcap_timeout: Duration) -> Duration {
    pcap_timeout
        .saturating_mul(DRAIN_STALL_MARGIN)
        .max(DRAIN_STALL_MIN_BOUND)
}
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

/// Erreur fatale d'intégrité du pipeline : contrairement à une erreur de
/// parsing, elle invalide la mise à jour partagée et impose l'arrêt immédiat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProcessingFatal {
    MatrixPoisoned,
    GraphPoisoned,
    /// Le producteur (thread de capture) n'a ni envoyé de paquet ni lâché
    /// son émetteur depuis [`drain_stall_bound`] : plutôt qu'un drainage
    /// qui n'a plus aucune raison structurelle de se terminer, la capture
    /// est arrêtée avec une cause observable (#166).
    ProducerStalled,
}

impl ProcessingFatal {
    fn reason(self) -> &'static str {
        match self {
            Self::MatrixPoisoned => "erreur fatale d'intégrité : verrou de la matrice empoisonné",
            Self::GraphPoisoned => "erreur fatale d'intégrité : verrou du graphe empoisonné",
            Self::ProducerStalled => {
                "erreur fatale : producteur de capture injoignable, drainage interrompu après un délai anormal"
            }
        }
    }
}

impl std::fmt::Display for ProcessingFatal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason())
    }
}

/// Résultat d'une transaction matrice + graphe. Les deux verrous sont acquis
/// avant la première mutation, toujours dans l'ordre global matrice → graphe.
struct IntegratedLevels {
    processed: u32,
    graph_updates: Vec<GraphUpdate>,
    #[cfg(feature = "capture_timing")]
    label_lookup_ns: u64,
    #[cfg(feature = "capture_timing")]
    matrix_update_ns: u64,
    #[cfg(feature = "capture_timing")]
    graph_update_ns: u64,
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
    /// Silence maximal toléré entre deux messages pendant le drainage avant
    /// de considérer le producteur injoignable (#166).
    drain_stall_bound: Duration,
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
        drain_stall_bound: Duration,
    ) -> Self {
        Self {
            on_event,
            session_id,
            link_type,
            flow_matrix,
            graph,
            drain_stall_bound,
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

    /// Intègre atomiquement tous les niveaux d'une même trame physique.
    ///
    /// Ordre global des verrous partagés : `FlowMatrix` puis `GraphData`.
    /// Les deux sont acquis avant toute mutation : si le graphe est déjà
    /// empoisonné, la matrice reste donc strictement inchangée (#166).
    fn integrate_owned_levels(
        &self,
        levels: &[CapturedPacketOwned],
    ) -> Result<IntegratedLevels, ProcessingFatal> {
        let mut matrix = self
            .flow_matrix
            .lock()
            .map_err(|_| ProcessingFatal::MatrixPoisoned)?;
        let mut graph = self
            .graph
            .lock()
            .map_err(|_| ProcessingFatal::GraphPoisoned)?;

        let mut graph_updates = Vec::new();
        #[cfg(feature = "capture_timing")]
        let mut label_lookup_ns = 0u64;
        #[cfg(feature = "capture_timing")]
        let mut matrix_update_ns = 0u64;
        #[cfg(feature = "capture_timing")]
        let mut graph_update_ns = 0u64;

        for owned in levels {
            #[cfg(feature = "capture_timing")]
            let label_lookup_start = Instant::now();
            let (source_ip, destination_ip) = flow_ips(owned);
            let link = LinkView::of(&owned.flow.data_link);
            let labels = (
                matrix.get_label(&link.source_mac, &source_ip),
                matrix.get_label(&link.destination_mac, &destination_ip),
            );
            #[cfg(feature = "capture_timing")]
            {
                label_lookup_ns =
                    label_lookup_ns.saturating_add(elapsed_ns_since(label_lookup_start));
            }

            #[cfg(feature = "capture_timing")]
            let matrix_update_start = Instant::now();
            matrix.update_flow(owned);
            #[cfg(feature = "capture_timing")]
            {
                matrix_update_ns =
                    matrix_update_ns.saturating_add(elapsed_ns_since(matrix_update_start));
            }

            #[cfg(feature = "capture_timing")]
            let graph_update_start = Instant::now();
            graph_updates.extend(graph.add_packet_flow(
                &owned.flow,
                labels.0,
                labels.1,
                1,
                owned.len as u64,
                owned.encap_id.as_slice(),
            ));
            #[cfg(feature = "capture_timing")]
            {
                graph_update_ns =
                    graph_update_ns.saturating_add(elapsed_ns_since(graph_update_start));
            }
        }

        Ok(IntegratedLevels {
            processed: matrix.row_count() as u32,
            graph_updates,
            #[cfg(feature = "capture_timing")]
            label_lookup_ns,
            #[cfg(feature = "capture_timing")]
            matrix_update_ns,
            #[cfg(feature = "capture_timing")]
            graph_update_ns,
        })
    }

    /// Traite un paquet du canal : parsing, matrice, graphe, batches.
    /// Retourne `false` si le canal IPC vers le front est cassé (le thread
    /// doit s'arrêter).
    fn process_packet(&mut self, pkt: &PacketBuffer) -> Result<bool, ProcessingFatal> {
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
                    return Ok(true);
                }
            }
        } else {
            match packet_parser::parse::parse(self.link_type, pkt.as_ref()) {
                Ok(flow) => (flow, ParseTiming::default()),
                Err(e) => {
                    self.note_parse_error(&e);
                    return Ok(true);
                }
            }
        };

        #[cfg(not(feature = "capture_timing"))]
        let flow = match packet_parser::parse::parse(self.link_type, pkt.as_ref()) {
            Ok(flow) => flow,
            Err(e) => {
                self.note_parse_error(&e);
                return Ok(true);
            }
        };
        let packet = CapturedPacket {
            ts_sec: pkt.header.ts.tv_sec,
            ts_usec: pkt.header.ts.tv_usec,
            caplen: pkt.header.caplen,
            len: pkt.header.len,
            flow,
        };

        #[cfg(feature = "capture_timing")]
        let packet_owned_start = timing_sample.map(|_| Instant::now());
        let owned_levels = packet.to_owned_packets();
        #[cfg(feature = "capture_timing")]
        let packet_owned_ns = packet_owned_start.map(elapsed_ns_since).unwrap_or(0);

        // Transaction unique pour l'externe et tous les niveaux tunnelés. Un
        // poison ne peut plus laisser matrice et graphe partiellement divergents.
        let integration = self.integrate_owned_levels(&owned_levels)?;
        self.processed = integration.processed;
        self.packets_integrated += 1;

        // Les updates graphe sont coalescées puis envoyées par lot, au même
        // rythme que le batch de paquets.
        #[cfg(feature = "capture_timing")]
        let graph_ipc_start = timing_sample.map(|_| Instant::now());
        #[cfg(feature = "capture_timing")]
        let graph_update_count = integration.graph_updates.len();
        for update in integration.graph_updates {
            self.graph_batch.push(update);
        }
        let graph_flush_ok = self.graph_batch.len() < GRAPH_BATCH_MAX || self.flush_graph_batch();
        #[cfg(feature = "capture_timing")]
        let graph_ipc_ns = graph_ipc_start.map(elapsed_ns_since).unwrap_or(0);
        if !graph_flush_ok {
            return Ok(false);
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
                label_lookup_ns: integration.label_lookup_ns,
                matrix_update_ns: integration.matrix_update_ns,
                graph_update_ns: integration.graph_update_ns,
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

        self.packet_batch.extend(owned_levels);

        // Flush si le batch est plein ou si l'intervalle est écoulé.
        if (self.packet_batch.len() >= PACKET_BATCH_MAX
            || self.last_batch_flush.elapsed() >= PACKET_BATCH_INTERVAL)
            && !self.flush_batches()
        {
            return Ok(false);
        }

        Ok(true)
    }

    /// Intègre un paquet à la matrice et au graphe sans rien émettre sur le
    /// canal IPC (chemin de drainage : le front est arrêté ou injoignable).
    fn ingest_packet_silently(&mut self, pkt: &PacketBuffer) -> Result<(), ProcessingFatal> {
        let flow = match packet_parser::parse::parse(self.link_type, pkt.as_ref()) {
            Ok(flow) => flow,
            Err(e) => {
                // Même comptabilité que le chemin nominal : un paquet drainé
                // illisible reste un paquet classé, pas un paquet évaporé (#158).
                self.note_parse_error(&e);
                return Ok(());
            }
        };
        let packet = CapturedPacket {
            ts_sec: pkt.header.ts.tv_sec,
            ts_usec: pkt.header.ts.tv_usec,
            caplen: pkt.header.caplen,
            len: pkt.header.len,
            flow,
        };
        let owned_levels = packet.to_owned_packets();
        let integration = self.integrate_owned_levels(&owned_levels)?;
        self.processed = integration.processed;
        self.packets_integrated += 1;
        Ok(())
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
    ) -> Result<usize, ProcessingFatal> {
        let mut drained = 0;
        let mut stall_deadline = Instant::now() + self.drain_stall_bound;
        loop {
            match rx.recv_timeout(DRAIN_RECV_TIMEOUT) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    let integration = self.ingest_packet_silently(&pkt);
                    buffer_pool.put(pkt);
                    integration?;
                    drained += 1;
                    stall_deadline = Instant::now() + self.drain_stall_bound;
                }
                // Émetteur lâché et canal vide : drainage complet.
                Err(RecvTimeoutError::Disconnected) => break,
                // L'émetteur vit encore : le timeout pcap est configurable
                // jusqu'à 10 s, donc 250 ms de silence ne signifie pas que le
                // producteur a fini. Continuer jusqu'à `Disconnected` garantit
                // que son dernier paquet et son dernier relevé pcap précèdent
                // le récapitulatif final (#158, #166) — mais un silence qui
                // dépasse `drain_stall_bound` n'est plus un timeout pcap
                // normal : le producteur est réputé injoignable plutôt que
                // d'attendre indéfiniment (#166).
                Err(RecvTimeoutError::Timeout) => {
                    if Instant::now() >= stall_deadline {
                        error!(
                            "Drainage interrompu après {:?} sans nouveau message ni déconnexion \
                             du producteur",
                            self.drain_stall_bound
                        );
                        return Err(ProcessingFatal::ProducerStalled);
                    }
                }
            }
        }
        if drained > 0 {
            info!("Arrêt : {drained} paquet(s) drainés du canal vers la matrice");
        }
        Ok(drained)
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

/// Rend au pool, sans tenter de les intégrer, les paquets qui restent après
/// une corruption des structures partagées. Attendre la déconnexion garantit
/// que le producteur a publié son dernier relevé pcap avant les stats finales.
fn discard_channel_after_fatal(
    rx: &Receiver<CaptureMessage>,
    buffer_pool: &PacketBufferPool,
    stall_bound: Duration,
) -> u64 {
    let mut discarded = 0u64;
    let mut stall_deadline = Instant::now() + stall_bound;
    loop {
        match rx.recv_timeout(DRAIN_RECV_TIMEOUT) {
            Ok(CaptureMessage::Packet(pkt)) => {
                buffer_pool.put(pkt);
                discarded += 1;
                stall_deadline = Instant::now() + stall_bound;
            }
            Err(RecvTimeoutError::Disconnected) => break,
            // Même garde-fou que `drain_channel` : on est déjà dans le
            // chemin fatal, un producteur injoignable ne doit pas non plus y
            // boucler indéfiniment (#166).
            Err(RecvTimeoutError::Timeout) => {
                if Instant::now() >= stall_deadline {
                    error!(
                        "Rejet du canal interrompu après {stall_bound:?} sans déconnexion du \
                         producteur"
                    );
                    break;
                }
            }
        }
    }
    discarded
}

/// Arrêt commun des erreurs d'intégrité. On ne retente jamais un verrou
/// empoisonné : la cause est mémorisée avant tout IPC, le producteur est
/// arrêté, puis les paquets encore acceptés sont rendus au pool et comptés
/// comme pertes applicatives. Les batches déjà cohérents et le récapitulatif
/// final ne partent qu'après la déconnexion du producteur. Le coordinateur
/// publiera `Stopped` une seule fois, après jointure et normalisation.
fn stop_on_processing_fatal(
    worker: &mut PacketWorker,
    emitter: &mut StatsEmitter,
    rx: &Receiver<CaptureMessage>,
    buffer_pool: &PacketBufferPool,
    stop_flag: &AtomicBool,
    terminal: &TerminalState,
    fatal: ProcessingFatal,
) {
    let reason = fatal.reason().to_string();
    error!("{reason}");
    // Mémoriser avant toute émission best-effort : même un callback IPC
    // fautif ne doit pas remplacer la cause d'intégrité précise par une
    // simple « panique du processing ».
    terminal.record_fatal(reason);
    stop_flag.store(true, Ordering::Release);
    // Le paquet qui a révélé le poison a déjà été retiré du canal et rendu au
    // pool par l'appelant. Il compte donc lui aussi parmi les pertes fatales.
    let discarded = 1 + discard_channel_after_fatal(rx, buffer_pool, worker.drain_stall_bound);
    emitter.drop_counters.add_processing_fatal(discarded);
    info!("Arrêt fatal : {discarded} paquet(s) accepté(s) abandonné(s) et compté(s)");
    let _ = worker.flush_batches();
    emitter.send_final(worker);
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
    terminal: Arc<TerminalState>,
    completion: Sender<()>,
    session_id: u64,
    link_type: LinkType,
    pcap_timeout: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _thread_guard = PipelineThreadGuard::new(
            "processing",
            completion,
            Arc::clone(&stop_flag),
            Arc::clone(&terminal),
        );
        debug!("Démarrage du thread de traitement");

        // Résolus une seule fois : le lookup d'état Tauri par paquet est inutile.
        let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>().inner().clone();
        let graph = app.state::<Arc<Mutex<GraphData>>>().inner().clone();

        let mut worker = PacketWorker::new(
            on_event,
            session_id,
            link_type,
            flow_matrix,
            graph,
            drain_stall_bound(pcap_timeout),
        );
        let mut emitter = StatsEmitter::new(
            channel_capacity as usize,
            shared_stats,
            drop_counters,
            session_id,
        );

        loop {
            if stop_flag.load(Ordering::Acquire) {
                // Arrêt demandé : les paquets déjà acceptés dans le canal
                // sont intégrés à la matrice (fidélité du relevé), puis les
                // derniers batches et le récapitulatif final partent vers le
                // front en best-effort (#158).
                match worker.drain_channel(&rx, &buffer_pool) {
                    Ok(_) => {
                        let _ = worker.flush_batches();
                        emitter.send_final(&worker);
                    }
                    Err(fatal) => {
                        stop_on_processing_fatal(
                            &mut worker,
                            &mut emitter,
                            &rx,
                            &buffer_pool,
                            &stop_flag,
                            &terminal,
                            fatal,
                        );
                    }
                }
                break;
            }

            let timeout = PACKET_BATCH_INTERVAL.saturating_sub(worker.last_batch_flush.elapsed());
            match rx.recv_timeout(timeout.max(Duration::from_millis(1))) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    if stop_flag.load(Ordering::Acquire) {
                        // Arrêt reçu entre deux paquets : celui-ci et le reste
                        // du canal sont intégrés à la matrice avant de sortir.
                        let integration = worker.ingest_packet_silently(&pkt);
                        buffer_pool.put(pkt);
                        let drained = integration
                            .and_then(|_| worker.drain_channel(&rx, &buffer_pool).map(|_| ()));
                        match drained {
                            Ok(()) => {
                                let _ = worker.flush_batches();
                                emitter.send_final(&worker);
                            }
                            Err(fatal) => stop_on_processing_fatal(
                                &mut worker,
                                &mut emitter,
                                &rx,
                                &buffer_pool,
                                &stop_flag,
                                &terminal,
                                fatal,
                            ),
                        }
                        break;
                    }

                    let processing = worker.process_packet(&pkt);
                    buffer_pool.put(pkt);
                    let keep_going = match processing {
                        Ok(keep_going) => keep_going,
                        Err(fatal) => {
                            stop_on_processing_fatal(
                                &mut worker,
                                &mut emitter,
                                &rx,
                                &buffer_pool,
                                &stop_flag,
                                &terminal,
                                fatal,
                            );
                            break;
                        }
                    };
                    if !keep_going {
                        // Canal IPC vers le front cassé : sans arrêt explicite,
                        // le thread de capture continuerait seul à remplir le
                        // canal (capture fantôme). On stoppe tout le pipeline,
                        // en gardant les paquets déjà acceptés dans la matrice.
                        error!("Canal IPC frontend cassé : arrêt du pipeline de capture");
                        terminal.record_autonomous(
                            "canal IPC frontend cassé : pipeline de capture arrêté".to_string(),
                        );
                        stop_flag.store(true, Ordering::Release);
                        match worker.drain_channel(&rx, &buffer_pool) {
                            Ok(_) => {
                                let _ = worker.flush_batches();
                                emitter.send_final(&worker);
                            }
                            Err(fatal) => stop_on_processing_fatal(
                                &mut worker,
                                &mut emitter,
                                &rx,
                                &buffer_pool,
                                &stop_flag,
                                &terminal,
                                fatal,
                            ),
                        }
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
                        terminal.record_autonomous(format!(
                            "plafond de {MAX_LIVE_FLOWS} flux atteint : capture arrêtée \
                             pour préserver la mémoire, le relevé reste fidèle jusqu'ici"
                        ));
                        stop_flag.store(true, Ordering::Release);
                        if let Err(fatal) = worker.drain_channel(&rx, &buffer_pool) {
                            stop_on_processing_fatal(
                                &mut worker,
                                &mut emitter,
                                &rx,
                                &buffer_pool,
                                &stop_flag,
                                &terminal,
                                fatal,
                            );
                            break;
                        }
                        let _ = worker.flush_batches();
                        emitter.send_final(&worker);
                        break;
                    }
                }

                Err(RecvTimeoutError::Timeout) => {
                    // Flush les batches restants après inactivité.
                    if !worker.flush_batches() {
                        error!("Canal IPC frontend cassé : arrêt du pipeline de capture");
                        terminal.record_autonomous(
                            "canal IPC frontend cassé : pipeline de capture arrêté".to_string(),
                        );
                        stop_flag.store(true, Ordering::Release);
                        match worker.drain_channel(&rx, &buffer_pool) {
                            Ok(_) => {
                                let _ = worker.flush_batches();
                                emitter.send_final(&worker);
                            }
                            Err(fatal) => stop_on_processing_fatal(
                                &mut worker,
                                &mut emitter,
                                &rx,
                                &buffer_pool,
                                &stop_flag,
                                &terminal,
                                fatal,
                            ),
                        }
                        break;
                    }
                }

                Err(RecvTimeoutError::Disconnected) => {
                    // Le thread de capture est mort : on récupère ce qui reste
                    // dans le canal avant de sortir.
                    error!("Erreur réception canal : canal déconnecté");
                    match worker.drain_channel(&rx, &buffer_pool) {
                        Ok(_) => {
                            let _ = worker.flush_batches();
                            emitter.send_final(&worker);
                        }
                        Err(fatal) => stop_on_processing_fatal(
                            &mut worker,
                            &mut emitter,
                            &rx,
                            &buffer_pool,
                            &stop_flag,
                            &terminal,
                            fatal,
                        ),
                    }
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

    type CapturedEvents = Arc<Mutex<Vec<serde_json::Value>>>;

    /// Généreuse pour tout test qui n'exerce pas spécifiquement la borne de
    /// silence : les scénarios existants s'arrêtent bien avant.
    const TEST_DRAIN_STALL_BOUND: Duration = Duration::from_secs(5);

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

    fn recording_channel() -> (Channel<CaptureEvent<'static>>, CapturedEvents) {
        let events = Arc::new(Mutex::new(Vec::new()));
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

    fn poison<T: Send + 'static>(mutex: &Arc<Mutex<T>>) {
        let cloned = Arc::clone(mutex);
        assert!(
            thread::spawn(move || {
                let _guard = cloned.lock().unwrap();
                panic!("empoisonnement intentionnel pour #166");
            })
            .join()
            .is_err()
        );
        assert!(mutex.is_poisoned());
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
            TEST_DRAIN_STALL_BOUND,
        );
        let pool = PacketBufferPool::new(8, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(8);

        for _ in 0..3 {
            tx.send(packet_message(&pool, &arp_frame())).unwrap();
        }
        // Le thread de capture sort et lâche l'émetteur : le drainage doit
        // consommer tout le buffer restant puis s'arrêter sur la déconnexion.
        drop(tx);

        let drained = worker.drain_channel(&rx, &pool).unwrap();

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
            TEST_DRAIN_STALL_BOUND,
        );
        let pool = PacketBufferPool::new(8, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(8);

        tx.send(packet_message(&pool, &[0x00, 0x01, 0x02])).unwrap();
        tx.send(packet_message(&pool, &arp_frame())).unwrap();
        drop(tx);

        let drained = worker.drain_channel(&rx, &pool).unwrap();

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

    /// Le timeout de polling interne n'est pas une fin de drainage : une
    /// interface configurée avec un timeout pcap plus long peut encore
    /// produire un dernier paquet avant de lâcher l'émetteur (#158).
    #[test]
    fn drain_channel_waits_for_sender_disconnect_after_a_quiet_interval() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let mut worker = PacketWorker::new(
            Channel::new(|_| Ok(())),
            1,
            LinkType::ETHERNET,
            Arc::clone(&flow_matrix),
            graph,
            TEST_DRAIN_STALL_BOUND,
        );
        let pool = Arc::new(PacketBufferPool::new(1, 65_536));
        let (tx, rx) = bounded::<CaptureMessage>(1);
        let pool_for_sender = Arc::clone(&pool);
        let sender = thread::spawn(move || {
            thread::sleep(DRAIN_RECV_TIMEOUT + Duration::from_millis(25));
            tx.send(packet_message(&pool_for_sender, &arp_frame()))
                .unwrap();
        });

        let drained = worker.drain_channel(&rx, &pool).unwrap();
        sender.join().unwrap();

        assert_eq!(drained, 1);
        assert_eq!(worker.packets_integrated, 1);
        assert_eq!(flow_matrix.lock().unwrap().row_count(), 1);
    }

    #[test]
    fn drain_stall_bound_applies_margin_and_floor() {
        assert_eq!(
            drain_stall_bound(Duration::from_millis(25)),
            DRAIN_STALL_MIN_BOUND,
            "un timeout pcap court reste couvert par le plancher"
        );
        assert_eq!(
            drain_stall_bound(Duration::from_secs(1)),
            Duration::from_secs(1) * DRAIN_STALL_MARGIN,
            "un timeout pcap long est multiplié par la marge"
        );
    }

    /// Garde-fou #166 : un producteur vivant qui ne se déconnecte jamais et
    /// n'envoie plus rien (ex. thread de capture réellement bloqué) ne doit
    /// pas transformer le drainage en attente infinie. Sans la borne, ce
    /// test n'aurait structurellement aucune raison de se terminer ; il est
    /// donc la preuve directe que `drain_channel` est borné en pratique.
    #[test]
    fn drain_channel_reports_a_stalled_producer_instead_of_looping_forever() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let mut worker = PacketWorker::new(
            Channel::new(|_| Ok(())),
            1,
            LinkType::ETHERNET,
            flow_matrix,
            graph,
            Duration::from_millis(1),
        );
        let pool = PacketBufferPool::new(1, 65_536);
        // `tx` est délibérément conservé vivant jusqu'à la fin du test : la
        // sortie de `drain_channel` doit venir de la borne de silence, pas
        // d'une déconnexion.
        let (tx, rx) = bounded::<CaptureMessage>(1);

        let result = worker.drain_channel(&rx, &pool);

        assert_eq!(result, Err(ProcessingFatal::ProducerStalled));
        assert!(
            tx.send(packet_message(&pool, &arp_frame())).is_ok(),
            "le canal n'a jamais été déconnecté : la sortie vient bien de la borne de silence"
        );
    }

    /// Une corruption découverte pendant l'épilogue d'arrêt suit le même
    /// chemin fatal que le traitement nominal. Le buffer est rendu au pool
    /// avant la propagation, et le paquet n'est jamais compté intégré.
    #[test]
    fn drain_channel_returns_buffer_and_propagates_graph_poison() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        poison(&graph);
        let mut worker = PacketWorker::new(
            Channel::new(|_| Ok(())),
            1,
            LinkType::ETHERNET,
            Arc::clone(&flow_matrix),
            graph,
            TEST_DRAIN_STALL_BOUND,
        );
        let pool = PacketBufferPool::new(1, 65_536);
        let (tx, rx) = bounded::<CaptureMessage>(1);
        tx.send(packet_message(&pool, &arp_frame())).unwrap();
        drop(tx);

        let result = worker.drain_channel(&rx, &pool);

        assert_eq!(result, Err(ProcessingFatal::GraphPoisoned));
        assert_eq!(pool.len(), 1, "le buffer est rendu même sur erreur fatale");
        assert_eq!(worker.packets_integrated, 0);
        assert_eq!(flow_matrix.lock().unwrap().row_count(), 0);
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
            TEST_DRAIN_STALL_BOUND,
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
            assert!(
                worker.process_packet(&buffer).unwrap(),
                "canal IPC de test vivant"
            );
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

    #[test]
    fn poisoned_matrix_stops_without_partial_integration() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        poison(&flow_matrix);
        let (on_event, events) = recording_channel();
        let mut worker = PacketWorker::new(
            on_event,
            41,
            LinkType::ETHERNET,
            Arc::clone(&flow_matrix),
            Arc::clone(&graph),
            TEST_DRAIN_STALL_BOUND,
        );
        let drop_counters = Arc::new(AppDropCounters::default());
        let mut emitter = StatsEmitter::new(
            8,
            Arc::new(SharedCaptureStats::default()),
            Arc::clone(&drop_counters),
            41,
        );
        let stop_flag = AtomicBool::new(false);
        let terminal = TerminalState::default();
        let pool = PacketBufferPool::new(2, 65_536);
        let CaptureMessage::Packet(buffer) = packet_message(&pool, &arp_frame());

        let fatal = worker.process_packet(&buffer).unwrap_err();
        pool.put(buffer);
        let buffers_before_queue = pool.len();
        let (tx, rx) = bounded::<CaptureMessage>(1);
        tx.send(packet_message(&pool, &arp_frame())).unwrap();
        drop(tx);

        assert_eq!(fatal, ProcessingFatal::MatrixPoisoned);
        assert_eq!(worker.packets_integrated, 0);
        assert!(worker.packet_batch.is_empty());
        assert!(worker.graph_batch.is_empty());
        assert!(graph.lock().unwrap().nodes.is_empty());
        let poisoned_matrix = match flow_matrix.lock() {
            Ok(_) => panic!("la matrice devait rester empoisonnée"),
            Err(poisoned) => poisoned.into_inner(),
        };
        assert_eq!(poisoned_matrix.row_count(), 0);

        stop_on_processing_fatal(
            &mut worker,
            &mut emitter,
            &rx,
            &pool,
            &stop_flag,
            &terminal,
            fatal,
        );

        assert!(stop_flag.load(Ordering::Acquire));
        assert_eq!(
            pool.len(),
            buffers_before_queue,
            "les buffers abandonnés sont tous rendus"
        );
        assert_eq!(
            drop_counters.total(),
            2,
            "paquet déclencheur et paquet en file sont comptés perdus"
        );
        worker
            .on_event
            .send(CaptureEvent::Stopped {
                session_id: worker.session_id,
                reason: terminal.preferred_reason().unwrap(),
            })
            .unwrap();
        let events = events.lock().unwrap();
        let stopped = events
            .iter()
            .find(|event| event["event"] == "stopped")
            .expect("un événement Stopped fatal est observable");
        let final_stats = events
            .iter()
            .find(|event| event["event"] == "stats")
            .expect("le récapitulatif final fatal est observable");
        assert_eq!(final_stats["data"]["appDropped"], 2);
        assert_eq!(stopped["data"]["sessionId"], 41);
        assert!(
            stopped["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("matrice")
        );
    }

    #[test]
    fn poisoned_graph_leaves_matrix_unchanged_and_stops() {
        let flow_matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        poison(&graph);
        let (on_event, events) = recording_channel();
        let mut worker = PacketWorker::new(
            on_event,
            42,
            LinkType::ETHERNET,
            Arc::clone(&flow_matrix),
            Arc::clone(&graph),
            TEST_DRAIN_STALL_BOUND,
        );
        let drop_counters = Arc::new(AppDropCounters::default());
        let mut emitter = StatsEmitter::new(
            8,
            Arc::new(SharedCaptureStats::default()),
            Arc::clone(&drop_counters),
            42,
        );
        let stop_flag = AtomicBool::new(false);
        let terminal = TerminalState::default();
        let pool = PacketBufferPool::new(2, 65_536);
        let CaptureMessage::Packet(buffer) = packet_message(&pool, &arp_frame());

        let fatal = worker.process_packet(&buffer).unwrap_err();
        pool.put(buffer);
        let (tx, rx) = bounded::<CaptureMessage>(1);
        drop(tx);

        assert_eq!(fatal, ProcessingFatal::GraphPoisoned);
        assert_eq!(
            flow_matrix.lock().unwrap().row_count(),
            0,
            "les deux verrous sont acquis avant la première mutation"
        );
        assert_eq!(worker.packets_integrated, 0);
        assert!(worker.packet_batch.is_empty());
        assert!(worker.graph_batch.is_empty());
        let poisoned_graph = graph.lock().unwrap_err().into_inner();
        assert!(poisoned_graph.nodes.is_empty());

        stop_on_processing_fatal(
            &mut worker,
            &mut emitter,
            &rx,
            &pool,
            &stop_flag,
            &terminal,
            fatal,
        );

        assert!(stop_flag.load(Ordering::Acquire));
        assert_eq!(drop_counters.total(), 1);
        worker
            .on_event
            .send(CaptureEvent::Stopped {
                session_id: worker.session_id,
                reason: terminal.preferred_reason().unwrap(),
            })
            .unwrap();
        let events = events.lock().unwrap();
        let stopped = events
            .iter()
            .find(|event| event["event"] == "stopped")
            .expect("un événement Stopped fatal est observable");
        assert_eq!(stopped["data"]["sessionId"], 42);
        assert!(
            stopped["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("graphe")
        );
    }

    #[test]
    fn fatal_summary_waits_for_the_producers_last_stats() {
        let (on_event, events) = recording_channel();
        let mut worker = PacketWorker::new(
            on_event,
            43,
            LinkType::ETHERNET,
            Arc::new(Mutex::new(FlowMatrix::new())),
            Arc::new(Mutex::new(GraphData::new())),
            TEST_DRAIN_STALL_BOUND,
        );
        let shared_stats = Arc::new(SharedCaptureStats::default());
        let mut emitter = StatsEmitter::new(
            1,
            Arc::clone(&shared_stats),
            Arc::new(AppDropCounters::default()),
            43,
        );
        let pool = Arc::new(PacketBufferPool::new(1, 65_536));
        let (tx, rx) = bounded::<CaptureMessage>(1);
        let stats_from_capture = Arc::clone(&shared_stats);
        let producer = thread::spawn(move || {
            thread::sleep(DRAIN_RECV_TIMEOUT + Duration::from_millis(25));
            stats_from_capture.store(pcap::Stat {
                received: 17,
                dropped: 2,
                if_dropped: 1,
            });
            drop(tx);
        });

        stop_on_processing_fatal(
            &mut worker,
            &mut emitter,
            &rx,
            &pool,
            &AtomicBool::new(false),
            &TerminalState::default(),
            ProcessingFatal::MatrixPoisoned,
        );
        producer.join().unwrap();

        let events = events.lock().unwrap();
        let final_stats = events
            .iter()
            .find(|event| event["event"] == "stats")
            .expect("les stats finales sont émises après la déconnexion");
        assert_eq!(final_stats["data"]["received"], 17);
        assert_eq!(final_stats["data"]["dropped"], 2);
        assert_eq!(final_stats["data"]["ifDropped"], 1);
        assert_eq!(
            final_stats["data"]["appDropped"], 1,
            "le paquet ayant révélé le poison reste comptabilisé"
        );
    }
}
