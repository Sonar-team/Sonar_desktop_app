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
        let out = $body;
        *$dst = $crate::timing::elapsed_ns(t0);
        out
    }};
}

#[cfg(not(feature = "parse_timing"))]
#[macro_export]
macro_rules! time_block_ns {
    ($dst:expr, $body:block) => {{
        let _ = $dst;
        $body
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_parse_timing_default() {
        let timing = ParseTiming::default();

        assert_eq!(timing.l2_ns, 0);
        assert_eq!(timing.l3_ns, 0);
        assert_eq!(timing.l4_ns, 0);
        assert_eq!(timing.l7_ns, 0);
        assert_eq!(timing.total_ns, 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_layer_attempt_default_is_skipped() {
        let attempt = LayerAttempt::default();
        assert_eq!(attempt, LayerAttempt::Skipped);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_layer_attempt_variants_values() {
        assert_eq!(LayerAttempt::Skipped as u8, 0);
        assert_eq!(LayerAttempt::Ok as u8, 1);
        assert_eq!(LayerAttempt::Unsupported as u8, 2);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_parse_report_default() {
        let report = ParseReport::default();

        assert_eq!(report.timing.l2_ns, 0);
        assert_eq!(report.timing.l3_ns, 0);
        assert_eq!(report.timing.l4_ns, 0);
        assert_eq!(report.timing.l7_ns, 0);
        assert_eq!(report.timing.total_ns, 0);

        assert_eq!(report.l3, LayerAttempt::Skipped);
        assert_eq!(report.l4, LayerAttempt::Skipped);
        assert_eq!(report.l7, LayerAttempt::Skipped);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_now_and_elapsed_ns() {
        let t0 = now();
        let elapsed = elapsed_ns(t0);

        assert!(elapsed <= u64::MAX);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_elapsed_ns_is_non_zero_after_work() {
        let t0 = now();

        let mut acc = 0u64;
        for i in 0..10_000 {
            acc = acc.wrapping_add(i);
        }

        let elapsed = elapsed_ns(t0);

        assert!(acc > 0);
        assert!(elapsed > 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_time_block_ns_sets_duration_and_returns_value() {
        let mut measured = 0u64;

        let result = time_block_ns!(&mut measured, {
            let mut sum = 0u64;
            for i in 0..1_000 {
                sum += i;
            }
            sum
        });

        assert_eq!(result, (0..1_000u64).sum::<u64>());
        assert!(measured > 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn test_time_block_ns_with_unit_return() {
        let mut measured = 0u64;
        let mut value = 0u32;

        time_block_ns!(&mut measured, {
            value = 42;
        });

        assert_eq!(value, 42);
        assert!(measured > 0);
    }

    #[cfg(not(feature = "parse_timing"))]
    #[test]
    fn test_parse_timing_default_without_feature() {
        let _timing = ParseTiming;
    }

    #[cfg(not(feature = "parse_timing"))]
    #[test]
    fn test_now_without_feature() {
        now();
        let elapsed = elapsed_ns(());

        assert_eq!(elapsed, 0);
    }

    #[cfg(not(feature = "parse_timing"))]
    #[test]
    fn test_time_block_ns_without_feature_returns_value_and_does_not_modify_dst() {
        let mut measured = 123u64;

        let result = time_block_ns!(&mut measured, {
            let mut sum = 0u64;
            for i in 0..10 {
                sum += i;
            }
            sum
        });

        assert_eq!(result, 45);
        assert_eq!(measured, 123);
    }

    #[cfg(not(feature = "parse_timing"))]
    #[test]
    fn test_time_block_ns_without_feature_with_unit_return() {
        let mut measured = 999u64;
        let value: u32;

        time_block_ns!(&mut measured, {
            value = 7;
        });

        assert_eq!(value, 7);
        assert_eq!(measured, 999);
    }
}
