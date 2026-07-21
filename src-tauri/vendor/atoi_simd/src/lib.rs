//! # Fast `&[u8]` to integer parser
//!
//! SIMD (fast) parsing is supported on x86_64 (SSE4.1, AVX2) and on Arm64 (aarch64, Neon),
//! but this library works even if you don't have a SIMD supported cpu (and it will be still faster than str::parse).
//!
//! Supports negative values and validates the input.
//!
//! Supported output types: u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
//!
//! Has good test coverage, and can be considered safe.
//!
//! To enable SIMD it needs the `target-feature` or `target-cpu` flags set, or it will fallback to non-SIMD functions.
//! You can copy the `./.cargo/config.toml` to your project, or use one of the following environment variables:
//!
//! -   `RUSTFLAGS="-C target-feature=+sse2,+sse3,+sse4.1,+ssse3,+avx,+avx2"` for x86_64;
//!
//! -   `RUSTFLAGS="-C target-feature=+neon"` for Arm64;
//!
//! -   `RUSTFLAGS="-C target-cpu=native"` will optimize for your current cpu.
//!
//! If you have `&str` then use `.as_bytes()`
//!
//! Supports `no_std` with `--no-default-features`
//!
//! # Examples
//!
//! ```
//! // a drop-in replacement for `str::parse`
//! assert_eq!(atoi_simd::parse::<u64, true, true>(b"+000000000000000000001234"), Ok(1234_u64));
//!
//! let val: u64 = atoi_simd::parse::<_, false, false>(b"1234").unwrap();
//! assert_eq!(val, 1234_u64);
//!
//! assert_eq!(atoi_simd::parse::<i64, false, false>(b"-2345"), Ok(-2345_i64));
//!
//! assert_eq!(atoi_simd::parse_neg::<i64, false>(b"2345"), Ok(-2345_i64));
//!
//! assert_eq!(atoi_simd::parse_prefix::<u64, false, false>(b"123something_else"), Ok((123_u64, 3)));
//! ```
#![allow(clippy::comparison_chain)]
#![cfg_attr(not(feature = "std"), no_std)]
// #![feature(stdsimd)]

#[allow(unused)]
#[inline(always)]
#[cold]
pub(crate) fn cold_path() {}

#[allow(unused)]
#[inline(always)]
#[rustversion::since(1.81.0)]
pub(crate) const unsafe fn assert_unchecked(cond: bool) {
    unsafe { ::core::hint::assert_unchecked(cond) }
}

#[allow(unused)]
#[inline(always)]
#[rustversion::before(1.81.0)]
pub(crate) const unsafe fn assert_unchecked(cond: bool) {}

mod error;
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
mod fallback;
mod linker;
mod short;
#[cfg(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    ),
))]
mod simd;

pub use crate::{
    error::AtoiSimdError,
    linker::{Parse, ParseNeg},
};

/// Parses a slice of digits, and checks for the first '-' char for signed integers.
///
/// Can skip the '+' char (SKIP_PLUS) and extra zeroes (more than an integer's max length) at the beginning (SKIP_ZEROES),
/// but it's a bit slower.
/// Even without SKIP_ZEROES it parses zeroes at the beginning, but up to an integer's max length (even a bit more).
/// SKIP_ZEROES makes it skip an infinite amount of zeroes.
#[inline]
pub fn parse<T: Parse, const SKIP_ZEROES: bool, const SKIP_PLUS: bool>(
    s: &[u8],
) -> Result<T, AtoiSimdError<'_>> {
    T::atoi_simd_parse::<SKIP_ZEROES, SKIP_PLUS>(s)
}

/// Parses a positive integer. Does not check any signs, so slice must not contain them.
///
/// Can skip extra zeroes (more than an integer's max length) at the beginning (SKIP_ZEROES), but it's a bit slower.
/// Even without SKIP_ZEROES it parses zeroes at the beginning, but up to an integer's max length (even a bit more).
/// SKIP_ZEROES makes it skip an infinite amount of zeroes.
#[inline]
pub fn parse_pos<T: Parse, const SKIP_ZEROES: bool>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    T::atoi_simd_parse_pos::<SKIP_ZEROES>(s)
}

