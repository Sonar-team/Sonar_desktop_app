#[cfg(feature = "parse_timing")]
use std::default;

// packet_parser/src/timing.rs
#[cfg(feature = "parse_timing")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ParseTiming {
    pub l2_ns: u64,
    pub l3_ns: u64,
    pub l4_ns: u64,
    pub l7_ns: u64,
    pub total_ns: u64,
}

#[cfg(feature = "parse_timing")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayerAttempt {
    #[default]
    Skipped = 0,
    Ok = 1,
    Unsupported = 2,
}

#[cfg(feature = "parse_timing")]
#[derive(Debug, Clone, Copy, Default)]
pub struct ParseReport {
    pub timing: ParseTiming,
    pub l3: LayerAttempt,
    pub l4: LayerAttempt,
    pub l7: LayerAttempt,
}

#[cfg(not(feature = "parse_timing"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct ParseTiming;

// packet_parser/src/timing.rs
#[cfg(feature = "parse_timing")]
#[inline(always)]
pub fn now() -> std::time::Instant {
    std::time::Instant::now()
}

#[cfg(feature = "parse_timing")]
#[inline(always)]
pub fn elapsed_ns(t0: std::time::Instant) -> u64 {
    t0.elapsed().as_nanos() as u64
}

#[cfg(not(feature = "parse_timing"))]
#[inline(always)]
pub fn now() {}

#[cfg(not(feature = "parse_timing"))]
#[inline(always)]
pub fn elapsed_ns(_: ()) -> u64 {
    0
}

// packet_parser/src/timing.rs
#[cfg(feature = "parse_timing")]
#[macro_export]
macro_rules! time_block_ns {
    ($dst:expr, $body:block) => {{
        let t0 = $crate::timing::now();
        let out = (|| $body)();
        *$dst = $crate::timing::elapsed_ns(t0);
        out
    }};
}

#[cfg(not(feature = "parse_timing"))]
#[macro_export]
macro_rules! time_block_ns {
    ($dst:expr, $body:block) => {{
        let _ = $dst; // évite warning
        (|| $body)()
    }};
}
