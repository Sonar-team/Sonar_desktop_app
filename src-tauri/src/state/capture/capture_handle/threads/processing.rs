use crossbeam::channel::{Receiver, RecvTimeoutError};
use log::{debug, error, info};
#[cfg(feature = "capture_timing")]
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
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
                capture::{PacketMinimal, PacketOwnedStats},
                channel::ChannelCapacityPayload,
                stats::{StatTriple, StatsPayload},
            },
            threads::packet_buffer::PacketBufferPool,
        },
        flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};
use packet_parser::PacketFlow;
#[cfg(feature = "capture_timing")]
use packet_parser::timing::ParseTiming;

// Flush le batch de paquets vers le frontend. Retourne false si le canal est cassé.
macro_rules! flush_batch {
    ($batch:expr, $on_event:expr, $last_flush:expr, $timing_logger:ident) => {{
        if $batch.is_empty() {
            true
        } else {
            let batch_len = $batch.len();
            let packets = std::mem::take(&mut $batch);
            $last_flush = Instant::now();
            #[cfg(feature = "capture_timing")]
            let ipc_start = Instant::now();
            let send_result = $on_event.send(CaptureEvent::PacketBatch { packets });
            #[cfg(feature = "capture_timing")]
            {
                if let Some(logger) = $timing_logger.as_mut() {
                    if let Err(e) = logger.write_packet_batch_ipc(
                        batch_len,
                        elapsed_ns_since(ipc_start),
                        send_result.is_ok(),
                    ) {
                        error!(
                            "Capture timing log disabled after batch IPC write error: {}",
                            e
                        );
                        $timing_logger = None;
                    }
                }
            }

            match send_result {
                Ok(_) => true,
                Err(e) => {
                    error!("[TAURI] Erreur envoi PacketBatch: {}", e);
                    false
                }
            }
        }
    }};
    ($batch:expr, $on_event:expr, $last_flush:expr) => {{
        if $batch.is_empty() {
            true
        } else {
            let packets = std::mem::take(&mut $batch);
            $last_flush = Instant::now();
            match $on_event.send(CaptureEvent::PacketBatch { packets }) {
                Ok(_) => true,
                Err(e) => {
                    error!("[TAURI] Erreur envoi PacketBatch: {}", e);
                    false
                }
            }
        }
    }};
}

#[cfg(feature = "capture_timing")]
fn parse_packet_flow_with_timing<'a>(
    bytes: &'a [u8],
) -> Result<(PacketFlow<'a>, ParseTiming), packet_parser::ParsedPacketError> {
    let mut timing = ParseTiming::default();
    let flow = PacketFlow::try_from_timed(bytes, &mut timing)?;
    Ok((flow, timing))
}

#[cfg(feature = "capture_timing")]
#[derive(Clone, Copy)]
struct CaptureTimingSample {
    seq: u64,
    sample_rate: u64,
}

#[cfg(feature = "capture_timing")]
#[derive(Default)]
struct CapturePipelineTiming {
    caplen: u32,
    len: u32,
    parse_l2_ns: u64,
    parse_l3_ns: u64,
    parse_l4_ns: u64,
    parse_l7_ns: u64,
    parse_total_ns: u64,
    packet_owned_ns: u64,
    label_lookup_ns: u64,
    matrix_update_ns: u64,
    graph_update_ns: u64,
    graph_ipc_ns: u64,
    graph_updates: usize,
    graph_ipc_failures: usize,
    pipeline_total_ns: u64,
}

#[cfg(feature = "capture_timing")]
struct CaptureTimingLogger {
    writer: BufWriter<File>,
    sample_rate: u64,
    seen: u64,
    batch_seen: u64,
    pending_flush: u64,
    last_flush: Instant,
}

