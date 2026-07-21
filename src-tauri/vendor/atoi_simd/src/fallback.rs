#![allow(dead_code)]

use crate::{
    short::{parse_short_neg, parse_short_pos},
    AtoiSimdError,
};
use ::core::convert::TryInto;
use debug_unsafe::slice::SliceGetter;

const SHORT: usize = 5;

macro_rules! overflow {
    ($curr:ident, $shift:expr, $more:ident, $max:expr) => {
        $curr >= $max / $shift && ($curr > $max / $shift || $more > $max % $shift)
    };
}

/* #[inline(always)]
fn check_len_4(val: u32) -> usize {
    ((((val & 0xF0F0_F0F0) | (((val.wrapping_add(0x0606_0606)) & 0xF0F0_F0F0) >> 4)) ^ 0x3333_3333)
        .trailing_zeros()
        >> 3) as usize // same as divide by 8 (drops extra bits from right)
} */

#[inline(always)]
fn load_8(s: &[u8]) -> u64 {
    match s.len() {
        8.. => u64::from_le_bytes(s[0..8].try_into().unwrap()),
        7 => {
            (u32::from_le_bytes(s[3..7].try_into().unwrap()) as u64) << 24
                | u32::from_le_bytes(s[0..4].try_into().unwrap()) as u64
        }
        6 => {
            (u32::from_le_bytes(s[2..6].try_into().unwrap()) as u64) << 16
                | u32::from_le_bytes(s[0..4].try_into().unwrap()) as u64
        }
        5 => (u32::from_le_bytes(s[1..5].try_into().unwrap()) as u64) << 8 | s[0] as u64,
        4 => u32::from_le_bytes(s[0..4].try_into().unwrap()) as u64,
        3 => (u16::from_le_bytes(s[1..3].try_into().unwrap()) as u64) << 8 | s[0] as u64,
        2 => u16::from_le_bytes(s[0..2].try_into().unwrap()) as u64,
        1 => s[0] as u64,
        0 => 0,
        #[allow(unreachable_patterns)]
        _ => unsafe { ::core::hint::unreachable_unchecked() }, // unreachable since 1.75
    }
}

/// val = 0x553A_3938_3736_3534; // b"456789:U"
/// val.wrapping_add(0x0606_0606_0606_0606)
/// (0x5B40_3F3E_3D3C_3B3A & 0xF0F0_F0F0_F0F0_F0F0) >> 4
/// (val & 0xF0F0_F0F0_F0F0_F0F0) | 0x0504_0303_0303_0303
/// 0x5534_3333_3333_3333 ^ 0x3333_3333_3333_3333
/// 0x6607_0000_0000_0000.trailing_zeros()
/// 48 / 8
/// 6
#[inline(always)]
fn check_len_8(val: u64) -> u32 {
    let high = (val.wrapping_add(0x0606_0606_0606_0606) & 0xF0F0_F0F0_F0F0_F0F0) >> 4;
    let low = val & 0xF0F0_F0F0_F0F0_F0F0;
    let res = (high | low) ^ 0x3333_3333_3333_3333;
    let len = res.trailing_zeros() / 8;
    unsafe { crate::assert_unchecked(len <= 8) }
    len
}

/* #[inline(always)]
fn check_len_16(val: u128) -> usize {
    ((((val & 0xF0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0)
        | (((val.wrapping_add(0x0606_0606_0606_0606_0606_0606_0606_0606))
            & 0xF0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0)
            >> 4))
        ^ 0x3333_3333_3333_3333_3333_3333_3333_3333)
        .trailing_zeros()
        >> 3) as usize // same as divide by 8 (drops extra bits from right)
} */

/* #[inline(always)]
fn process_4(mut val: u32, len: usize) -> u32 {
    val <<= 32_usize.saturating_sub(len << 3); // << 3 - same as mult by 8
    val = (val & 0x0F0F_0F0F).wrapping_mul(0xA01) >> 8;
    (val & 0x00FF_00FF).wrapping_mul(0x64_0001) >> 16
} */

#[inline(always)]
fn process_8(mut val: u64, len: u32) -> u64 {
    val <<= 64_u32.saturating_sub(len << 3); // << 3 - same as mult by 8
    val = (val & 0x0F0F_0F0F_0F0F_0F0F).wrapping_mul(0xA01) >> 8;
    val = (val & 0x00FF_00FF_00FF_00FF).wrapping_mul(0x64_0001) >> 16;
    (val & 0x0000_FFFF_0000_FFFF).wrapping_mul(0x2710_0000_0001) >> 32
}

