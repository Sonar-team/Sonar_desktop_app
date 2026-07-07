use log::{error, info};
use packet_parser::PacketFlow;
#[cfg(feature = "capture_timing")]
use packet_parser::timing::ParseTiming;
use pcap::Capture;
#[cfg(feature = "capture_timing")]
use serde_json::json;
#[cfg(feature = "capture_timing")]
use std::path::PathBuf;
#[cfg(feature = "capture_timing")]
use std::time::Instant;
use std::{
    collections::HashMap,
    io::ErrorKind,
    net::IpAddr,
    sync::{Arc, Mutex},
};
#[cfg(feature = "capture_timing")]
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{State, ipc::Channel};

// Helpers utilisés uniquement par la détection de conflits historique (tests).
#[cfg(test)]
use crate::state::flow_matrix::{is_non_unicast_mac, is_placeholder_ip};

use crate::{
    errors::{CaptureStateError, import::PcapImportError, label::LabelError},
    events::CaptureEvent,
    setup::labels::{clean_csv_field, parse_label_fields},
    state::{
        capture::{CaptureState, capture_handle::messages::capture::PacketMinimal},
        flow_matrix::{FlowMatrix, FlowMatrixRow, FlowStats},
        graph::GraphData,
        labels_list::LabelStore,
    },
};

/// Un `Channel` Tauri est lié à une seule commande ; pendant une capture live
/// les événements doivent passer par le channel de la capture pour arriver au
/// front. Retourne ce dernier s'il existe, sinon celui passé à la commande.
fn event_channel(
    capture_state: &State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Channel<CaptureEvent<'static>> {
    let live = capture_state.lock().unwrap().on_event.clone();
    live.unwrap_or(on_event)
}

#[cfg(feature = "capture_timing")]
#[derive(Clone, Copy)]
struct ImportTimingSample {
    seq: u64,
    sample_rate: u64,
}

#[cfg(feature = "capture_timing")]
struct ImportTimingLogger {
    writer: BufWriter<File>,
    sample_rate: u64,
    packet_seen: u64,
    pending_flush: u64,
    last_flush: Instant,
}

#[cfg(not(feature = "capture_timing"))]
type ImportTimingLogger = ();

#[cfg(feature = "capture_timing")]
impl ImportTimingLogger {
    fn new() -> io::Result<Self> {
        let path = import_timing_log_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let sample_rate = std::env::var("SONAR_IMPORT_TIMING_SAMPLE_RATE")
            .or_else(|_| std::env::var("SONAR_CAPTURE_TIMING_SAMPLE_RATE"))
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(1);

        info!(
            "Import timing log enabled: path={} sample_rate={}",
            path.display(),
            sample_rate
        );

        Ok(Self {
            writer: BufWriter::new(file),
            sample_rate,
            packet_seen: 0,
            pending_flush: 0,
            last_flush: Instant::now(),
        })
    }

    fn next_sample(&mut self) -> Option<ImportTimingSample> {
        self.packet_seen = self.packet_seen.saturating_add(1);
        if self.packet_seen % self.sample_rate != 0 {
            return None;
        }

        Some(ImportTimingSample {
            seq: self.packet_seen,
            sample_rate: self.sample_rate,
        })
    }

    fn write_value(&mut self, value: serde_json::Value) -> io::Result<()> {
        serde_json::to_writer(&mut self.writer, &value).map_err(io::Error::other)?;
        self.writer.write_all(b"\n")?;

        self.pending_flush = self.pending_flush.saturating_add(1);
        if self.pending_flush >= 256
            || self.last_flush.elapsed() >= std::time::Duration::from_secs(1)
        {
            self.writer.flush()?;
            self.pending_flush = 0;
            self.last_flush = Instant::now();
        }

        Ok(())
    }
}

#[cfg(feature = "capture_timing")]
fn import_timing_log_path() -> PathBuf {
    if let Ok(path) = std::env::var("SONAR_CAPTURE_TIMING_LOG") {
        return PathBuf::from(path);
    }

    let file_name = format!("capture-timing-{}.jsonl", std::process::id());

    #[cfg(target_os = "linux")]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/"))
                    .join(".local/share")
            });
        return base.join("fr.sonar.app/logs").join(file_name);
    }

    #[cfg(target_os = "windows")]
    {
        return dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
            .join("fr.sonar.app\\logs")
            .join(file_name);
    }

    #[cfg(target_os = "macos")]
    {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/Users/Shared"))
            .join("Library/Logs/fr.sonar.app")
            .join(file_name);
    }
}

#[cfg(feature = "capture_timing")]
fn elapsed_ns_since(start: Instant) -> u64 {
    start.elapsed().as_nanos() as u64
}

#[cfg(feature = "capture_timing")]
fn now_unix_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

// La détection de conflits « historique » (basée fichier) ne sert plus qu'aux
// tests : en production, `dedup_labels_first_wins` déduplique et enregistre les
// conflits pendant l'import.
#[cfg(test)]
type ConflictsList = Vec<(usize, usize, String, String, String, String, String)>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelRow {
    line: usize,
    raw: String,
    mac: String,
    ip: String,
    label: String,
}

impl LabelRow {
    fn into_tuple(self) -> (String, String, String) {
        (self.mac, self.ip, self.label)
    }
}

/// Un choix de label en compétition sur une même clé : (n° de ligne, label, ligne brute).
pub type LabelChoice = (usize, String, String);

/// Conflit de labels résolu automatiquement à l'import : une même clé
/// `(mac, ip)` portait plusieurs labels différents. Le premier est retenu
/// (« premier gagné »), les suivants sont écartés mais conservés ici pour que
/// l'utilisateur puisse arbitrer via le module de gestion des labels.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabelConflictReport {
    pub mac: String,
    pub ip: String,
    pub kept: LabelChoice,
    pub dropped: Vec<LabelChoice>,
}

/// Résultat d'un import de labels : nombre de labels appliqués et conflits
/// résolus (non bloquants).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabelImportReport {
    pub applied: usize,
    pub conflicts: Vec<LabelConflictReport>,
}

/// État partagé : conflits du dernier import de labels, à arbitrer.
#[derive(Default)]
pub struct LabelConflictStore {
    pub conflicts: Vec<LabelConflictReport>,
}

