use log::{error, info};
use packet_parser::PacketFlow;
#[cfg(feature = "capture_timing")]
use packet_parser::timing::ParseTiming;
use pcap::Capture;
#[cfg(feature = "capture_timing")]
use serde_json::json;
#[cfg(feature = "capture_timing")]
use std::time::Instant;
#[cfg(feature = "capture_timing")]
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use std::{
    io::ErrorKind,
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError, label::LabelError},
    events::CaptureEvent,
    setup::labels::{clean_csv_field, parse_label_row},
    state::{
        capture::capture_handle::messages::capture::PacketMinimal, flow_matrix::FlowMatrix,
        graph::GraphData, labels_list::LabelStore,
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

type ConflictsList = Vec<(String, String, String)>;

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

/*<----- Csv part -----> */

#[tauri::command(async)]
pub fn get_label_rows(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<Vec<(String, String, String)>, CaptureStateError> {
    let label_store = label_store.lock().unwrap();
    let label_rows = label_store.get();

    if label_rows.is_empty() {
        return Ok(label_rows.clone());
    }

    let filtered_label_rows =
        if !is_mac_address(&label_rows[0].0) && !is_ip_address(&label_rows[0].1) {
            label_rows.iter().skip(1).cloned().collect()
        } else {
            label_rows.clone()
        };
    Ok(filtered_label_rows)
}

#[tauri::command(async)]
pub fn clear_label_store(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<(), CaptureStateError> {
    let mut labels = label_store.lock().unwrap();
    labels.clear();
    println!("LabelStore cleared");
    Ok(())
}

fn verif_label_rows_format(file: String) -> Result<(), CaptureStateError> {
    let mut invalid_lines: Vec<String> = Vec::new();

    let file = match std::fs::read_to_string(&file) {
        Ok(csv_data) => csv_data,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    for line in file.lines() {
        let parts: Vec<_> = line.split(',').map(clean_csv_field).collect();
        if parts.len() != 3 && !parts.is_empty() {
            invalid_lines.push(line.to_string())
        }
    }

    if !invalid_lines.is_empty() {
        Err(LabelError::InvalidRowsFormat { invalid_lines }.into())
    } else {
        Ok(())
    }
}

fn verif_labels_conflicts(file_path: String) -> Result<(), CaptureStateError> {
    let file: PathBuf = PathBuf::from(file_path);

    let file = match std::fs::read_to_string(&file) {
        Ok(csv_data) => csv_data,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    let rows: Vec<(String, String, String)> = file.lines().filter_map(parse_label_row).collect();

    let mut same_ip_different_mac: ConflictsList = Vec::new();
    let mut same_ip_different_label: ConflictsList = Vec::new();

    let x = if is_mac_address(&rows[0].0) || is_ip_address(&rows[0].1) {
        0
    } else {
        1
    };
    println!("{}", x);
    for (i, (mac1, ip1, label1)) in rows.iter().enumerate().skip(x) {
        for (mac2, ip2, label2) in rows[i + 1..].iter() {
            if ip1 == ip2 && !ip1.is_empty() {
                if mac1 != mac2 {
                    eprintln!("⚠️  IP '{}' : MAC '{}' vs '{}'", ip1, mac1, mac2);
                    same_ip_different_mac.push((
                        ip1.to_string(),
                        mac1.to_string(),
                        mac2.to_string(),
                    ))
                }

                if label1 != label2 {
                    eprintln!("⚠️  IP '{}' : label '{}' vs '{}'", ip1, label1, label2);
                    same_ip_different_label.push((
                        ip1.to_string(),
                        label1.to_string(),
                        label2.to_string(),
                    ))
                }
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
    let mut invalid_ip: Vec<String> = Vec::new();
    let mut invalid_mac: Vec<String> = Vec::new();
    println!("verif_mac_ip_format called with csv_path: {:?}", csv_path);

    let file = match std::fs::read_to_string(&csv_path) {
        Ok(csv_data) => csv_data,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    let rows: Vec<(String, String, String)> = file.lines().filter_map(parse_label_row).collect();
    println!("rows in verif_mac_ip_format: {:?}", rows);

    let x = if is_mac_address(&rows[0].0) || is_ip_address(&rows[0].1) {
        0
    } else {
        1
    };

    for (mac, ip, _label) in rows.iter().skip(x) {
        if !is_ip_address(ip) && !ip.is_empty() {
            invalid_ip.push(ip.to_string());
        }
        if !is_mac_address(mac) && !mac.is_empty() {
            invalid_mac.push(mac.to_string());
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

#[tauri::command(async)]
pub fn import_label_file(
    incoming_file_path: String,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<(), CaptureStateError> {
    {
        let mut label_store = label_store.lock().unwrap();

        verif_label_rows_format(incoming_file_path.clone())?;
        verif_mac_ip_format(incoming_file_path.clone())?;
        verif_labels_conflicts(incoming_file_path.clone())?;

        label_store.clear();

        let file = match std::fs::read_to_string(&incoming_file_path) {
            Ok(csv_data) => csv_data,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error.into()),
        };

        let labels: Vec<String> = file.lines().map(|l| l.to_string()).collect();

        for label in labels {
            let Some((mac, ip, label)) = parse_label_row(&label) else {
                continue;
            };

            label_store.add((mac, ip, label))
        }

        println!(
            "copie du contenu de {:?} dans l'état partagé 'LabelStore' effectuée",
            &incoming_file_path
        );
    }

    let mut state_label = state_label.lock().unwrap();
    labels_to_matrix(label_store, &mut state_label)?;

    Ok(())
}

pub fn labels_to_matrix(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    let mut label_store = label_store.lock().unwrap();
    load_labels_from_folder(&mut label_store, matrice)
}

pub fn load_labels_from_folder(
    label_store: &mut LabelStore,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    let rows = label_store.get();

    for (mac, ip, label) in rows {
        matrice.add_label(mac.to_string(), ip.to_string(), label.to_string());
    }

    Ok(())
}

#[tauri::command]
pub fn is_matrix_empty(state: tauri::State<'_, Arc<Mutex<FlowMatrix>>>) -> bool {
    state.lock().unwrap().matrix.is_empty()
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

        assert!(result.is_err());
    }

    #[test]
    fn extra_column_returns_invalid_file_format_error() {
        let dir = TempDir::new("sonar_test_extra_column");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc,réseau-2\n",
        )
        .unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_err());
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
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.11,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_err());
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
    fn no_conflict_returns_ok() {
        let dir = TempDir::new("sonar_test_no_conflict");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:1f,192.168.1.2,ma-tablette\naa:bb:cc:dd:ee:ff,192.168.1.3,mon-tel\n").unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok())
    }

    #[test]
    fn same_ip_different_mac_returns_conflict_error() {
        let dir = TempDir::new("sonar_test_same_ip_diff_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:1f,192.168.1.1,mon-pc\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_err())
    }

    #[test]
    fn same_ip_different_label_returns_conflict_error() {
        let dir = TempDir::new("sonar_test_same_ip_diff_label");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:ff,192.168.1.1,ma-tablette\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_err())
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

        load_labels_from_folder(&mut label_store, &mut matrix).unwrap();

        assert_eq!(matrix.get_label_list().len(), 3)
    }
}
