//! Instrumentation de la capture live (feature `capture_timing`) :
//! échantillonnage des paquets, journal JSONL des durées par étape du
//! pipeline, timing IPC des batches et résumé de fin de run.

use log::info;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use packet_parser::{LinkType, PacketFlow, timing::ParseTiming};

use super::packet_buffer::PacketBufferPool;
use super::processing::{PACKET_BATCH_INTERVAL_MS, PACKET_BATCH_MAX};

static CAPTURE_TIMING_RUN_COUNTER: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

pub(super) fn parse_packet_flow_with_timing(
    link_type: LinkType,
    bytes: &[u8],
) -> Result<(PacketFlow<'_>, ParseTiming), packet_parser::ParseError> {
    let mut timing = ParseTiming::default();
    let flow = packet_parser::parse::parse_timed(link_type, bytes, &mut timing)?;
    Ok((flow, timing))
}

#[derive(Clone, Copy)]
pub(super) struct CaptureTimingSample {
    pub(super) seq: u64,
    pub(super) sample_rate: u64,
}

#[derive(Default)]
pub(super) struct CapturePipelineTiming {
    pub(super) caplen: u32,
    pub(super) len: u32,
    pub(super) parse_l2_ns: u64,
    pub(super) parse_l3_ns: u64,
    pub(super) parse_l4_ns: u64,
    pub(super) parse_l7_ns: u64,
    pub(super) parse_total_ns: u64,
    pub(super) packet_owned_ns: u64,
    pub(super) label_lookup_ns: u64,
    pub(super) matrix_update_ns: u64,
    pub(super) graph_update_ns: u64,
    pub(super) graph_ipc_ns: u64,
    pub(super) graph_updates: usize,
    pub(super) graph_ipc_failures: usize,
    pub(super) pipeline_total_ns: u64,
}

pub(super) struct CaptureTimingLogger {
    writer: BufWriter<File>,
    run_id: String,
    sample_rate: u64,
    seen: u64,
    batch_seen: u64,
    batch_first_ts_unix_ns: Option<u128>,
    batch_last_ts_unix_ns: Option<u128>,
    batch_packet_total: u64,
    batch_full_total: u64,
    batch_ipc_total_ns: u128,
    batch_ipc_values: Vec<u64>,
    summary_written: bool,
    pending_flush: u64,
    last_flush: Instant,
}

impl CaptureTimingLogger {
    pub(super) fn new() -> io::Result<Self> {
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
        let run_id = capture_timing_run_id();

        info!(
            "Capture timing log enabled: path={} run_id={} sample_rate={}",
            path.display(),
            run_id,
            sample_rate
        );

        Ok(Self {
            writer: BufWriter::new(file),
            run_id,
            sample_rate,
            seen: 0,
            batch_seen: 0,
            batch_first_ts_unix_ns: None,
            batch_last_ts_unix_ns: None,
            batch_packet_total: 0,
            batch_full_total: 0,
            batch_ipc_total_ns: 0,
            batch_ipc_values: Vec::new(),
            summary_written: false,
            pending_flush: 0,
            last_flush: Instant::now(),
        })
    }

    pub(super) fn next_sample(&mut self) -> Option<CaptureTimingSample> {
        self.seen = self.seen.saturating_add(1);
        if !self.seen.is_multiple_of(self.sample_rate) {
            return None;
        }

        Some(CaptureTimingSample {
            seq: self.seen,
            sample_rate: self.sample_rate,
        })
    }