/// Déduplique les lignes par clé `(mac, ip)` en gardant la **première**
/// occurrence. Retourne les lignes retenues (une par clé) et la liste des
/// conflits (clés ayant reçu au moins deux labels différents).
fn dedup_labels_first_wins(rows: Vec<LabelRow>) -> (Vec<LabelRow>, Vec<LabelConflictReport>) {
    let mut order: Vec<(String, String)> = Vec::new();
    let mut kept: HashMap<(String, String), LabelRow> = HashMap::new();
    let mut dropped: HashMap<(String, String), Vec<LabelChoice>> = HashMap::new();

    for row in rows {
        let key = (row.mac.clone(), row.ip.clone());
        match kept.get(&key) {
            None => {
                order.push(key.clone());
                kept.insert(key, row);
            }
            Some(first) => {
                if first.label != row.label {
                    dropped.entry(key).or_default().push((
                        row.line,
                        row.label.clone(),
                        row.raw.clone(),
                    ));
                }
                // Doublon (même clé) : ignoré, le premier est conservé.
            }
        }
    }

    let mut conflicts = Vec::new();
    let mut kept_rows = Vec::new();
    for key in order {
        let row = kept.remove(&key).expect("clé retenue présente");
        if let Some(dropped_choices) = dropped.remove(&key) {
            conflicts.push(LabelConflictReport {
                mac: row.mac.clone(),
                ip: row.ip.clone(),
                kept: (row.line, row.label.clone(), row.raw.clone()),
                dropped: dropped_choices,
            });
        }
        kept_rows.push(row);
    }

    (kept_rows, conflicts)
}

