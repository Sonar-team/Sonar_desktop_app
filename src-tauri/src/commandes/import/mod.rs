use log::{error, info};
use packet_parser::PacketFlow;
#[cfg(feature = "capture_timing")]
use packet_parser::timing::ParseTiming;
use pcap::Capture;
#[cfg(feature = "capture_timing")]
use serde_json::json;
use std::sync::{Arc, Mutex};
#[cfg(feature = "capture_timing")]
use std::time::Instant;
#[cfg(feature = "capture_timing")]
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError},
    events::CaptureEvent,
    state::{
        capture::capture_handle::messages::capture::PacketMinimal, flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};

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
                let graph_updates =
                    graph.add_packet_flow(&owned_packet.flow, source_label, destination_label);
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

            graph.add_packet_flow(&owned_packet.flow, source_label, destination_label);

            // Stats périodiques (optionnel)
            if (packet_count.is_multiple_of(1000) || packet_count == total)
                && let Err(e) = on_event.send(CaptureEvent::Stats {
                    received: packet_count as u32,
                    dropped: 0,
                    if_dropped: 0,
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