    pub(super) fn write_pipeline(
        &mut self,
        sample: CaptureTimingSample,
        timing: CapturePipelineTiming,
    ) -> io::Result<()> {
        let ts_unix_ns = unix_now_ns();

        writeln!(
            self.writer,
            "{{\"event\":\"capture_pipeline_timing\",\"ts_unix_ns\":{},\"run_id\":\"{}\",\"seq\":{},\"sample_rate\":{},\"caplen\":{},\"len\":{},\"parse_l2_ns\":{},\"parse_l3_ns\":{},\"parse_l4_ns\":{},\"parse_l7_ns\":{},\"parse_total_ns\":{},\"packet_owned_ns\":{},\"label_lookup_ns\":{},\"matrix_update_ns\":{},\"graph_update_ns\":{},\"graph_ipc_ns\":{},\"graph_updates\":{},\"graph_ipc_failures\":{},\"pipeline_total_ns\":{}}}",
            ts_unix_ns,
            self.run_id,
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

        self.flush_if_due()
    }

    pub(super) fn write_packet_batch_ipc(
        &mut self,
        batch_len: usize,
        ipc_ns: u64,
        ok: bool,
    ) -> io::Result<()> {
        self.batch_seen = self.batch_seen.saturating_add(1);
        let ts_unix_ns = unix_now_ns();
        let batch_full = usize::from(batch_len >= PACKET_BATCH_MAX);

        self.batch_first_ts_unix_ns.get_or_insert(ts_unix_ns);
        self.batch_last_ts_unix_ns = Some(ts_unix_ns);
        self.batch_packet_total = self
            .batch_packet_total
            .saturating_add(batch_len.try_into().unwrap_or(u64::MAX));
        self.batch_full_total = self
            .batch_full_total
            .saturating_add(batch_full.try_into().unwrap_or(0));
        self.batch_ipc_total_ns = self.batch_ipc_total_ns.saturating_add(ipc_ns as u128);
        self.batch_ipc_values.push(ipc_ns);

        writeln!(
            self.writer,
            "{{\"event\":\"capture_packet_batch_ipc_timing\",\"ts_unix_ns\":{},\"run_id\":\"{}\",\"batch_seq\":{},\"batch_len\":{},\"batch_max\":{},\"batch_interval_ms\":{},\"batch_full\":{},\"ipc_ns\":{},\"ok\":{}}}",
            ts_unix_ns,
            self.run_id,
            self.batch_seen,
            batch_len,
            PACKET_BATCH_MAX,
            PACKET_BATCH_INTERVAL_MS,
            batch_full,
            ipc_ns,
            ok
        )?;

        self.flush_if_due()
    }

    fn flush_if_due(&mut self) -> io::Result<()> {
        self.pending_flush = self.pending_flush.saturating_add(1);
        if self.pending_flush >= 256 || self.last_flush.elapsed() >= Duration::from_secs(1) {
            self.writer.flush()?;
            self.pending_flush = 0;
            self.last_flush = Instant::now();
        }

        Ok(())
    }

    pub(super) fn write_run_summary(&mut self, buffer_pool: &PacketBufferPool) -> io::Result<()> {
        if self.summary_written {
            return self.writer.flush();
        }

        let ts_unix_ns = unix_now_ns();
        let batch_count = self.batch_seen;
        let active_duration_ns = match (self.batch_first_ts_unix_ns, self.batch_last_ts_unix_ns) {
            (Some(first), Some(last)) => last.saturating_sub(first) as u64,
            _ => 0,
        };
        let avg_packets_per_second = if active_duration_ns > 0 {
            (self.batch_packet_total as f64 * 1_000_000_000f64) / active_duration_ns as f64
        } else {
            0.0
        };
        let packet_batch_ipc_avg_ns = if batch_count > 0 {
            (self.batch_ipc_total_ns / batch_count as u128) as u64
        } else {
            0
        };
        let mut sorted_ipc = self.batch_ipc_values.clone();
        sorted_ipc.sort_unstable();
        let packet_batch_ipc_p95_ns = percentile_ns(&sorted_ipc, 0.95);
        let packet_batch_ipc_p99_ns = percentile_ns(&sorted_ipc, 0.99);

        let pool_stats = buffer_pool.stats();
        writeln!(
            self.writer,
            "{{\"event\":\"capture_run_summary\",\"ts_unix_ns\":{},\"run_id\":\"{}\",\"packet_total\":{},\"avg_packets_per_second\":{:.3},\"batch_count\":{},\"batch_max\":{},\"batch_interval_ms\":{},\"full_batch_count\":{},\"packet_batch_ipc_avg_ns\":{},\"packet_batch_ipc_p95_ns\":{},\"packet_batch_ipc_p99_ns\":{},\"active_duration_ns\":{},\"pool_small_allocated\":{},\"pool_large_allocated\":{},\"pool_allocated_bytes\":{},\"pool_exhausted\":{}}}",
            ts_unix_ns,
            self.run_id,
            self.batch_packet_total,
            avg_packets_per_second,
            batch_count,
            PACKET_BATCH_MAX,
            PACKET_BATCH_INTERVAL_MS,
            self.batch_full_total,
            packet_batch_ipc_avg_ns,
            packet_batch_ipc_p95_ns,
            packet_batch_ipc_p99_ns,
            active_duration_ns,
            pool_stats.small_allocated,
            pool_stats.large_allocated,
            buffer_pool.allocated_bytes(),
            pool_stats.exhausted
        )?;
        self.summary_written = true;
        self.writer.flush()
    }
}

pub(super) fn elapsed_ns_since(start: Instant) -> u64 {
    start.elapsed().as_nanos() as u64
}

fn unix_now_ns() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn capture_timing_run_id() -> String {
    let run_index = CAPTURE_TIMING_RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let raw_prefix = std::env::var("SONAR_CAPTURE_TIMING_RUN_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("capture-{}-{}", std::process::id(), unix_now_ns()));

    let sanitized_prefix: String = raw_prefix
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();

    format!("{sanitized_prefix}-run{run_index:02}")
}

fn percentile_ns(sorted_values: &[u64], quantile: f64) -> u64 {
    if sorted_values.is_empty() {
        return 0;
    }

    let last_index = sorted_values.len() - 1;
    let index = (last_index as f64 * quantile).ceil() as usize;
    sorted_values[index.min(last_index)]
}

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
        base.join("fr.sonar.ssf/logs").join(file_name)
    }

    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
            .join("fr.sonar.ssf\\logs")
            .join(file_name)
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/Users/Shared"))
            .join("Library/Logs/fr.sonar.ssf")
            .join(file_name)
    }
}