/// Parses a negative integer. Does not check any signs, so slice must not contain them (including '-' sign).
///
/// Can skip extra zeroes (more than an integer's max length) at the beginning (SKIP_ZEROES), but it's a bit slower.
/// Even without SKIP_ZEROES it parses zeroes at the beginning, but up to an integer's max length (even a bit more).
/// SKIP_ZEROES makes it skip an infinite amount of zeroes.
#[inline]
pub fn parse_neg<T: ParseNeg, const SKIP_ZEROES: bool>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    T::atoi_simd_parse_neg::<SKIP_ZEROES>(s)
}

/// Parses a slice of digits until it reaches an invalid character,
/// and checks for the first '-' char for signed integers.
/// Returns the parsed value and the parsed size of the slice.
///
/// Can skip the '+' char (SKIP_PLUS) and extra zeroes (more than an integer's max length) at the beginning (SKIP_ZEROES),
/// but it's a bit slower.
/// Even without SKIP_ZEROES it parses zeroes at the beginning, but up to an integer's max length (even a bit more).
/// SKIP_ZEROES makes it skip an infinite amount of zeroes.
#[inline]
pub fn parse_prefix<T: Parse, const SKIP_ZEROES: bool, const SKIP_PLUS: bool>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError<'_>> {
    T::atoi_simd_parse_prefix::<SKIP_ZEROES, SKIP_PLUS>(s)
}

/// Parses a positive integer until it reaches an invalid character.
/// Returns the parsed value and the parsed size of the slice.
/// Does not check any signs, so slice must not contain them.
///
/// Can skip extra zeroes (more than an integer's max length) at the beginning (SKIP_ZEROES), but it's a bit slower.
/// Even without SKIP_ZEROES it parses zeroes at the beginning, but up to an integer's max length (even a bit more).
/// SKIP_ZEROES makes it skip an infinite amount of zeroes.
#[inline]
pub fn parse_prefix_pos<T: Parse, const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError<'_>> {
    T::atoi_simd_parse_prefix_pos::<SKIP_ZEROES>(s)
}

/// Parses a negative integer until it reaches an invalid character.
/// Returns the parsed value and the parsed size of the slice.
/// Does not check any signs, so slice must not contain them (including '-' sign).
///
/// Can skip extra zeroes (more than an integer's max length) at the beginning (SKIP_ZEROES), but it's a bit slower.
/// Even without SKIP_ZEROES it parses zeroes at the beginning, but up to an integer's max length (even a bit more).
/// SKIP_ZEROES makes it skip an infinite amount of zeroes.
#[inline]
pub fn parse_prefix_neg<T: ParseNeg, const SKIP_ZEROES: bool>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError<'_>> {
    T::atoi_simd_parse_prefix_neg::<SKIP_ZEROES>(s)
}

#[deprecated(since = "0.18.0", note = "Use `parse::<_, true, true>` instead")]
#[inline]
pub fn parse_skipped<T: Parse>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    parse::<_, true, true>(s)
}

#[deprecated(since = "0.17.0", note = "Use `parse_prefix` instead")]
#[inline]
pub fn parse_any<T: Parse>(s: &[u8]) -> Result<(T, usize), AtoiSimdError<'_>> {
    parse_prefix::<_, false, false>(s)
}

#[deprecated(since = "0.17.0", note = "Use `parse_prefix_pos` instead")]
#[inline]
pub fn parse_any_pos<T: Parse>(s: &[u8]) -> Result<(T, usize), AtoiSimdError<'_>> {
    parse_prefix_pos::<_, false>(s)
}

#[deprecated(since = "0.17.0", note = "Use `parse_prefix_neg` instead")]
#[inline]
pub fn parse_any_neg<T: ParseNeg>(s: &[u8]) -> Result<(T, usize), AtoiSimdError<'_>> {
    parse_prefix_neg::<_, false>(s)
}