#[cfg(feature = "capture_timing")]
impl CaptureTimingLogger {
    fn new() -> io::Result<Self> {
        let path = capture_timing_log_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let sample_rate = std::env::var("SONAR_CAPTURE_TIMING_SAMPLE_RATE")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(100);

        info!(
            "Capture timing log enabled: path={} sample_rate={}",
            path.display(),
            sample_rate
        );

        Ok(Self {
            writer: BufWriter::new(file),
            sample_rate,
            seen: 0,
            batch_seen: 0,
            pending_flush: 0,
            last_flush: Instant::now(),
        })
    }

    fn next_sample(&mut self) -> Option<CaptureTimingSample> {
        self.seen = self.seen.saturating_add(1);
        if self.seen % self.sample_rate != 0 {
            return None;
        }

        Some(CaptureTimingSample {
            seq: self.seen,
            sample_rate: self.sample_rate,
        })
    }

    fn write_pipeline(
        &mut self,
        sample: CaptureTimingSample,
        timing: CapturePipelineTiming,
    ) -> io::Result<()> {
        let ts_unix_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        writeln!(
            self.writer,
            "{{\"event\":\"capture_pipeline_timing\",\"ts_unix_ns\":{},\"seq\":{},\"sample_rate\":{},\"caplen\":{},\"len\":{},\"parse_l2_ns\":{},\"parse_l3_ns\":{},\"parse_l4_ns\":{},\"parse_l7_ns\":{},\"parse_total_ns\":{},\"packet_owned_ns\":{},\"label_lookup_ns\":{},\"matrix_update_ns\":{},\"graph_update_ns\":{},\"graph_ipc_ns\":{},\"graph_updates\":{},\"graph_ipc_failures\":{},\"pipeline_total_ns\":{}}}",
            ts_unix_ns,
            sample.seq,
            sample.sample_rate,
            timing.caplen,
            timing.len,
            timing.parse_l2_ns,
            timing.parse_l3_ns,
            timing.parse_l4_ns,
            timing.parse_l7_ns,
            timing.parse_total_ns,
            timing.packet_owned_ns,
            timing.label_lookup_ns,
            timing.matrix_update_ns,
            timing.graph_update_ns,
            timing.graph_ipc_ns,
            timing.graph_updates,
            timing.graph_ipc_failures,
            timing.pipeline_total_ns
        )?;

        self.pending_flush = self.pending_flush.saturating_add(1);
        if self.pending_flush >= 256 || self.last_flush.elapsed() >= Duration::from_secs(1) {
            self.writer.flush()?;
            self.pending_flush = 0;
            self.last_flush = Instant::now();
        }

        Ok(())
    }

    fn write_packet_batch_ipc(
        &mut self,
        batch_len: usize,
        ipc_ns: u64,
        ok: bool,
    ) -> io::Result<()> {
        self.batch_seen = self.batch_seen.saturating_add(1);
        let ts_unix_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        writeln!(
            self.writer,
            "{{\"event\":\"capture_packet_batch_ipc_timing\",\"ts_unix_ns\":{},\"batch_seq\":{},\"batch_len\":{},\"ipc_ns\":{},\"ok\":{}}}",
            ts_unix_ns, self.batch_seen, batch_len, ipc_ns, ok
        )?;

        self.pending_flush = self.pending_flush.saturating_add(1);
        if self.pending_flush >= 256 || self.last_flush.elapsed() >= Duration::from_secs(1) {
            self.writer.flush()?;
            self.pending_flush = 0;
            self.last_flush = Instant::now();
        }

        Ok(())
    }
}

#[cfg(feature = "capture_timing")]
fn elapsed_ns_since(start: Instant) -> u64 {
    start.elapsed().as_nanos() as u64
}

