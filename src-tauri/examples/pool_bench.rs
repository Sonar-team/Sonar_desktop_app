//! Benchmark du PacketBufferPool : mémoire et débit sous charge simulée.
//!
//! Deux modes, à lancer dans des process séparés pour des mesures RSS propres :
//! ```bash
//! cargo run --release --example pool_bench -- eager   # ancien comportement (préallocation)
//! cargo run --release --example pool_bench -- lazy    # nouveau pool (2 classes, paresseux)
//! ```
//!
//! La charge simule une capture réelle : mix de tailles de trames (petits ACKs,
//! trames MTU, ~1 % de jumbo GRO/TSO), avec une profondeur d'in-flight qui
//! oscille et des rafales de backpressure.

use std::collections::VecDeque;
use std::time::Instant;

use pcap::PacketHeader;
use sonar_lib::packet_buffer::{PacketBuffer, PacketBufferPool};

const POOL_SIZE: usize = 10_002; // chan_capacity (10 000) + 2, config par défaut
const SNAPLEN: usize = 65_536;
const ITERATIONS: usize = 2_000_000;
const STEADY_DEPTH: usize = 256; // in-flight nominal
const BURST_DEPTH: usize = 4_096; // rafale de backpressure
const BURST_EVERY: usize = 200_000;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    match mode.as_str() {
        "eager" => run_eager(),
        "lazy" => run_lazy(),
        _ => {
            eprintln!("usage: pool_bench <eager|lazy>");
            std::process::exit(1);
        }
    }
}

/// Ancien comportement : POOL_SIZE buffers de SNAPLEN octets préalloués.
fn run_eager() {
    let queue = crossbeam::queue::SegQueue::new();
    for _ in 0..POOL_SIZE {
        queue.push(PacketBuffer::new(SNAPLEN));
    }
    println!(
        "[eager] pool préalloué : {} buffers × {} KiB",
        POOL_SIZE,
        SNAPLEN / 1024
    );
    report_rss("après préallocation");

    let start = Instant::now();
    let processed = drive_workload(|_caplen| queue.pop(), |buffer| queue.push(buffer));
    report_throughput(processed, start);
    report_rss("après charge");
}

/// Nouveau pool : allocation paresseuse, deux classes de tailles.
fn run_lazy() {
    let pool = PacketBufferPool::new(POOL_SIZE, SNAPLEN);
    println!("[lazy] pool vide au démarrage (budget {POOL_SIZE} buffers)");
    report_rss("au démarrage");

    let start = Instant::now();
    let processed = drive_workload(|caplen| pool.get(caplen), |buffer| pool.put(buffer));
    report_throughput(processed, start);

    let stats = pool.stats();
    println!(
        "[lazy] alloués : {} petits + {} jumbo = {:.1} MiB ; refus : {}",
        stats.small_allocated,
        stats.large_allocated,
        pool.allocated_bytes() as f64 / (1024.0 * 1024.0),
        stats.exhausted
    );
    report_rss("après charge");
}

/// Fait tourner le workload et retourne le nombre de paquets traités.
fn drive_workload<G, P>(mut get: G, mut put: P) -> usize
where
    G: FnMut(usize) -> Option<PacketBuffer>,
    P: FnMut(PacketBuffer),
{
    let mut rng = XorShift(0x2545_F491_4F6C_DD1D);
    let mut in_flight: VecDeque<PacketBuffer> = VecDeque::with_capacity(BURST_DEPTH);
    let payload = vec![0xA5u8; SNAPLEN];
    let mut processed = 0usize;

    for i in 0..ITERATIONS {
        let caplen = pick_caplen(&mut rng);
        let target_depth = if i % BURST_EVERY < BURST_DEPTH {
            BURST_DEPTH
        } else {
            STEADY_DEPTH
        };

        if let Some(mut buffer) = get(caplen) {
            buffer.write_from_parts(&header(caplen as u32), &payload[..caplen]);
            in_flight.push_back(buffer);
            processed += 1;
        }

        while in_flight.len() > target_depth {
            if let Some(buffer) = in_flight.pop_front() {
                put(buffer);
            }
        }
    }
    while let Some(buffer) = in_flight.pop_front() {
        put(buffer);
    }

    processed
}

/// Mix réaliste : 60 % petits paquets, 25 % MTU, 14 % moyens, 1 % jumbo.
fn pick_caplen(rng: &mut XorShift) -> usize {
    match rng.next() % 100 {
        0..=59 => 66,
        60..=84 => 1_514,
        85..=98 => 300,
        _ => 9_000,
    }
}

fn header(caplen: u32) -> PacketHeader {
    PacketHeader {
        ts: libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        caplen,
        len: caplen,
    }
}

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn report_throughput(processed: usize, start: Instant) {
    let elapsed = start.elapsed();
    println!(
        "débit : {:.2} M paquets/s ({} paquets en {:.2?})",
        processed as f64 / elapsed.as_secs_f64() / 1_000_000.0,
        processed,
        elapsed
    );
}

fn report_rss(label: &str) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let rss_kb: u64 = status
        .lines()
        .find(|line| line.starts_with("VmRSS:"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    println!("RSS {label} : {:.1} MiB", rss_kb as f64 / 1024.0);
}