#[inline(always)]
fn process_16(mut val: u128, len: u32) -> u64 {
    val <<= 128_u32.saturating_sub(len << 3); // << 3 - same as mult by 8
    val = (val & 0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F).wrapping_mul(0xA01) >> 8;
    val = (val & 0x00FF_00FF_00FF_00FF_00FF_00FF_00FF_00FF).wrapping_mul(0x64_0001) >> 16;
    val = (val & 0x0000_FFFF_0000_FFFF_0000_FFFF_0000_FFFF).wrapping_mul(0x2710_0000_0001) >> 32;
    ((val & 0x0000_0000_FFFF_FFFF_0000_0000_FFFF_FFFF).wrapping_mul(0x5F5_E100_0000_0000_0000_0001)
        >> 64) as u64
}

/* #[inline(always)]
fn parse_4(s: &[u8]) -> Result<(u32, usize), AtoiSimdError<'_>> {
    let val: u32 = unsafe { read_unaligned(s.as_ptr().cast()) };
    // let val: u64 = unsafe { *::core::mem::transmute_copy::<&[u8], *const u64>(&s) };
    let len = check_4(val);
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let val = process_4(val, len);
    Ok((val, len))
} */

#[inline(always)]
fn len_zeroes(val: u64) -> u32 {
    let xor = val ^ 0x30303030_30303030;
    xor.trailing_zeros() / 8
}

#[inline(always)]
fn len_zeroes_128(val: u128) -> u32 {
    let xor = val ^ 0x30303030_30303030_30303030_30303030;
    xor.trailing_zeros() / 8
}

#[inline(always)]
fn parse_8<const COUNT_ZEROES: bool>(s: &[u8]) -> Result<(u64, u32, u32), AtoiSimdError<'_>> {
    let val = load_8(s);
    let len = check_len_8(val);
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let zeroes = if COUNT_ZEROES && len == 8 {
        len_zeroes(val)
    } else {
        0
    };
    let val = process_8(val, len);

    Ok((val, len, zeroes))
}

/* #[inline(always)]
fn parse_16(s: &[u8]) -> Result<(u64, usize), AtoiSimdError<'_>> {
    let val = load_16(s);
    let len = check_len_16(val);
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let val = process_16(val, len);
    Ok((val, len))
} */

enum EarlyReturn<T, E> {
    Ok(T),
    Err(E),
    Ret(T),
}

/// len must be <= 8
#[inline(always)]
fn parse_16_by_8<const COUNT_ZEROES: bool>(
    s: &[u8],
) -> EarlyReturn<(u64, u32, u32), AtoiSimdError<'_>> {
    let mut val = load_8(s);
    let mut len = check_len_8(val);
    match len {
        0 => EarlyReturn::Err(AtoiSimdError::Empty),
        1 => EarlyReturn::Ret((val & 0xF, len, 0)),
        2..=7 => EarlyReturn::Ret((process_8(val, len), len, 0)),
        8 => {
            let val_h = load_8(s.get_safe_unchecked(8..));
            len += check_len_8(val_h);
            let val_128 = ((val_h as u128) << 64) | val as u128;
            val = process_16(val_128, len);

            // works better than `len < 16`
            if len == 16 {
                let zeroes = if COUNT_ZEROES {
                    len_zeroes_128(val_128)
                } else {
                    0
                };
                EarlyReturn::Ok((val, len, zeroes))
            } else {
                debug_assert!(len < 16);
                EarlyReturn::Ret((val, len, 0))
            }
        }
        _ => {
            if cfg!(debug_assertions) {
                unreachable!("fallback parse_16_by_8: wrong len {}", len);
            } else {
                unsafe { ::core::hint::unreachable_unchecked() }
            }
        }
    }
}

#[inline(always)]
pub(crate) fn parse_fb_pos<const MAX: u64, const SKIP_ZEROES: bool>(
    mut s: &[u8],
) -> Result<(u64, usize), AtoiSimdError<'_>> {
    debug_assert!(MAX < i64::MAX as u64);

    let mut skipped = 0;
    let (val, len) = loop {
        match parse_16_by_8::<SKIP_ZEROES>(s) {
            EarlyReturn::Ok((v, l, zeroes)) => {
                if SKIP_ZEROES && zeroes > 0 {
                    crate::cold_path();
                    skipped += zeroes;
                    s = s.get_safe_unchecked((zeroes as usize)..);
                    continue;
                }
                break (v, l);
            }
            EarlyReturn::Err(AtoiSimdError::Empty) if skipped > 0 => {
                return Ok((0, skipped as usize));
            }
            EarlyReturn::Err(e) => return Err(e),
            EarlyReturn::Ret((v, l, _)) => break (v, l),
        }
    };
    if val > MAX {
        return Err(AtoiSimdError::Overflow(s));
    }

    Ok((val, (len + skipped) as usize))
}

