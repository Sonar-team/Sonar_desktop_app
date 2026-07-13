// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::str;

use crate::errors::application::giop::GiopParseError;

pub const GIOP_HEADER_LEN: usize = 12;
pub const GIOP_MAGIC: &[u8; 4] = b"GIOP";

/// Minimum encoded size of one service context entry:
/// context_id (u32) + context_data length (u32).
pub const SERVICE_CONTEXT_MIN_LEN: usize = 8;

/// Validates that the buffer is large enough to hold a GIOP header.
pub fn ensure_min_len(payload: &[u8]) -> Result<(), GiopParseError> {
    if payload.len() < GIOP_HEADER_LEN {
        return Err(GiopParseError::InvalidSize);
    }

    Ok(())
}

/// Extracts and validates the "GIOP" magic bytes.
///
/// The caller must have validated that the buffer holds at least
/// [`GIOP_HEADER_LEN`] bytes (see [`ensure_min_len`]).
pub fn parse_magic(payload: &[u8]) -> Result<[u8; 4], GiopParseError> {
    if payload.len() < GIOP_MAGIC.len() {
        return Err(GiopParseError::InvalidSize);
    }

    let magic = [payload[0], payload[1], payload[2], payload[3]];
    if &magic != GIOP_MAGIC {
        return Err(GiopParseError::InvalidMagic);
    }

    Ok(magic)
}

/// Validates the GIOP version. Only versions 1.0, 1.1 and 1.2 are supported.
pub fn validate_version(major_version: u8, minor_version: u8) -> Result<(), GiopParseError> {
    if major_version != 1 || minor_version > 2 {
        return Err(GiopParseError::UnsupportedVersion(
            major_version,
            minor_version,
        ));
    }

    Ok(())
}

/// Validates that the buffer holds the full message announced by the header
/// (header length + message_length).
pub fn validate_total_length(total_needed: usize, actual: usize) -> Result<(), GiopParseError> {
    if actual < total_needed {
        return Err(GiopParseError::TruncatedBody {
            expected: total_needed,
            actual,
        });
    }

    Ok(())
}

/// Validates that a CDR read of `needed` bytes fits in the `remaining` bytes
/// of the body.
pub fn ensure_available(remaining: usize, needed: usize) -> Result<(), GiopParseError> {
    if remaining < needed {
        return Err(GiopParseError::UnexpectedEof);
    }

    Ok(())
}

/// Validates the TargetAddress discriminator (0 = KeyAddr, 1 = ProfileAddr,
/// 2 = ReferenceAddr).
pub fn validate_target_discriminator(discriminator: u8) -> Result<(), GiopParseError> {
    if discriminator > 2 {
        return Err(GiopParseError::UnknownTargetDiscriminator(discriminator));
    }

    Ok(())
}

/// Coherence check on the announced service context count: each entry needs
/// at least [`SERVICE_CONTEXT_MIN_LEN`] bytes, so the count cannot exceed
/// `remaining / SERVICE_CONTEXT_MIN_LEN`. This bounds allocations against
/// forged counts before any entry is read.
pub fn validate_service_context_count(
    count: usize,
    remaining: usize,
) -> Result<(), GiopParseError> {
    if count > remaining / SERVICE_CONTEXT_MIN_LEN {
        return Err(GiopParseError::InvalidServiceContextCount {
            count,
            available: remaining,
        });
    }

    Ok(())
}

/// Validates a CDR string body as UTF-8, trimming the trailing NUL terminator
/// if present, and returns it as a borrowed `&str` (zero-copy).
pub fn parse_cdr_string(bytes: &[u8]) -> Result<&str, GiopParseError> {
    let content = match bytes.split_last() {
        Some((0, rest)) => rest,
        _ => bytes,
    };

    str::from_utf8(content).map_err(|_| GiopParseError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_min_len() {
        assert!(ensure_min_len(&[0u8; GIOP_HEADER_LEN]).is_ok());
        assert!(matches!(
            ensure_min_len(&[0u8; GIOP_HEADER_LEN - 1]),
            Err(GiopParseError::InvalidSize)
        ));
    }

    #[test]
    fn test_parse_magic() {
        assert_eq!(parse_magic(b"GIOP\x01\x02"), Ok(*b"GIOP"));
        assert!(matches!(
            parse_magic(b"NOPE"),
            Err(GiopParseError::InvalidMagic)
        ));
        assert!(matches!(
            parse_magic(b"GI"),
            Err(GiopParseError::InvalidSize)
        ));
    }

    #[test]
    fn test_validate_version() {
        assert!(validate_version(1, 0).is_ok());
        assert!(validate_version(1, 2).is_ok());
        assert!(matches!(
            validate_version(2, 0),
            Err(GiopParseError::UnsupportedVersion(2, 0))
        ));
        assert!(matches!(
            validate_version(1, 3),
            Err(GiopParseError::UnsupportedVersion(1, 3))
        ));
    }

    #[test]
    fn test_validate_total_length() {
        assert!(validate_total_length(12, 12).is_ok());
        assert!(validate_total_length(12, 20).is_ok());
        assert!(matches!(
            validate_total_length(22, 12),
            Err(GiopParseError::TruncatedBody {
                expected: 22,
                actual: 12
            })
        ));
    }

    #[test]
    fn test_ensure_available() {
        assert!(ensure_available(4, 4).is_ok());
        assert!(ensure_available(8, 4).is_ok());
        assert!(matches!(
            ensure_available(3, 4),
            Err(GiopParseError::UnexpectedEof)
        ));
    }

    #[test]
    fn test_validate_target_discriminator() {
        assert!(validate_target_discriminator(0).is_ok());
        assert!(validate_target_discriminator(1).is_ok());
        assert!(validate_target_discriminator(2).is_ok());
        assert!(matches!(
            validate_target_discriminator(9),
            Err(GiopParseError::UnknownTargetDiscriminator(9))
        ));
    }

    #[test]
    fn test_validate_service_context_count() {
        assert!(validate_service_context_count(0, 0).is_ok());
        assert!(validate_service_context_count(2, 16).is_ok());
        assert!(matches!(
            validate_service_context_count(3, 16),
            Err(GiopParseError::InvalidServiceContextCount {
                count: 3,
                available: 16
            })
        ));
        // Forged huge count must be rejected before any allocation.
        assert!(matches!(
            validate_service_context_count(usize::MAX, 8),
            Err(GiopParseError::InvalidServiceContextCount { .. })
        ));
    }

    #[test]
    fn test_parse_cdr_string() {
        assert_eq!(parse_cdr_string(b"op\0"), Ok("op"));
        assert_eq!(parse_cdr_string(b"op"), Ok("op"));
        assert_eq!(parse_cdr_string(b"\0"), Ok(""));
        assert_eq!(parse_cdr_string(b""), Ok(""));
        assert!(matches!(
            parse_cdr_string(&[0xFF, 0xFE]),
            Err(GiopParseError::InvalidUtf8)
        ));
    }
}
