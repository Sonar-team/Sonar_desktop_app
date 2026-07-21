use super::*;
use crate::simd::shared_64::*;

#[cfg(target_pointer_width = "64")]
impl ParsePos for usize {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<usize, AtoiSimdError<'_>> {
        parse_simd_checked_u64::<SKIP_ZEROES>(s).map(|v| v as usize)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(usize, usize), AtoiSimdError<'_>> {
        parse_simd_u64::<SKIP_ZEROES>(s).map(|(v, i)| (v as usize, i))
    }
}

#[cfg(target_pointer_width = "64")]
impl ParsePos for isize {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<isize, AtoiSimdError<'_>> {
        parse_simd_checked_i64::<SKIP_ZEROES>(s).map(|v| v as isize)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(isize, usize), AtoiSimdError<'_>> {
        parse_simd_i64::<SKIP_ZEROES>(s).map(|(v, i)| (v as isize, i))
    }
}

#[cfg(target_pointer_width = "64")]
impl ParseNeg for isize {
    #[inline(always)]
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<isize, AtoiSimdError<'_>> {
        parse_simd_checked_i64_neg::<SKIP_ZEROES>(s).map(|v| v as isize)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(isize, usize), AtoiSimdError<'_>> {
        parse_simd_i64_neg::<SKIP_ZEROES>(s).map(|(v, i)| (v as isize, i))
    }
}

impl ParsePos for u64 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<u64, AtoiSimdError<'_>> {
        parse_simd_checked_u64::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(u64, usize), AtoiSimdError<'_>> {
        parse_simd_u64::<SKIP_ZEROES>(s)
    }
}

impl ParsePos for i64 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i64, AtoiSimdError<'_>> {
        parse_simd_checked_i64::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i64, usize), AtoiSimdError<'_>> {
        parse_simd_i64::<SKIP_ZEROES>(s)
    }
}

impl ParseNeg for i64 {
    #[inline(always)]
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i64, AtoiSimdError<'_>> {
        parse_simd_checked_i64_neg::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i64, usize), AtoiSimdError<'_>> {
        parse_simd_i64_neg::<SKIP_ZEROES>(s)
    }
}

impl ParsePos for u128 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<u128, AtoiSimdError<'_>> {
        parse_simd_checked_u128::<39, SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(u128, usize), AtoiSimdError<'_>> {
        parse_simd_u128::<39, SKIP_ZEROES>(s)
    }
}

impl ParsePos for i128 {
    #[inline(always)]
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i128, AtoiSimdError<'_>> {
        parse_simd_checked_i128::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i128, usize), AtoiSimdError<'_>> {
        parse_simd_i128::<SKIP_ZEROES>(s)
    }
}

impl ParseNeg for i128 {
    #[inline(always)]
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<i128, AtoiSimdError<'_>> {
        parse_simd_checked_i128_neg::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(i128, usize), AtoiSimdError<'_>> {
        parse_simd_i128_neg::<SKIP_ZEROES>(s)
    }
}
