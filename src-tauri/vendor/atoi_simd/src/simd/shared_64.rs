#![allow(dead_code)] // used when you don't have avx

use crate::{short::parse_short_pos, AtoiSimdError};

pub(crate) use super::parse_simd_u128;

#[inline(always)]
pub(crate) fn parse_simd_checked_u128<const LEN_LIMIT: u32, const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<u128, AtoiSimdError<'_>> {
    let s_len = s.len();
    let (res, len) = if s_len < super::SHORT {
        parse_short_pos::<{ u64::MAX }>(s).map(|(v, l)| (v as u128, l))?
    } else if s_len < 17 {
        super::parse_simd_16_noskip(s).map(|(v, l)| (v as u128, l))?
    } else {
        parse_simd_u128::<LEN_LIMIT, SKIP_ZEROES>(s)?
    };
    if len != s_len {
        return Err(AtoiSimdError::Invalid128(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_simd_i128<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(i128, usize), AtoiSimdError<'_>> {
    let (res, len) = parse_simd_u128::<39, SKIP_ZEROES>(s)?;
    if res > i128::MAX as u128 {
        Err(AtoiSimdError::Overflow(s))
    } else {
        Ok((res as i128, len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_i128<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i128, AtoiSimdError<'_>> {
    let res = parse_simd_checked_u128::<39, SKIP_ZEROES>(s)?;
    if res > i128::MAX as u128 {
        Err(AtoiSimdError::Overflow(s))
    } else {
        Ok(res as i128)
    }
}

#[inline(always)]
pub(crate) fn parse_simd_u64<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(u64, usize), AtoiSimdError<'_>> {
    let (res, len) = parse_simd_u128::<20, SKIP_ZEROES>(s)?;
    if res > u64::MAX as u128 {
        crate::cold_path();
        Err(AtoiSimdError::Overflow(s))
    } else {
        Ok((res as u64, len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_u64<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<u64, AtoiSimdError<'_>> {
    let res = parse_simd_checked_u128::<20, SKIP_ZEROES>(s)?;
    if res > u64::MAX as u128 {
        crate::cold_path();
        Err(AtoiSimdError::Overflow(s))
    } else {
        Ok(res as u64)
    }
}

#[inline(always)]
pub(crate) fn parse_simd_i64<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(i64, usize), AtoiSimdError<'_>> {
    let (res, len) = parse_simd_u128::<19, SKIP_ZEROES>(s)?;
    if res > i64::MAX as u128 {
        crate::cold_path();
        Err(AtoiSimdError::Overflow(s))
    } else {
        Ok((res as i64, len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_i64<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i64, AtoiSimdError<'_>> {
    let res = parse_simd_checked_u128::<19, SKIP_ZEROES>(s)?;
    if res > i64::MAX as u128 {
        crate::cold_path();
        Err(AtoiSimdError::Overflow(s))
    } else {
        Ok(res as i64)
    }
}

#[inline(always)]
pub(crate) fn parse_simd_i64_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(i64, usize), AtoiSimdError<'_>> {
    let (res, len) = parse_simd_u128::<19, SKIP_ZEROES>(s)?;
    const MAX: u128 = -(i64::MIN as i128) as u128;
    if res > MAX {
        Err(AtoiSimdError::Overflow(s))
    } else if res == MAX {
        Ok((i64::MIN, len))
    } else {
        Ok((-(res as i64), len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_i64_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i64, AtoiSimdError<'_>> {
    let res = parse_simd_checked_u128::<19, SKIP_ZEROES>(s)?;
    const MAX: u128 = -(i64::MIN as i128) as u128;
    if res > MAX {
        Err(AtoiSimdError::Overflow(s))
    } else if res == MAX {
        Ok(i64::MIN)
    } else {
        Ok(-(res as i64))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_i128_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(i128, usize), AtoiSimdError<'_>> {
    let (res, len) = parse_simd_u128::<39, SKIP_ZEROES>(s)?;
    const MAX: u128 = i128::MAX as u128 + 1;
    if res > MAX {
        Err(AtoiSimdError::Overflow(s))
    } else if res == MAX {
        Ok((i128::MIN, len))
    } else {
        Ok((-(res as i128), len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_i128_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i128, AtoiSimdError<'_>> {
    let res = parse_simd_checked_u128::<39, SKIP_ZEROES>(s)?;
    const MAX: u128 = i128::MAX as u128 + 1;
    if res > MAX {
        Err(AtoiSimdError::Overflow(s))
    } else if res == MAX {
        Ok(i128::MIN)
    } else {
        Ok(-(res as i128))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_corner_cases() {
        assert_eq!(
            parse_simd_checked_u128::<32, true>(b"12345678901234567890123456789012").unwrap(),
            12345678901234567890123456789012
        );

        assert_eq!(
            parse_simd_u128::<32, true>(b"12345678901234567890123456789012s").unwrap(),
            (12345678901234567890123456789012, 32)
        );

        assert_eq!(
            parse_simd_checked_u128::<32, true>(b"123456789012345678901234567890123").unwrap(),
            123456789012345678901234567890123
        );
    }
}
