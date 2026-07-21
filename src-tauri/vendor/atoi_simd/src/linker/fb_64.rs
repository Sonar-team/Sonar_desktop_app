use super::*;
use crate::fallback::*;

#[cfg(target_pointer_width = "64")]
impl ParsePos for usize {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<usize, AtoiSimdError<'_>> {
        parse_fb_checked_64_pos::<{ u64::MAX }, 4, SKIP_ZEROES>(s).map(|v| v as usize)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(usize, usize), AtoiSimdError<'_>> {
        parse_fb_64_pos::<{ u64::MAX }, 4, SKIP_ZEROES>(s).map(|(v, i)| (v as usize, i))
    }
}

#[cfg(target_pointer_width = "64")]
impl ParsePos for isize {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<isize, AtoiSimdError<'_>> {
        parse_fb_checked_64_pos::<{ i64::MAX as u64 }, 3, SKIP_ZEROES>(s).map(|v| v as isize)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(isize, usize), AtoiSimdError<'_>> {
        parse_fb_64_pos::<{ i64::MAX as u64 }, 3, SKIP_ZEROES>(s).map(|(v, i)| (v as isize, i))
    }
}

#[cfg(target_pointer_width = "64")]
impl ParseNeg for isize {
    #[inline(always)]
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<isize, AtoiSimdError<'_>> {
        parse_fb_checked_64_neg::<SKIP_ZEROES>(s).map(|v| v as isize)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(isize, usize), AtoiSimdError<'_>> {
        parse_fb_64_neg::<SKIP_ZEROES>(s).map(|(v, i)| (v as isize, i))
    }
}

impl ParsePos for u64 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<u64, AtoiSimdError<'_>> {
        parse_fb_checked_64_pos::<{ u64::MAX }, 4, SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(u64, usize), AtoiSimdError<'_>> {
        parse_fb_64_pos::<{ u64::MAX }, 4, SKIP_ZEROES>(s)
    }
}

impl ParsePos for i64 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i64, AtoiSimdError<'_>> {
        parse_fb_checked_64_pos::<{ i64::MAX as u64 }, 3, SKIP_ZEROES>(s).map(|v| v as i64)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i64, usize), AtoiSimdError<'_>> {
        parse_fb_64_pos::<{ i64::MAX as u64 }, 3, SKIP_ZEROES>(s).map(|(v, i)| (v as i64, i))
    }
}

impl ParseNeg for i64 {
    #[inline(always)]
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i64, AtoiSimdError<'_>> {
        parse_fb_checked_64_neg::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i64, usize), AtoiSimdError<'_>> {
        parse_fb_64_neg::<SKIP_ZEROES>(s)
    }
}

impl ParsePos for u128 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<u128, AtoiSimdError<'_>> {
        parse_fb_checked_128_pos::<{ u128::MAX }, SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(u128, usize), AtoiSimdError<'_>> {
        parse_fb_128_pos::<{ u128::MAX }, SKIP_ZEROES>(s)
    }
}

impl ParsePos for i128 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i128, AtoiSimdError<'_>> {
        parse_fb_checked_128_pos::<{ i128::MAX as u128 }, SKIP_ZEROES>(s).map(|v| v as i128)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i128, usize), AtoiSimdError<'_>> {
        parse_fb_128_pos::<{ i128::MAX as u128 }, SKIP_ZEROES>(s).map(|(v, i)| (v as i128, i))
    }
}

impl ParseNeg for i128 {
    #[inline(always)]
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i128, AtoiSimdError<'_>> {
        parse_fb_checked_128_neg::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i128, usize), AtoiSimdError<'_>> {
        parse_fb_128_neg::<SKIP_ZEROES>(s)
    }
}
