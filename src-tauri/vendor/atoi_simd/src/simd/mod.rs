#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod neon;
#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3"
))]
mod sse_avx;

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(crate) use neon::*;
#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse",
    target_feature = "sse4.1",
    target_feature = "ssse3"
))]
pub(crate) use sse_avx::*;

pub(crate) mod shared_32;
pub(crate) mod shared_64;

use crate::AtoiSimdError;

#[inline(always)]
pub(crate) fn process_skipped(
    res: Result<(u64, usize), AtoiSimdError<'_>>,
    skipped: u32,
) -> Result<(u64, usize), AtoiSimdError<'_>> {
    if skipped > 0 {
        if matches!(res, Err(AtoiSimdError::Empty)) {
            Ok((0, skipped as usize))
        } else {
            res.map(|(v, l)| (v, l + skipped as usize))
        }
    } else {
        res
    }
}