#[cfg(feature = "capture_timing")]
fn capture_timing_log_path() -> PathBuf {
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

pub fn spawn_processing_thread(
    rx: Receiver<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
    stop_flag: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        let mut processed = 0;
        let mut last_channel = ChannelCapacityPayload::default();
        let mut last_update = Instant::now();
        let mut last_len = 0usize;
        static TEMPO: Duration = Duration::from_millis(40);
        let mut stats = StatTriple::default();
        let batch_max: usize = 64;
        let batch_interval = Duration::from_millis(50);
        let mut packet_batch: Vec<PacketOwnedStats> = Vec::with_capacity(batch_max);
        let mut last_batch_flush = Instant::now();
        #[cfg(feature = "capture_timing")]
        let mut timing_logger = match CaptureTimingLogger::new() {
            Ok(logger) => Some(logger),
            Err(e) => {
                error!("Capture timing log disabled: {}", e);
                None
            }
        };

        loop {
            let timeout = batch_interval.saturating_sub(last_batch_flush.elapsed());
            match rx.recv_timeout(timeout.max(Duration::from_millis(1))) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        // Drain quietly after stop: no packet event and no matrix update.
                        packet_batch.clear();
                        buffer_pool.put(pkt);
                        continue;
                    }

                    #[cfg(feature = "capture_timing")]
                    let timing_sample = timing_logger
                        .as_mut()
                        .and_then(CaptureTimingLogger::next_sample);
                    #[cfg(feature = "capture_timing")]
                    let pipeline_start = timing_sample.map(|_| Instant::now());

                    #[cfg(feature = "capture_timing")]
                    let (flow, parse_timing) = if timing_sample.is_some() {
                        match parse_packet_flow_with_timing(pkt.as_ref()) {
                            Ok(parsed) => parsed,
                            Err(e) => {
                                error!("Failed to parse PacketFlow: {}", e);
                                buffer_pool.put(pkt);
                                continue;
                            }
                        }
                    } else {
                        match PacketFlow::try_from(pkt.as_ref()) {
                            Ok(flow) => (flow, ParseTiming::default()),
                            Err(e) => {
                                error!("Failed to parse PacketFlow: {}", e);
                                buffer_pool.put(pkt);
                                continue;
                            }
                        }
                    };

                    #[cfg(not(feature = "capture_timing"))]
                    let flow = match PacketFlow::try_from(pkt.as_ref()) {
                        Ok(flow) => flow,
                        Err(e) => {
                            error!("Failed to parse PacketFlow: {}", e);
                            buffer_pool.put(pkt);
                            continue;
                        }
                    };

                    let packet = PacketMinimal {
                        ts_sec: pkt.header.ts.tv_sec,
                        ts_usec: pkt.header.ts.tv_usec,
                        caplen: pkt.header.caplen,
                        len: pkt.header.len,
                        flow,
                    };

                    // ajout les paquets à la matrice de flux
                    #[cfg(feature = "capture_timing")]
                    let packet_owned_start = timing_sample.map(|_| Instant::now());
                    let record_owned = packet.to_owned_packet();
                    #[cfg(feature = "capture_timing")]
                    let packet_owned_ns = packet_owned_start.map(elapsed_ns_since).unwrap_or(0);

                    let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>();
                    #[cfg(feature = "capture_timing")]
                    let label_lookup_start = timing_sample.map(|_| Instant::now());
                    let (source_label, destination_label) =
                        if let Ok(locked_state) = flow_matrix.lock() {
                            let source_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.source_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();
                            let destination_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.destination_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();

                            (
                                locked_state
                                    .get_label(&record_owned.flow.data_link.source_mac, &source_ip),
                                locked_state.get_label(
                                    &record_owned.flow.data_link.destination_mac,
                                    &destination_ip,
                                ),
                            )
                        } else {
                            (None, None)
                        };
                    #[cfg(feature = "capture_timing")]
                    let label_lookup_ns = label_lookup_start.map(elapsed_ns_since).unwrap_or(0);

                    #[cfg(feature = "capture_timing")]
                    let matrix_update_start = timing_sample.map(|_| Instant::now());
                    if let Ok(mut locked_state) = flow_matrix.lock() {
                        locked_state.update_flow(&record_owned);
                        processed = locked_state.matrix.len() as u32;
                    };
                    #[cfg(feature = "capture_timing")]
                    let matrix_update_ns = matrix_update_start.map(elapsed_ns_since).unwrap_or(0);

                    let graph = app.state::<Arc<Mutex<GraphData>>>();
                    #[cfg(feature = "capture_timing")]
                    let graph_update_start = timing_sample.map(|_| Instant::now());
                    let graph_updates = if let Ok(mut g) = graph.lock() {
                        g.add_packet_flow(&record_owned.flow, source_label, destination_label)
                    } else {
                        Vec::new()
                    };
                    #[cfg(feature = "capture_timing")]
                    let graph_update_ns = graph_update_start.map(elapsed_ns_since).unwrap_or(0);

                    #[cfg(feature = "capture_timing")]
                    let graph_ipc_start = timing_sample.map(|_| Instant::now());
                    #[cfg(feature = "capture_timing")]
                    let graph_update_count = graph_updates.len();
                    #[cfg(feature = "capture_timing")]
                    let mut graph_ipc_failures = 0usize;
                    if !graph_updates.is_empty() {
                        for update in graph_updates {
                            if let Err(e) = on_event.send(CaptureEvent::Graph { update: &update }) {
                                #[cfg(feature = "capture_timing")]
                                {
                                    graph_ipc_failures += 1;
                                }
                                error!("[TAURI] Erreur envoi GraphUpdate: {}", e);
                                break;
                            }
                        }
                    }
                    #[cfg(feature = "capture_timing")]
                    let graph_ipc_ns = graph_ipc_start.map(elapsed_ns_since).unwrap_or(0);

                    #[cfg(feature = "capture_timing")]
                    if let (Some(sample), Some(start), Some(logger)) =
                        (timing_sample, pipeline_start, timing_logger.as_mut())
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
                            graph_ipc_failures,
                            pipeline_total_ns: elapsed_ns_since(start),
                        };

                        if let Err(e) = logger.write_pipeline(sample, pipeline_timing) {
                            error!("Capture timing log disabled after write error: {}", e);
                            timing_logger = None;
                        }
                    }

                    // Accumuler dans le batch
                    packet_batch.push(record_owned);

                    // Flush si le batch est plein ou si l'intervalle est écoulé
                    if packet_batch.len() >= batch_max
                        || last_batch_flush.elapsed() >= batch_interval
                    {
                        #[cfg(feature = "capture_timing")]
                        let flush_ok =
                            flush_batch!(packet_batch, on_event, last_batch_flush, timing_logger);
                        #[cfg(not(feature = "capture_timing"))]
                        let flush_ok = flush_batch!(packet_batch, on_event, last_batch_flush);

                        if !flush_ok {
                            buffer_pool.put(pkt);
                            break;
                        }
                    }

                    buffer_pool.put(pkt);
                }

                Ok(CaptureMessage::Stats(new_stats)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        continue;
                    }

                    if let Err(e) =
                        StatsPayload::maybe_send(&mut stats, new_stats, processed, &on_event)
                    {
                        error!("[TAURI] Erreur envoi Stats: {}", e);
                    }
                }

                Err(RecvTimeoutError::Timeout) => {
                    // Flush le batch restant après inactivité
                    #[cfg(feature = "capture_timing")]
                    let flush_ok =
                        flush_batch!(packet_batch, on_event, last_batch_flush, timing_logger);
                    #[cfg(not(feature = "capture_timing"))]
                    let flush_ok = flush_batch!(packet_batch, on_event, last_batch_flush);

                    if !flush_ok {
                        break;
                    }
                }

                Err(RecvTimeoutError::Disconnected) => {
                    error!("Erreur réception canal : canal déconnecté");
                    break;
                }
            }

            let current_len = rx.len();

            if last_len != current_len || last_update.elapsed() >= TEMPO {
                last_update = Instant::now();
                last_len = current_len;

                if let Err(e) = ChannelCapacityPayload::send_if_changed(
                    &mut last_channel,
                    current_len,
                    channel_capacity as usize,
                    &on_event,
                ) {
                    error!("[TAURI] Erreur émission canal : {}", e);
                }
            }
        }
    });
}