fn read_label_file(csv_path: &str) -> Result<String, CaptureStateError> {
    match std::fs::read_to_string(csv_path) {
        Ok(csv_data) => Ok(csv_data),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

fn count_packets_in_pcap(file_path: &str) -> Result<usize, CaptureStateError> {
    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;

    let mut count: usize = 0;
    while cap.next_packet().is_ok() {
        count += 1;
    }
    Ok(count)
}

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcap_paths: Vec<String>,
    on_event: Channel<CaptureEvent<'_>>,
) -> Result<(), CaptureStateError> {
    let mut timing_logger: Option<ImportTimingLogger> = {
        #[cfg(feature = "capture_timing")]
        {
            match ImportTimingLogger::new() {
                Ok(logger) => Some(logger),
                Err(e) => {
                    error!("Import timing log disabled: {}", e);
                    None
                }
            }
        }
        #[cfg(not(feature = "capture_timing"))]
        {
            None
        }
    };
    #[cfg(feature = "capture_timing")]
    let command_start = Instant::now();

    info!(
        "[convert_from_pcap_list] COMMAND CALLED avec pcap_paths = {:?}",
        pcap_paths
    );

    // started
    if let Err(e) = on_event.send(CaptureEvent::Started {
        device: "",
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
        snaplen: 65536,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    };

    let mut matrice_guard = matrice.lock().unwrap();
    let mut graph_guard = graph.lock().unwrap();
    #[cfg(feature = "capture_timing")]
    let reset_start = Instant::now();
    matrice_guard.clear();
    graph_guard.clear();
    #[cfg(feature = "capture_timing")]
    let reset_ns = elapsed_ns_since(reset_start);

    labels_to_matrix(label_store, &mut matrice_guard)?;

    info!("[convert_from_pcap_list] Matrice & GraphData reset");

    for pcap_path in &pcap_paths {
        info!("[convert_from_pcap_list] Traitement de {}", pcap_path);
        handle_pcap_file(
            pcap_path,
            &mut matrice_guard,
            &mut graph_guard,
            &on_event,
            &mut timing_logger,
        )?;
    }

    info!("[convert_from_pcap_list] FIN traitement liste PCAP");

    // 🔥 snapshot complet envoyé sur le channel
    #[cfg(feature = "capture_timing")]
    let snapshot_build_start = Instant::now();
    let snapshot: GraphData = graph_guard.get_all_graph_data(); // doit renvoyer un GraphData possédé
    #[cfg(feature = "capture_timing")]
    let snapshot_build_ns = elapsed_ns_since(snapshot_build_start);

    #[cfg(feature = "capture_timing")]
    let snapshot_ipc_start = Instant::now();
    let snapshot_send_result = on_event.send(CaptureEvent::GraphSnapshot {
        graph_data: &snapshot,
    });
    if let Err(e) = &snapshot_send_result {
        error!("Erreur lors de l'envoi de GraphSnapshot: {:?}", e);
    }
    #[cfg(feature = "capture_timing")]
    let snapshot_send_ok = snapshot_send_result.is_ok();
    #[cfg(feature = "capture_timing")]
    {
        let snapshot_ipc_ns = elapsed_ns_since(snapshot_ipc_start);
        if let Some(logger) = timing_logger.as_mut() {
            if let Err(e) = logger.write_value(json!({
                "event": "import_snapshot_timing",
                "ts_unix_ns": now_unix_ns(),
                "files": pcap_paths.len(),
                "matrix_count": matrice_guard.matrix.len(),
                "graph_nodes": snapshot.nodes.len(),
                "graph_edges": snapshot.edges.len(),
                "snapshot_build_ns": snapshot_build_ns,
                "snapshot_ipc_ns": snapshot_ipc_ns,
                "snapshot_ipc_ok": snapshot_send_ok,
                "reset_ns": reset_ns,
                "command_total_ns": elapsed_ns_since(command_start)
            })) {
                error!(
                    "Import timing log disabled after snapshot write error: {}",
                    e
                );
            }
        }
    }

    Ok(())
}

fn handle_pcap_file(
    file_path: &str,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent<'_>>,
    timing_logger: &mut Option<ImportTimingLogger>,
) -> Result<(), CaptureStateError> {
    #[cfg(not(feature = "capture_timing"))]
    let _ = timing_logger;
    #[cfg(feature = "capture_timing")]
    let file_start = Instant::now();
    #[cfg(feature = "capture_timing")]
    let count_start = Instant::now();
    let total = count_packets_in_pcap(file_path)?;
    #[cfg(feature = "capture_timing")]
    let count_packets_ns = elapsed_ns_since(count_start);

    info!(
        "[handle_pcap_file] {} : {} paquets détectés",
        file_path, total
    );

    #[cfg(feature = "capture_timing")]
    let open_start = Instant::now();
    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;
    #[cfg(feature = "capture_timing")]
    let open_ns = elapsed_ns_since(open_start);

    let mut packet_count: usize = 0;
    #[cfg(feature = "capture_timing")]
    let process_start = Instant::now();
    #[cfg(feature = "capture_timing")]
    let mut parse_ok_count: usize = 0;
    #[cfg(feature = "capture_timing")]
    let mut parse_error_count: usize = 0;

    while let Ok(packet) = cap.next_packet() {
        packet_count += 1;

        #[cfg(feature = "capture_timing")]
        let timing_sample = timing_logger
            .as_mut()
            .and_then(ImportTimingLogger::next_sample);
        #[cfg(feature = "capture_timing")]
        let pipeline_start = timing_sample.map(|_| Instant::now());

        #[cfg(feature = "capture_timing")]
        let parsed_flow = if timing_sample.is_some() {
            let mut parse_timing = ParseTiming::default();
            PacketFlow::try_from_timed(packet.data, &mut parse_timing)
                .map(|flow| (flow, parse_timing))
                .map_err(|error| (error, parse_timing))
        } else {
            PacketFlow::try_from(packet.data)
                .map(|flow| (flow, ParseTiming::default()))
                .map_err(|error| (error, ParseTiming::default()))
        };

        #[cfg(not(feature = "capture_timing"))]
        let parsed_flow = PacketFlow::try_from(packet.data);

        #[cfg(feature = "capture_timing")]
        match parsed_flow {
            Ok((flow, parse_timing)) => {
                parse_ok_count += 1;
                let packet_min = PacketMinimal {
                    ts_sec: packet.header.ts.tv_sec,
                    ts_usec: packet.header.ts.tv_usec,
                    caplen: packet.header.caplen,
                    len: packet.header.len,
                    flow,
                };

                let packet_owned_start = timing_sample.map(|_| Instant::now());
                let owned_packet = packet_min.to_owned_packet();
                let packet_owned_ns = packet_owned_start.map(elapsed_ns_since).unwrap_or(0);

                let matrix_update_start = timing_sample.map(|_| Instant::now());
                matrice.update_flow(&owned_packet);
                let matrix_count = matrice.matrix.len();
                let matrix_update_ns = matrix_update_start.map(elapsed_ns_since).unwrap_or(0);

                let label_lookup_start = timing_sample.map(|_| Instant::now());
                let source_ip = owned_packet
                    .flow
                    .internet
                    .as_ref()
                    .and_then(|i| i.source_ip)
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let destination_ip = owned_packet
                    .flow
                    .internet
                    .as_ref()
                    .and_then(|i| i.destination_ip)
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let source_label =
                    matrice.get_label(&owned_packet.flow.data_link.source_mac, &source_ip);
                let destination_label = matrice.get_label(
                    &owned_packet.flow.data_link.destination_mac,
                    &destination_ip,
                );
                let label_lookup_ns = label_lookup_start.map(elapsed_ns_since).unwrap_or(0);

                let graph_update_start = timing_sample.map(|_| Instant::now());
                let graph_updates = graph.add_packet_flow(
                    &owned_packet.flow,
                    source_label,
                    destination_label,
                    1,
                    owned_packet.len as u64,
                );
                let graph_update_ns = graph_update_start.map(elapsed_ns_since).unwrap_or(0);

                let mut stats_ipc_ns = 0;
                let mut stats_ipc_sent = false;
                let mut stats_ipc_ok = false;
                if packet_count.is_multiple_of(1000) || packet_count == total {
                    stats_ipc_sent = true;
                    let stats_ipc_start = timing_sample.map(|_| Instant::now());
                    stats_ipc_ok = on_event
                        .send(CaptureEvent::Stats {
                            received: packet_count as u32,
                            dropped: 0,
                            if_dropped: 0,
                            app_dropped: 0,
                            processed: matrix_count as u32,
                        })
                        .map_err(|e| {
                            error!("Erreur lors de l'envoi de Stats: {:?}", e);
                            e
                        })
                        .is_ok();
                    stats_ipc_ns = stats_ipc_start.map(elapsed_ns_since).unwrap_or(0);
                }

                if let (Some(sample), Some(start), Some(logger)) =
                    (timing_sample, pipeline_start, timing_logger.as_mut())
                {
                    if let Err(e) = logger.write_value(json!({
                        "event": "import_packet_timing",
                        "ts_unix_ns": now_unix_ns(),
                        "file_path": file_path,
                        "seq": sample.seq,
                        "packet_index": packet_count,
                        "total_packets": total,
                        "sample_rate": sample.sample_rate,
                        "caplen": packet.header.caplen,
                        "len": packet.header.len,
                        "parse_l2_ns": parse_timing.l2_ns,
                        "parse_l3_ns": parse_timing.l3_ns,
                        "parse_l4_ns": parse_timing.l4_ns,
                        "parse_l7_ns": parse_timing.l7_ns,
                        "parse_total_ns": parse_timing.total_ns,
                        "packet_owned_ns": packet_owned_ns,
                        "matrix_update_ns": matrix_update_ns,
                        "label_lookup_ns": label_lookup_ns,
                        "graph_update_ns": graph_update_ns,
                        "graph_updates": graph_updates.len(),
                        "stats_ipc_ns": stats_ipc_ns,
                        "stats_ipc_sent": stats_ipc_sent,
                        "stats_ipc_ok": stats_ipc_ok,
                        "matrix_count": matrix_count,
                        "pipeline_total_ns": elapsed_ns_since(start)
                    })) {
                        error!("Import timing log disabled after packet write error: {}", e);
                        *timing_logger = None;
                    }
                }
            }
            Err((parse_error, parse_timing)) => {
                parse_error_count += 1;

                if let (Some(sample), Some(start), Some(logger)) =
                    (timing_sample, pipeline_start, timing_logger.as_mut())
                {
                    if let Err(e) = logger.write_value(json!({
                        "event": "import_parse_error_timing",
                        "ts_unix_ns": now_unix_ns(),
                        "file_path": file_path,
                        "seq": sample.seq,
                        "packet_index": packet_count,
                        "total_packets": total,
                        "sample_rate": sample.sample_rate,
                        "caplen": packet.header.caplen,
                        "len": packet.header.len,
                        "error": parse_error.to_string(),
                        "parse_l2_ns": parse_timing.l2_ns,
                        "parse_l3_ns": parse_timing.l3_ns,
                        "parse_l4_ns": parse_timing.l4_ns,
                        "parse_l7_ns": parse_timing.l7_ns,
                        "parse_total_ns": parse_timing.total_ns,
                        "pipeline_total_ns": elapsed_ns_since(start)
                    })) {
                        error!(
                            "Import timing log disabled after parse error write error: {}",
                            e
                        );
                        *timing_logger = None;
                    }
                }
            }
        }

        #[cfg(not(feature = "capture_timing"))]
        if let Ok(flow) = parsed_flow {
            let packet_min = PacketMinimal {
                ts_sec: packet.header.ts.tv_sec,
                ts_usec: packet.header.ts.tv_usec,
                caplen: packet.header.caplen,
                len: packet.header.len,
                flow,
            };

            let owned_packet = packet_min.to_owned_packet();
            matrice.update_flow(&owned_packet);
            let matrix_count = matrice.matrix.len();
            let source_ip = owned_packet
                .flow
                .internet
                .as_ref()
                .and_then(|i| i.source_ip)
                .map(|ip| ip.to_string())
                .unwrap_or_default();
            let destination_ip = owned_packet
                .flow
                .internet
                .as_ref()
                .and_then(|i| i.destination_ip)
                .map(|ip| ip.to_string())
                .unwrap_or_default();
            let source_label =
                matrice.get_label(&owned_packet.flow.data_link.source_mac, &source_ip);
            let destination_label = matrice.get_label(
                &owned_packet.flow.data_link.destination_mac,
                &destination_ip,
            );
            // info!(
            //     "[handle_pcap_file] {} : paquet {}/{} ; lignes matrice = {}",
            //     file_path,
            //     packet_count,
            //     total,
            //     matrix_count
            // );

            graph.add_packet_flow(
                &owned_packet.flow,
                source_label,
                destination_label,
                1,
                owned_packet.len as u64,
            );

            // Stats périodiques (optionnel)
            if (packet_count.is_multiple_of(1000) || packet_count == total)
                && let Err(e) = on_event.send(CaptureEvent::Stats {
                    received: packet_count as u32,
                    dropped: 0,
                    if_dropped: 0,
                    app_dropped: 0,
                    processed: matrix_count as u32,
                })
            {
                error!("Erreur lors de l'envoi de Stats: {:?}", e);
            }

            // si tu veux envoyer des Packet individuellement (live)
            // if let Err(e) = on_event.send(CaptureEvent::Packet { packet: &packet_min }) { ... }
        }
    }

    #[cfg(feature = "capture_timing")]
    let process_ns = elapsed_ns_since(process_start);
    #[cfg(feature = "capture_timing")]
    let finished_ipc_start = Instant::now();
    let finished_send_result = on_event.send(CaptureEvent::Finished {
        file_name: file_path,
        packet_total_count: total,
        matrix_total_count: matrice.matrix.len(),
    });
    if let Err(e) = &finished_send_result {
        error!("Erreur lors de l'envoi de Finished: {:?}", e);
    };
    #[cfg(feature = "capture_timing")]
    let finished_send_ok = finished_send_result.is_ok();
    #[cfg(feature = "capture_timing")]
    {
        let finished_ipc_ns = elapsed_ns_since(finished_ipc_start);
        if let Some(logger) = timing_logger.as_mut() {
            if let Err(e) = logger.write_value(json!({
                "event": "import_file_timing",
                "ts_unix_ns": now_unix_ns(),
                "file_path": file_path,
                "total_packets": total,
                "read_packets": packet_count,
                "parse_ok": parse_ok_count,
                "parse_errors": parse_error_count,
                "matrix_count": matrice.matrix.len(),
                "graph_nodes": graph.nodes.len(),
                "graph_edges": graph.edges.len(),
                "count_packets_ns": count_packets_ns,
                "open_ns": open_ns,
                "process_ns": process_ns,
                "finished_ipc_ns": finished_ipc_ns,
                "finished_ipc_ok": finished_send_ok,
                "file_total_ns": elapsed_ns_since(file_start)
            })) {
                error!("Import timing log disabled after file write error: {}", e);
                *timing_logger = None;
            }
        }
    }

    info!(
        "[handle_pcap_file] Finised with {} paquets lu, {} lignes matrice",
        total,
        matrice.matrix.len()
    );
    Ok(())
}

/*<----- Csv part -----> */

#[tauri::command(async)]
pub fn get_label_rows(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<Vec<(String, String, String)>, CaptureStateError> {
    let label_store = label_store.lock().unwrap();
    Ok(label_store.get().clone())
}

#[tauri::command(async)]
pub fn clear_label_store(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<(), CaptureStateError> {
    let mut labels = label_store.lock().unwrap();
    labels.clear();
    Ok(())
}

fn label_record_line(record: &csv::StringRecord, fallback: usize) -> usize {
    record
        .position()
        .map(|position| position.line() as usize)
        .unwrap_or(fallback)
}

fn label_error_line(error: &csv::Error, fallback: usize) -> usize {
    error
        .position()
        .map(|position| position.line() as usize)
        .unwrap_or(fallback)
}

fn label_record_to_display(record: &csv::StringRecord) -> String {
    record.iter().collect::<Vec<_>>().join(",")
}

fn read_label_records(
    csv_path: &str,
) -> Result<Vec<(usize, csv::StringRecord)>, CaptureStateError> {
    let file = read_label_file(csv_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file.as_bytes());

    let mut records = Vec::new();
    let mut invalid_lines = Vec::new();

    for (index, result) in rdr.records().enumerate() {
        let fallback_line = index + 1;
        match result {
            Ok(record) => {
                let line = label_record_line(&record, fallback_line);
                if record.iter().all(|field| clean_csv_field(field).is_empty()) {
                    continue;
                }
                if record.len() < 3 {
                    invalid_lines.push((line, label_record_to_display(&record)));
                } else {
                    records.push((line, record));
                }
            }
            Err(error) => {
                invalid_lines.push((label_error_line(&error, fallback_line), error.to_string()))
            }
        }
    }

    if invalid_lines.is_empty() {
        Ok(records)
    } else {
        Err(LabelError::InvalidRowsFormat { invalid_lines }.into())
    }
}

fn read_label_rows(csv_path: &str) -> Result<Vec<LabelRow>, CaptureStateError> {
    Ok(read_label_records(csv_path)?
        .into_iter()
        .filter_map(|(line, record)| {
            let raw = label_record_to_display(&record);
            parse_label_fields(record.iter()).map(|(mac, ip, label)| LabelRow {
                line,
                raw,
                mac,
                ip,
                label,
            })
        })
        .collect())
}

fn verif_label_rows_format(file: String) -> Result<(), CaptureStateError> {
    read_label_records(&file)?;
    Ok(())
}

#[cfg(test)]
fn verif_labels_conflicts(file_path: String) -> Result<(), CaptureStateError> {
    let rows = read_label_rows(&file_path)?;

    // Le store de labels est indexé par la clé `(mac, ip)`. Deux lignes ne
    // s'écrasent (perte de donnée) que si elles portent **la même clé** avec un
    // label différent. Deux équipements distincts partageant une IP (MAC
    // différentes) cohabitent sans ambiguïté : ce n'est pas un conflit.
    //
    // Les champs non identifiants (IP placeholder `0.0.0.0`/`::`, MAC broadcast
    // ou multicast) sont ignorés : ils peuvent être partagés par plusieurs
    // machines et produisaient de faux conflits, notamment sur des fichiers de
    // labels exportés par SONAR lui-même.
    let mut same_ip_different_label: ConflictsList = Vec::new();
    // Conservé pour compatibilité de l'API d'erreur ; plus alimenté.
    let same_ip_different_mac: ConflictsList = Vec::new();

    let is_identifying =
        |row: &LabelRow| !is_placeholder_ip(&row.ip) && !is_non_unicast_mac(&row.mac);

    let skip = usize::from(rows.first().is_some_and(is_header_row));
    for (i, row1) in rows.iter().enumerate().skip(skip) {
        if !is_identifying(row1) || (row1.mac.is_empty() && row1.ip.is_empty()) {
            continue;
        }
        for row2 in rows[i + 1..].iter() {
            if !is_identifying(row2) {
                continue;
            }
            // Même clé de store (mac, ip) mais label différent -> écrasement
            // silencieux -> conflit réel.
            if row1.mac == row2.mac && row1.ip == row2.ip && row1.label != row2.label {
                same_ip_different_label.push((
                    row1.line,
                    row2.line,
                    row1.ip.clone(),
                    row1.label.clone(),
                    row2.label.clone(),
                    row1.raw.clone(),
                    row2.raw.clone(),
                ))
            }
        }
    }

    if !same_ip_different_label.is_empty() || !same_ip_different_mac.is_empty() {
        Err(LabelError::LabelLinesConflicts {
            same_ip_diff_mac: same_ip_different_mac,
            same_ip_diff_label: same_ip_different_label,
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn verif_mac_ip_format(csv_path: String) -> Result<(), CaptureStateError> {
    let mut invalid_ip: Vec<(usize, String, String)> = Vec::new();
    let mut invalid_mac: Vec<(usize, String, String)> = Vec::new();

    let rows = read_label_rows(&csv_path)?;

    let skip = usize::from(rows.first().is_some_and(is_header_row));
    for row in rows.iter().skip(skip) {
        if !is_ip_address(&row.ip) && !row.ip.is_empty() {
            invalid_ip.push((row.line, row.ip.clone(), row.raw.clone()));
        }
        if !is_mac_address(&row.mac) && !row.mac.is_empty() {
            invalid_mac.push((row.line, row.mac.clone(), row.raw.clone()));
        }
    }

    if !invalid_ip.is_empty() || !invalid_mac.is_empty() {
        return Err(LabelError::InvalidMacIpFormat {
            invalid_mac,
            invalid_ip,
        }
        .into());
    }
    Ok(())
}

fn is_ip_address(value: &str) -> bool {
    value.parse::<IpAddr>().is_ok()
}

fn is_mac_address(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

// Une ligne d'en-tête ("mac,ip,label") ne peut être que la première du fichier :
// on la reconnaît parce que ni son champ MAC ni son champ IP ne sont des adresses valides.
fn is_header_row(row: &LabelRow) -> bool {
    !is_mac_address(&row.mac) && !is_ip_address(&row.ip)
}

#[tauri::command(async)]
pub fn import_label_file(
    incoming_file_path: String,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<LabelImportReport, CaptureStateError> {
    let on_event = event_channel(&capture_state, on_event);

    // Les erreurs de FORMAT restent bloquantes ; les conflits de labels, eux,
    // ne bloquent plus : on garde le premier label par clé (mac, ip) et on
    // enregistre les doublons écartés pour arbitrage.
    verif_label_rows_format(incoming_file_path.clone())?;
    verif_mac_ip_format(incoming_file_path.clone())?;

    let conflicts = {
        let mut label_store = label_store.lock().unwrap();
        label_store.clear();

        let mut rows = read_label_rows(&incoming_file_path)?;

        // L'en-tête éventuel est écarté ici pour que le store ne contienne que des données.
        if rows.first().is_some_and(is_header_row) {
            rows.remove(0);
        }

        let (rows, conflicts) = dedup_labels_first_wins(rows);

        for row in rows {
            label_store.add(row.into_tuple())
        }

        conflicts
    };

    let applied = {
        let mut state_label = state_label.lock().unwrap();
        labels_to_matrix(label_store, &mut state_label)?;

        // Réapplique les labels sur les nœuds du graphe (même résolution que la
        // capture) et notifie le front nœud par nœud : pas de snapshot complet,
        // pour préserver la disposition du graphe côté UI.
        let updates = {
            let mut graph_guard = graph.lock().unwrap();
            graph_guard.refresh_labels(|mac, ip| state_label.get_label(mac, ip))
        };

        info!(
            "[import_label_file] {} label(s) de nœud mis à jour dans le graphe, {} conflit(s) résolu(s)",
            updates.len(),
            conflicts.len()
        );

        for update in &updates {
            if let Err(e) = on_event.send(CaptureEvent::Graph { update }) {
                error!("Erreur lors de l'envoi du GraphUpdate label: {:?}", e);
            }
        }

        updates.len()
    };

    // Mémorise les conflits pour le module d'arbitrage.
    conflict_store.lock().unwrap().conflicts = conflicts.clone();

    Ok(LabelImportReport { applied, conflicts })
}

/// Conflits de labels résolus lors du dernier import (pour le module
/// d'arbitrage / la vue de gestion des labels).
#[tauri::command]
pub fn get_label_conflicts(
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
) -> Result<Vec<LabelConflictReport>, CaptureStateError> {
    Ok(conflict_store.lock().unwrap().conflicts.clone())
}

/// Arbitrage d'un conflit : applique le `label` choisi à la clé `(mac, ip)`
/// dans la matrice, le store et le graphe, retire le conflit de la liste et
/// renvoie les conflits restants.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn resolve_label_conflict(
    mac: String,
    ip: String,
    label: String,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
) -> Result<Vec<LabelConflictReport>, CaptureStateError> {
    // Applique le label choisi à la matrice (résolution des flux) et au store
    // (table + réexport).
    matrix
        .lock()
        .unwrap()
        .add_label(mac.clone(), ip.clone(), label.clone());
    label_store.lock().unwrap().set(&mac, &ip, &label);

    // Rafraîchit le nœud du graphe et notifie l'UI, sans reconstruire le graphe.
    let graph_update = graph
        .lock()
        .unwrap()
        .update_node_label(&mac, &ip, label.clone());
    let on_event = capture_state.lock().unwrap().on_event.clone();
    if let (Some(update), Some(on_event)) = (graph_update, on_event)
        && let Err(e) = on_event.send(CaptureEvent::Graph { update: &update })
    {
        error!("Erreur d'envoi du GraphUpdate arbitrage: {e}");
    }

    // Retire le conflit résolu, renvoie ceux qui restent.
    let mut store = conflict_store.lock().unwrap();
    store.conflicts.retain(|c| !(c.mac == mac && c.ip == ip));
    Ok(store.conflicts.clone())
}

pub fn labels_to_matrix(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    let label_store = label_store.lock().unwrap();
    copy_labels_to_matrix(&label_store, matrice)
}

pub fn copy_labels_to_matrix(
    label_store: &LabelStore,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    for (mac, ip, label) in label_store.get() {
        matrice.add_label(mac.to_string(), ip.to_string(), label.to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn is_matrix_empty(state: tauri::State<'_, Arc<Mutex<FlowMatrix>>>) -> bool {
    state.lock().unwrap().matrix.is_empty()
}

/*<----- Import matrice -----> */

/// Lit un CSV de matrice de flux (format de `FlowMatrix::export_to_csv`).
/// Le fichier est entièrement validé avant de toucher à l'état.
fn read_matrix_rows(csv_path: &str) -> Result<Vec<FlowMatrixRow>, CaptureStateError> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(csv_path)
        .map_err(|e| std::io::Error::other(format!("Ouverture de {csv_path}: {e}")))?;

    let mut rows = Vec::new();
    for (i, result) in rdr.deserialize::<FlowMatrixRow>().enumerate() {
        let row =
            result.map_err(|e| std::io::Error::other(format!("Ligne {} invalide: {e}", i + 2)))?;
        rows.push(row);
    }
    Ok(rows)
}

fn read_matrix_rows_from_files(
    incoming_file_paths: &[String],
) -> Result<Vec<FlowMatrixRow>, CaptureStateError> {
    if incoming_file_paths.is_empty() {
        return Err(std::io::Error::other("Aucun fichier de matrice sélectionné").into());
    }

    let mut rows = Vec::new();
    for path in incoming_file_paths {
        rows.extend(read_matrix_rows(path)?);
    }
    Ok(rows)
}

fn rebuild_matrix_and_graph_from_rows(
    rows: &[FlowMatrixRow],
    label_store: &LabelStore,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
) -> Result<(), CaptureStateError> {
    matrice.clear();
    graph.clear();

    // Labels du store courant, puis ceux portés par les fichiers (prioritaires
    // à clé égale puisque insérés après).
    copy_labels_to_matrix(label_store, matrice)?;
    for row in rows {
        if let Some(label) = row.label_source.as_ref().filter(|l| !l.is_empty()) {
            matrice.add_label(row.mac_source.clone(), row.ip_source.clone(), label.clone());
        }
        if let Some(label) = row.label_destination.as_ref().filter(|l| !l.is_empty()) {
            matrice.add_label(
                row.mac_destination.clone(),
                row.ip_destination.clone(),
                label.clone(),
            );
        }
    }

    for row in rows {
        let (flow, stats) = row.to_flow_and_stats();

        // Les doublons éventuels des fichiers sont fusionnés.
        let entry = matrice.matrix.entry(flow.clone()).or_insert(FlowStats {
            count: 0,
            total_bytes: 0,
            last_seen: stats.last_seen,
        });
        entry.count += stats.count;
        entry.total_bytes = entry.total_bytes.saturating_add(stats.total_bytes);
        if stats.last_seen > entry.last_seen {
            entry.last_seen = stats.last_seen;
        }

        let source_label = matrice.get_label(&flow.data_link.source_mac, &row.ip_source);
        let destination_label =
            matrice.get_label(&flow.data_link.destination_mac, &row.ip_destination);
        graph.add_packet_flow(
            &flow,
            source_label,
            destination_label,
            stats.count,
            stats.total_bytes,
        );
    }

    Ok(())
}

#[tauri::command(async)]
pub fn import_matrix_file(
    incoming_file_path: String,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<(), CaptureStateError> {
    import_matrix_files(
        vec![incoming_file_path],
        matrice,
        graph,
        label_store,
        capture_state,
        on_event,
    )
}

#[tauri::command(async)]
pub fn import_matrix_files(
    incoming_file_paths: Vec<String>,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<(), CaptureStateError> {
    let on_event = event_channel(&capture_state, on_event);

    info!(
        "[import_matrix_files] COMMAND CALLED avec {} fichier(s): {:?}",
        incoming_file_paths.len(),
        incoming_file_paths
    );

    // Un fichier invalide ne doit pas effacer la matrice courante.
    let rows = read_matrix_rows_from_files(&incoming_file_paths)?;

    // Même ordre de verrouillage que convert_from_pcap_list et net_capture
    // (matrice -> graph -> label_store) pour éviter un interblocage ABBA.
    let mut matrice_guard = matrice.lock().unwrap();
    let mut graph_guard = graph.lock().unwrap();
    let label_store_guard = label_store.lock().unwrap();
    rebuild_matrix_and_graph_from_rows(
        &rows,
        &label_store_guard,
        &mut matrice_guard,
        &mut graph_guard,
    )?;

    info!(
        "[import_matrix_files] {} fichier(s), {} ligne(s) importée(s) -> {} flux fusionné(s), {} nœuds, {} arêtes",
        incoming_file_paths.len(),
        rows.len(),
        matrice_guard.matrix.len(),
        graph_guard.nodes.len(),
        graph_guard.edges.len()
    );

    let snapshot = graph_guard.get_all_graph_data();
    if let Err(e) = on_event.send(CaptureEvent::GraphSnapshot {
        graph_data: &snapshot,
    }) {
        error!("Erreur lors de l'envoi de GraphSnapshot: {:?}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(name);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn valid_csv_format_returns_ok() {
        let dir = TempDir::new("sonar_test_valid_csv_format");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn missing_column_returns_invalid_file_format_error() {
        let dir = TempDir::new("sonar_test_missing_column");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "192.168.1.1,mon-pc\n").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::InvalidRowsFormat { invalid_lines }) => {
                assert_eq!(invalid_lines, vec![(1, "192.168.1.1,mon-pc".to_string())]);
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn extra_column_is_merged_into_label() {
        let dir = TempDir::new("sonar_test_extra_column");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc,réseau-2\n",
        )
        .unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());
        assert!(result.is_ok());

        let rows = read_label_rows(file_path.to_str().unwrap()).unwrap();
        assert_eq!(
            rows.into_iter()
                .map(LabelRow::into_tuple)
                .collect::<Vec<_>>(),
            vec![(
                "aa:bb:cc:dd:ee:ff".to_string(),
                "192.168.1.1".to_string(),
                "mon-pc réseau-2".to_string()
            )]
        );
    }

    #[test]
    fn empty_file_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn valid_mac_and_ip_returns_ok() {
        let dir = TempDir::new("sonar_test_valid_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn malformed_ip_returns_error() {
        let dir = TempDir::new("sonar_test_malformed_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "mac,ip,label\naa:bb:cc:dd:ee:ff,192.168.11,mon-pc\n",
        )
        .unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::InvalidMacIpFormat {
                invalid_mac,
                invalid_ip,
            }) => {
                assert!(invalid_mac.is_empty());
                assert_eq!(
                    invalid_ip,
                    vec![(
                        2,
                        "192.168.11".to_string(),
                        "aa:bb:cc:dd:ee:ff,192.168.11,mon-pc".to_string()
                    )]
                );
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn malformed_mac_returns_error() {
        let dir = TempDir::new("sonar_test_malformed_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:e:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_err());
    }

    #[test]
    fn empty_mac_and_ip_are_accepted() {
        let dir = TempDir::new("sonar_test_empty_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,,mon-pc\n,192.168.1.1,mon-pc\n,,mon-pc\n",
        )
        .unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn empty_file_mac_ip_format_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn empty_file_conflicts_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file_conflicts");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn no_conflict_returns_ok() {
        let dir = TempDir::new("sonar_test_no_conflict");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:1f,192.168.1.2,ma-tablette\naa:bb:cc:dd:ee:ff,192.168.1.3,mon-tel\n").unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok())
    }

    #[test]
    fn same_ip_different_mac_is_not_a_conflict() {
        // Deux équipements distincts (MAC différentes) partageant une IP :
        // clés (mac, ip) distinctes, cohabitation dans le store -> pas un conflit.
        let dir = TempDir::new("sonar_test_same_ip_diff_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,poste-A\naa:bb:cc:dd:ee:1f,192.168.1.1,poste-B\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn same_mac_ip_key_different_label_returns_conflict_error() {
        // Même clé (mac, ip) avec deux labels -> écrasement silencieux -> conflit.
        let dir = TempDir::new("sonar_test_same_key_diff_label");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:ff,192.168.1.1,ma-tablette\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::LabelLinesConflicts {
                same_ip_diff_mac,
                same_ip_diff_label,
            }) => {
                assert!(same_ip_diff_mac.is_empty());
                assert_eq!(same_ip_diff_label.len(), 1);
                let (_, _, ip, l1, l2, _, _) = &same_ip_diff_label[0];
                assert_eq!(ip, "192.168.1.1");
                assert_eq!(l1, "mon-pc");
                assert_eq!(l2, "ma-tablette");
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn placeholder_ip_does_not_trigger_conflict() {
        // Plusieurs équipements avec 0.0.0.0 / :: : IP non identifiante -> pas de conflit.
        let dir = TempDir::new("sonar_test_placeholder_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "48:21:0b:41:45:65,0.0.0.0,TVD09\nfc:3f:db:37:5e:0f,0.0.0.0,SIE\n48:21:0b:41:45:65,::,TVD09\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn broadcast_mac_does_not_trigger_conflict() {
        // Une ligne avec MAC broadcast sur une IP réelle ne doit pas entrer en
        // conflit avec la MAC réelle du même équipement.
        let dir = TempDir::new("sonar_test_broadcast_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "48:9e:bd:45:40:92,100.180.65.40,SIC21-CSD\nff:ff:ff:ff:ff:ff,100.180.65.40,SIC21-CSD\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    fn label_row(line: usize, mac: &str, ip: &str, label: &str) -> LabelRow {
        LabelRow {
            line,
            raw: format!("{mac},{ip},{label}"),
            mac: mac.to_string(),
            ip: ip.to_string(),
            label: label.to_string(),
        }
    }

    #[test]
    fn dedup_first_wins_keeps_first_and_records_conflict() {
        let rows = vec![
            label_row(1, "48:9e:bd:31:62:32", "100.180.65.57", "SIE - PC TELEC"),
            label_row(2, "48:9e:bd:31:62:32", "100.180.65.57", "TVD09"),
            label_row(3, "aa:bb:cc:dd:ee:ff", "192.168.1.1", "autre"),
        ];

        let (kept, conflicts) = dedup_labels_first_wins(rows);

        // Une seule ligne conservée pour la clé en doublon (la première).
        assert_eq!(kept.len(), 2);
        let telec = kept.iter().find(|r| r.mac == "48:9e:bd:31:62:32").unwrap();
        assert_eq!(telec.label, "SIE - PC TELEC", "le premier label est gardé");

        // Le conflit est enregistré avec le label écarté.
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].ip, "100.180.65.57");
        assert_eq!(conflicts[0].kept.1, "SIE - PC TELEC");
        assert_eq!(conflicts[0].dropped.len(), 1);
        assert_eq!(conflicts[0].dropped[0].1, "TVD09");
    }

    #[test]
    fn dedup_first_wins_no_conflict_when_same_label_repeated() {
        let rows = vec![
            label_row(1, "48:9e:bd:45:40:92", "100.180.65.40", "SIC21-CSD"),
            label_row(2, "48:9e:bd:45:40:92", "100.180.65.40", "SIC21-CSD"),
        ];

        let (kept, conflicts) = dedup_labels_first_wins(rows);

        assert_eq!(kept.len(), 1);
        assert!(conflicts.is_empty(), "même label répété -> pas un conflit");
    }

    #[test]
    fn dedup_first_wins_distinct_keys_are_all_kept() {
        // Deux équipements distincts partageant une IP : clés différentes -> conservés.
        let rows = vec![
            label_row(1, "84:a9:38:5e:d1:51", "100.180.65.32", "SIE - COMMIS"),
            label_row(2, "a0:2b:b8:3d:a2:7e", "100.180.65.32", "SIC21-SIC"),
        ];

        let (kept, conflicts) = dedup_labels_first_wins(rows);

        assert_eq!(kept.len(), 2);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn empty_ip_does_not_trigger_conflict() {
        let dir = TempDir::new("sonar_test_empty_ip_no_conflict");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:f1,,mon-pc\naa:bb:cc:dd:ee:ff,,ma-tablette\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok())
    }

    #[test]
    fn new_matrix_is_empty() {
        let matrix = FlowMatrix::new();
        assert!(matrix.matrix.is_empty());
    }

    #[test]
    fn labels_to_matrix_loads_labels_into_matrix() {
        let mut matrix = FlowMatrix::new();
        let mut label_store = LabelStore::new();
        let tab_test = [
            (
                String::from("aa:bb:cc:dd:ee:ff"),
                String::from("192.168.1.1"),
                String::from("mon-pc"),
            ),
            (
                String::from("aa:bb:cc:d5:ee:ff"),
                String::from("192.168.1.10"),
                String::from("ma-télé"),
            ),
            (
                String::from("aa:bb:cc:dd:ee:55"),
                String::from("aa:bb:cc:dd:ee:55"),
                String::from("mon-aspi"),
            ),
        ];

        for row in tab_test {
            label_store.add(row);
        }

        copy_labels_to_matrix(&label_store, &mut matrix).unwrap();

        assert_eq!(matrix.get_label_list().len(), 3)
    }

    /// Reproduit la construction de `import_matrix_file` (labels puis flux).
    fn build_matrix_and_graph(rows: &[FlowMatrixRow]) -> (FlowMatrix, GraphData) {
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();
        let label_store = LabelStore::new();
        rebuild_matrix_and_graph_from_rows(rows, &label_store, &mut matrix, &mut graph).unwrap();

        (matrix, graph)
    }

    #[test]
    fn import_matrix_rows_merges_duplicate_flows() {
        let mut rows = read_matrix_rows(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test_files/20260703_DR_Matrice.csv")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let mut duplicate = rows[0].clone();
        duplicate.count = 7;
        duplicate.total_bytes = 1234;
        duplicate.last_seen = "2099-01-01 00:00:00".to_string();
        rows.push(duplicate);

        let (flow, _stats) = rows[0].to_flow_and_stats();
        let (matrix, graph) = build_matrix_and_graph(&rows);
        let merged = matrix.matrix.get(&flow).unwrap();

        assert_eq!(
            matrix.matrix.len(),
            30,
            "le flux dupliqué doit être fusionné"
        );
        assert_eq!(merged.count, rows[0].count + 7);
        assert_eq!(merged.total_bytes, rows[0].total_bytes.saturating_add(1234));
        assert!(
            graph
                .edges
                .values()
                .any(|edge| edge.count >= rows[0].count + 7),
            "le graphe doit recevoir le trafic fusionné"
        );
    }

    /// Rejoue la chaîne de `import_matrix_file` sur le fichier réel de
    /// `test_files/` : lecture CSV, reconstruction des flux, remplissage de la
    /// matrice et du graphe.
    #[test]
    fn import_real_matrix_file_rebuilds_matrix_and_graph() {
        let matrix_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_DR_Matrice.csv")
            .to_str()
            .unwrap()
            .to_string();

        let rows = read_matrix_rows(&matrix_path).unwrap();
        assert_eq!(
            rows.len(),
            30,
            "30 lignes de données (l'en-tête est écarté)"
        );

        let (matrix, graph) = build_matrix_and_graph(&rows);

        assert_eq!(matrix.matrix.len(), 30, "un flux par ligne du fichier");
        assert!(!graph.nodes.is_empty(), "le graphe doit contenir des nœuds");
        assert!(
            !graph.edges.is_empty(),
            "le graphe doit contenir des arêtes"
        );

        // Les labels du fichier sont réappliqués sur les nœuds du graphe.
        let labelled = graph
            .nodes
            .values()
            .filter(|n| n.label.as_deref() == Some("pc sonar"))
            .count();
        assert!(
            labelled > 0,
            "au moins un nœud doit porter le label du fichier"
        );

        // Le CSV survit à un aller-retour export -> import (mêmes flux).
        let reexported = matrix.to_flat_vec();
        assert_eq!(reexported.len(), 30);
    }

    /// Même chaîne sur la matrice générée de 1000 lignes : vérifie la tenue
    /// en charge du parseur et la construction d'un graphe complet.
    #[test]
    fn import_1000_row_matrix_builds_full_graph() {
        let matrix_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_DR_Matrice_1000.csv")
            .to_str()
            .unwrap()
            .to_string();

        let rows = read_matrix_rows(&matrix_path).unwrap();
        assert_eq!(rows.len(), 1000);

        let (matrix, graph) = build_matrix_and_graph(&rows);

        assert_eq!(matrix.matrix.len(), 1000, "1000 flux distincts");
        assert_eq!(graph.nodes.len(), 232, "un nœud par adresse IP distincte");
        assert!(graph.edges.len() > 100, "arêtes: {}", graph.edges.len());

        // Les poids de trafic du fichier sont reportés sur les arêtes.
        assert!(
            graph
                .edges
                .values()
                .all(|e| e.count > 0 && e.total_bytes > 0),
            "chaque arête doit porter son trafic cumulé"
        );

        let labelled = graph.nodes.values().filter(|n| n.label.is_some()).count();
        assert!(labelled >= 20, "nœuds labellisés: {labelled}");
    }

    /// Rejoue la chaîne complète de `import_label_file` sur les fichiers réels
    /// de `test_files/` : vérifications, écart de l'en-tête, remplissage du
    /// store puis résolution des labels comme le ferait la matrice de flux.
    #[test]
    fn import_real_label_file_resolves_labels_for_matrix_hosts() {
        let label_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_DR_Labels.csv")
            .to_str()
            .unwrap()
            .to_string();

        verif_label_rows_format(label_path.clone()).unwrap();
        verif_mac_ip_format(label_path.clone()).unwrap();
        verif_labels_conflicts(label_path.clone()).unwrap();

        let mut rows = read_label_rows(&label_path).unwrap();
        if rows.first().is_some_and(is_header_row) {
            rows.remove(0);
        }

        let mut label_store = LabelStore::new();
        for row in rows {
            label_store.add(row.into_tuple());
        }
        assert_eq!(label_store.get().len(), 9, "l'en-tête doit être écarté");

        let mut matrix = FlowMatrix::new();
        copy_labels_to_matrix(&label_store, &mut matrix).unwrap();

        // Correspondance exacte MAC + IP.
        assert_eq!(
            matrix.get_label("e0:d5:5e:28:9b:d4", "192.168.1.181"),
            Some(String::from("PC Sonar"))
        );
        assert_eq!(
            matrix.get_label("44:15:24:20:a5:64", "192.168.1.254"),
            Some(String::from("Passerelle"))
        );
        // Repli sur l'IP seule, prioritaire sur la MAC seule.
        assert_eq!(
            matrix.get_label("e0:d5:5e:28:9b:d4", "2001:861:3fc7:9b00:e09f:22b:19f7:888e"),
            Some(String::from("PC Sonar public"))
        );
        // Repli sur la MAC seule quand l'IP est inconnue du fichier.
        assert_eq!(
            matrix.get_label("e0:d5:5e:28:9b:d4", "fe80::b861:e852:5fa3:d3e5"),
            Some(String::from("Machine de capture"))
        );
        // La notation CIDR du fichier est ramenée à l'adresse seule.
        assert_eq!(
            matrix.get_label("44:15:24:20:a5:64", "2600:1901:0:3084::"),
            Some(String::from("Serveur GCP (CIDR)"))
        );
        // Un label vide devient "Label?".
        assert_eq!(
            matrix.get_label("44:15:24:20:a5:64", "2606:50c0:8002::154"),
            Some(String::from("Label?"))
        );
        // Hôte absent du fichier de labels : aucun label.
        assert_eq!(matrix.get_label("44:15:24:20:a5:64", "2607:6bc0::10"), None);
    }
}
