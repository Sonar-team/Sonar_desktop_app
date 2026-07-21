#[cfg(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    ),
))]
mod simd_32;
#[cfg(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ),
))]
mod simd_64;

#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    ),
)))]
mod fb_32;
#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ),
)))]
mod fb_64;

use crate::AtoiSimdError;
use debug_unsafe::slice::SliceGetter;

/// Note: all of the provided methods are `#[inline(always)]`
pub trait ParsePos: Sized {
    fn atoi_simd_parse_pos<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<Self, AtoiSimdError<'_>>;
    fn atoi_simd_parse_prefix_pos<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(Self, usize), AtoiSimdError<'_>>;
}

/// Note: all of the provided methods are `#[inline(always)]`
pub trait ParseNeg: Sized {
    fn atoi_simd_parse_neg<const SKIP_ZEROES: bool>(s: &[u8]) -> Result<Self, AtoiSimdError<'_>>;
    fn atoi_simd_parse_prefix_neg<const SKIP_ZEROES: bool>(
        s: &[u8],
    ) -> Result<(Self, usize), AtoiSimdError<'_>>;
}

/// Note: all of the provided methods are `#[inline(always)]`
pub trait Parse: ParsePos {
    #[inline(always)]
    fn atoi_simd_parse<const SKIP_ZEROES: bool, const SKIP_PLUS: bool>(
        mut s: &[u8],
    ) -> Result<Self, AtoiSimdError<'_>> {
        if SKIP_PLUS && *s.first().ok_or(AtoiSimdError::Empty)? == b'+' {
            s = s.get_safe_unchecked(1..);
        }

        Self::atoi_simd_parse_pos::<SKIP_ZEROES>(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_prefix<const SKIP_ZEROES: bool, const SKIP_PLUS: bool>(
        mut s: &[u8],
    ) -> Result<(Self, usize), AtoiSimdError<'_>> {
        if SKIP_PLUS && *s.first().ok_or(AtoiSimdError::Empty)? == b'+' {
            s = s.get_safe_unchecked(1..);
        }

        Self::atoi_simd_parse_prefix_pos::<SKIP_ZEROES>(s)
    }
}

#[inline(always)]
fn atoi_simd_parse_signed<
    T: ParsePos + ParseNeg,
    const SKIP_ZEROES: bool,
    const SKIP_PLUS: bool,
>(
    mut s: &[u8],
) -> Result<T, AtoiSimdError<'_>> {
    let neg = match *s.first().ok_or(AtoiSimdError::Empty)? {
        b'+' if SKIP_PLUS => {
            s = s.get_safe_unchecked(1..);
            false
        }
        b'-' => {
            s = s.get_safe_unchecked(1..);
            true
        }
        _ => false,
    };

    if neg {
        T::atoi_simd_parse_neg::<SKIP_ZEROES>(s)
    } else {
        T::atoi_simd_parse_pos::<SKIP_ZEROES>(s)
    }
}

#[inline(always)]
fn atoi_simd_parse_prefix_signed<
    T: ParsePos + ParseNeg,
    const SKIP_ZEROES: bool,
    const SKIP_PLUS: bool,
>(
    mut s: &[u8],
) -> Result<(T, usize), AtoiSimdError<'_>> {
    let neg = match *s.first().ok_or(AtoiSimdError::Empty)? {
        b'+' if SKIP_PLUS => {
            s = s.get_safe_unchecked(1..);
            false
        }
        b'-' => {
            s = s.get_safe_unchecked(1..);
            true
        }
        _ => false,
    };

    if neg {
        T::atoi_simd_parse_prefix_neg::<SKIP_ZEROES>(s).map(|(v, l)| (v, l + 1))
    } else {
        T::atoi_simd_parse_prefix_pos::<SKIP_ZEROES>(s)
    }
}

impl Parse for u8 {}
impl Parse for u16 {}
impl Parse for u32 {}
impl Parse for usize {}
impl Parse for u64 {}
impl Parse for u128 {}

macro_rules! parse_impl_signed {
    ($($t:ty)*) => {$(
        impl Parse for $t {
            #[inline(always)]
            fn atoi_simd_parse<const SKIP_ZEROES: bool, const SKIP_PLUS: bool>(s: &[u8]) -> Result<Self, AtoiSimdError<'_>> {
                atoi_simd_parse_signed::<_, SKIP_ZEROES, SKIP_PLUS>(s)
            }

            #[inline(always)]
            fn atoi_simd_parse_prefix<const SKIP_ZEROES: bool, const SKIP_PLUS: bool>(s: &[u8]) -> Result<(Self, usize), AtoiSimdError<'_>> {
                atoi_simd_parse_prefix_signed::<_, SKIP_ZEROES, SKIP_PLUS>(s)
            }
        }
    )*};
}

parse_impl_signed!(i8 i16 i32 isize i64 i128);