#[inline(always)]
pub(crate) fn parse_fb_neg<const MIN: i64, const SKIP_ZEROES: bool>(
    mut s: &[u8],
) -> Result<(i64, usize), AtoiSimdError<'_>> {
    debug_assert!(MIN > i64::MIN);
    debug_assert!(MIN < 0);

    let mut skipped = 0;
    let (val, len) = loop {
        match parse_16_by_8::<SKIP_ZEROES>(s) {
            EarlyReturn::Ok((v, l, zeroes)) => {
                if SKIP_ZEROES && zeroes > 0 {
                    crate::cold_path();
                    skipped += zeroes;
                    s = s.get_safe_unchecked((zeroes as usize)..);
                    continue;
                }
                break (v, l);
            }
            EarlyReturn::Err(AtoiSimdError::Empty) if skipped > 0 => {
                return Ok((0, skipped as usize));
            }
            EarlyReturn::Err(e) => return Err(e),
            EarlyReturn::Ret((v, l, _)) => break (v, l),
        }
    };
    let val = -(val as i64);
    if val < MIN {
        return Err(AtoiSimdError::Overflow(s));
    }

    Ok((val, (len + skipped) as usize))
}

#[inline(always)]
pub(crate) fn parse_fb_64_pos<const MAX: u64, const LEN_MORE: u32, const SKIP_ZEROES: bool>(
    mut s: &[u8],
) -> Result<(u64, usize), AtoiSimdError<'_>> {
    debug_assert!(MAX >= i64::MAX as u64);

    let mut skipped = 0;
    loop {
        let (val, len, zeroes) = match parse_16_by_8::<SKIP_ZEROES>(s) {
            EarlyReturn::Ok(v) => v,
            EarlyReturn::Err(AtoiSimdError::Empty) if skipped > 0 => {
                return Ok((0, skipped as usize));
            }
            EarlyReturn::Err(e) => return Err(e),
            EarlyReturn::Ret((v, l, _)) => return Ok((v, (l + skipped) as usize)),
        };

        let (more, len, zeroes_more) = match parse_8::<SKIP_ZEROES>(s.get_safe_unchecked(16..)) {
            Ok(v) => v,
            Err(AtoiSimdError::Empty) => return Ok((val, (len + skipped) as usize)),
            Err(e) => return Err(e),
        };

        if len > LEN_MORE {
            if SKIP_ZEROES && zeroes > 0 {
                let zeroes = if zeroes == 16 {
                    zeroes + zeroes_more
                } else {
                    zeroes
                };
                skipped += zeroes;
                s = s.get_safe_unchecked((zeroes as usize)..);
                continue;
            }
            return Err(AtoiSimdError::Size((len + 16 + skipped) as usize, s));
        }

        let shift = 10_u64.pow(len);
        if len == LEN_MORE && overflow!(val, shift, more, MAX) {
            return Err(AtoiSimdError::Overflow(s));
        }
        let res = val * shift + more;

        return Ok((res, (len + 16 + skipped) as usize));
    }
}

#[inline(always)]
pub(crate) fn parse_fb_64_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(i64, usize), AtoiSimdError<'_>> {
    let (val, len) = parse_fb_64_pos::<{ i64::MAX as u64 + 1 }, 3, SKIP_ZEROES>(s)?;
    if val > i64::MAX as u64 {
        return Ok((i64::MIN, len));
    }

    Ok((-(val as i64), len))
}

#[inline(always)]
pub(crate) fn parse_fb_128_pos<const MAX: u128, const SKIP_ZEROES: bool>(
    mut s: &[u8],
) -> Result<(u128, usize), AtoiSimdError<'_>> {
    debug_assert!(MAX >= i128::MAX as u128);

    let mut skipped = 0;
    loop {
        let (mut val, len, zeroes) = match parse_16_by_8::<SKIP_ZEROES>(s) {
            EarlyReturn::Ok((v, l, z)) => (v as u128, l, z),
            EarlyReturn::Err(AtoiSimdError::Empty) if skipped > 0 => {
                return Ok((0, skipped as usize));
            }
            EarlyReturn::Err(e) => return Err(e),
            EarlyReturn::Ret((v, l, _)) => return Ok((v as u128, (l + skipped) as usize)),
        };

        let (more, len, zeroes_more) =
            match parse_16_by_8::<SKIP_ZEROES>(s.get_safe_unchecked(16..)) {
                EarlyReturn::Ok(v) | EarlyReturn::Ret(v) => v,
                EarlyReturn::Err(AtoiSimdError::Empty) => {
                    return Ok((val, (len + skipped) as usize));
                }
                EarlyReturn::Err(e) => return Err(e),
            };
        val = val * 10_u128.pow(len) + more as u128;
        if len < 16 {
            return Ok((val, (len + 16 + skipped) as usize));
        }

        let (more, len, zeroes_more2) = match parse_8::<SKIP_ZEROES>(s.get_safe_unchecked(32..)) {
            Ok((v, l, z)) => (v as u128, l, z),
            Err(AtoiSimdError::Empty) => return Ok((val, (32 + skipped) as usize)),
            Err(e) => return Err(e),
        };
        if len > 7 {
            if SKIP_ZEROES && zeroes > 0 {
                let zeroes = if zeroes == 16 {
                    let zeroes_new = zeroes + zeroes_more;
                    if zeroes_more == 16 {
                        zeroes_new + zeroes_more2
                    } else {
                        zeroes_new
                    }
                } else {
                    zeroes
                };
                skipped += zeroes;
                s = s.get_safe_unchecked((zeroes as usize)..);
                continue;
            }
            return Err(AtoiSimdError::Size((len + 32 + skipped) as usize, s));
        } else if len == 7 && overflow!(val, 10_000_000, more, MAX) {
            return Err(AtoiSimdError::Overflow(s));
        }
        let res = val * 10_u128.pow(len) + more;

        return Ok((res, (len + 32 + skipped) as usize));
    }
}