pub fn spawn_processing_thread_cli(
    rx: Receiver<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
    stop_flag: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        let mut processed = 0;
        let mut last_channel = ChannelCapacityPayload::default();
        let mut last_update = Instant::now();
        let mut last_len = 0usize;
        static TEMPO: Duration = Duration::from_millis(40);
        let mut stats = StatTriple::default();
        let batch_max: usize = 64;
        let batch_interval = Duration::from_millis(50);
        let mut packet_batch: Vec<PacketOwnedStats> = Vec::with_capacity(batch_max);
        let mut last_batch_flush = Instant::now();

        loop {
            let timeout = batch_interval.saturating_sub(last_batch_flush.elapsed());
            match rx.recv_timeout(timeout.max(Duration::from_millis(1))) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        packet_batch.clear();
                        buffer_pool.put(pkt);
                        continue;
                    }

                    let flow = match PacketFlow::try_from(pkt.as_ref()) {
                        Ok(flow) => flow,
                        Err(e) => {
                            error!("Failed to parse PacketFlow: {}", e);
                            buffer_pool.put(pkt);
                            continue;
                        }
                    };

                    let packet = PacketMinimal {
                        ts_sec: pkt.header.ts.tv_sec,
                        ts_usec: pkt.header.ts.tv_usec,
                        caplen: pkt.header.caplen,
                        len: pkt.header.len,
                        flow,
                    };

                    let record_owned = packet.to_owned_packet();
                    let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>();
                    let (source_label, destination_label) =
                        if let Ok(locked_state) = flow_matrix.lock() {
                            let source_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.source_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();
                            let destination_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.destination_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();

                            (
                                locked_state
                                    .get_label(&record_owned.flow.data_link.source_mac, &source_ip),
                                locked_state.get_label(
                                    &record_owned.flow.data_link.destination_mac,
                                    &destination_ip,
                                ),
                            )
                        } else {
                            (None, None)
                        };
                    if let Ok(mut locked_state) = flow_matrix.lock() {
                        let (flow_stats, flow) = locked_state.update_flow_cli(&record_owned);
                        info!("flow_stats: {:?}, {:?}", flow_stats, flow);
                        processed = locked_state.matrix.len() as u32;
                    };

                    let graph = app.state::<Arc<Mutex<GraphData>>>();
                    if let Ok(mut g) = graph.lock() {
                        let updates =
                            g.add_packet_flow(&record_owned.flow, source_label, destination_label);
                        if !updates.is_empty() {
                            for update in updates {
                                if let Err(e) =
                                    on_event.send(CaptureEvent::Graph { update: &update })
                                {
                                    error!("[TAURI] Erreur envoi GraphUpdate: {}", e);
                                    break;
                                }
                            }
                        }
                    };

                    packet_batch.push(record_owned);

                    if packet_batch.len() >= batch_max
                        || last_batch_flush.elapsed() >= batch_interval
                    {
                        if !flush_batch!(packet_batch, on_event, last_batch_flush) {
                            buffer_pool.put(pkt);
                            break;
                        }
                    }

                    buffer_pool.put(pkt);
                }

                Ok(CaptureMessage::Stats(new_stats)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        continue;
                    }

                    if let Err(e) =
                        StatsPayload::maybe_send(&mut stats, new_stats, processed, &on_event)
                    {
                        error!("[TAURI] Erreur envoi Stats: {}", e);
                    }
                }

                Err(RecvTimeoutError::Timeout) => {
                    if !flush_batch!(packet_batch, on_event, last_batch_flush) {
                        break;
                    }
                }

                Err(RecvTimeoutError::Disconnected) => {
                    error!("Erreur réception canal : canal déconnecté");
                    break;
                }
            }

            let current_len = rx.len();

            if last_len != current_len || last_update.elapsed() >= TEMPO {
                last_update = Instant::now();
                last_len = current_len;

                if let Err(e) = ChannelCapacityPayload::send_if_changed(
                    &mut last_channel,
                    current_len,
                    channel_capacity as usize,
                    &on_event,
                ) {
                    error!("[TAURI] Erreur émission canal : {}", e);
                }
            }
        }
    });
}