#[inline(always)]
pub(crate) fn parse_fb_128_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(i128, usize), AtoiSimdError<'_>> {
    let (val, len) = parse_fb_128_pos::<{ i128::MAX as u128 + 1 }, SKIP_ZEROES>(s)?;
    if val > i128::MAX as u128 {
        return Ok((i128::MIN, len));
    }

    Ok((-(val as i128), len))
}

#[inline(always)]
pub(crate) fn parse_fb_checked_pos<const MAX: u64, const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<u64, AtoiSimdError<'_>> {
    let (res, len) = parse_short_pos::<MAX>(s)?;
    if len != s.len() {
        return Err(AtoiSimdError::Invalid64(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_neg<const MIN: i64, const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i64, AtoiSimdError<'_>> {
    debug_assert!(MIN < 0);

    let (res, len) = parse_short_neg::<MIN>(s)?;
    if len != s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_64_pos<
    const MAX: u64,
    const LEN_MORE: u32,
    const SKIP_ZEROES: bool,
>(
    s: &[u8],
) -> Result<u64, AtoiSimdError<'_>> {
    let s_len = s.len();
    let (res, len) = if s_len < SHORT {
        parse_short_pos::<MAX>(s)?
    } else {
        parse_fb_64_pos::<MAX, LEN_MORE, SKIP_ZEROES>(s)?
    };
    if len != s_len {
        return Err(AtoiSimdError::Invalid64(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_64_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i64, AtoiSimdError<'_>> {
    let s_len = s.len();
    let (res, len) = if s_len < SHORT {
        parse_short_neg::<{ i64::MIN }>(s)?
    } else {
        parse_fb_64_neg::<SKIP_ZEROES>(s)?
    };
    if len != s_len {
        return Err(AtoiSimdError::Invalid64(-res as u64, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_128_pos<const MAX: u128, const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<u128, AtoiSimdError<'_>> {
    let s_len = s.len();
    let (res, len) = if s_len < SHORT {
        parse_short_pos::<{ u64::MAX }>(s).map(|(v, l)| (v as u128, l))?
    } else {
        parse_fb_128_pos::<MAX, SKIP_ZEROES>(s)?
    };
    if len != s_len {
        return Err(AtoiSimdError::Invalid128(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_128_neg<const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<i128, AtoiSimdError<'_>> {
    let s_len = s.len();
    let (res, len) = if s_len < SHORT {
        parse_short_neg::<{ i64::MIN }>(s).map(|(v, l)| (v as i128, l))?
    } else {
        parse_fb_128_neg::<SKIP_ZEROES>(s)?
    };
    if len != s_len {
        return Err(AtoiSimdError::Invalid128(-res as u128, len, s));
    }
    Ok(res)
}

/* #[inline(always)]
pub(crate) fn parse_short_pos<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError<'_>> {
    let (val, len) = parse_4(s)?;
    let val = val as u64;
    if val > MAX {
        return Err(AtoiSimdError::Overflow(s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_short_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError<'_>> {
    debug_assert!(MIN < 0);
    let (val, len) = parse_4(s)?;
    let val = -(val as i64);
    if val < MIN {
        return Err(AtoiSimdError::Overflow(s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_short_checked_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError<'_>> {
    debug_assert!(MIN < 0);
    let (res, len) = parse_short_neg::<MIN>(s)?;
    if len != s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len));
    }

    Ok(res)
} */

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_check_len_8() {
        let data = [(0xFF30_3030_3030_3039, 7)];

        for (input, len) in data {
            let loaded_len = check_len_8(input);
            assert_eq!(loaded_len, len, "input: {:X?}", input);
        }
    }
}
