// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::srvloc::SrvlocPacketParseError;

pub fn validate_packet_not_empty(payload: &[u8]) -> Result<(), SrvlocPacketParseError> {
    if payload.is_empty() {
        return Err(SrvlocPacketParseError::InvalidPacketLength);
    }

    Ok(())
}

pub fn ensure_len(buf: &[u8], needed: usize) -> Result<(), SrvlocPacketParseError> {
    if buf.len() < needed {
        Err(SrvlocPacketParseError::Truncated {
            expected_at_least: needed,
            actual: buf.len(),
        })
    } else {
        Ok(())
    }
}

/// Highest function code defined by SLPv1 (RFC 2165) : SrvTypeRply = 10.
pub const SRVLOC_V1_MAX_FUNCTION: u8 = 10;
/// Highest function code defined by SLPv2 (RFC 2608) : SAAdvert = 11.
pub const SRVLOC_V2_MAX_FUNCTION: u8 = 11;

/// Validates that the length declared in the SRVLOC header matches the actual
/// payload length. This is the strongest guard against foreign payloads
/// (e.g. DHCP/BOOTP, whose first bytes mimic an SLP header) being
/// misclassified as SRVLOC.
pub fn validate_declared_packet_length(
    declared: usize,
    actual: usize,
) -> Result<(), SrvlocPacketParseError> {
    if declared != actual {
        return Err(SrvlocPacketParseError::InconsistentPacketLength { declared, actual });
    }
    Ok(())
}

/// Validates that the function code exists for the given SLP version
/// (RFC 2165 for v1, RFC 2608 for v2). Function 0 is never valid.
pub fn validate_function(version: u8, function: u8) -> Result<(), SrvlocPacketParseError> {
    let max = match version {
        1 => SRVLOC_V1_MAX_FUNCTION,
        2 => SRVLOC_V2_MAX_FUNCTION,
        other => return Err(SrvlocPacketParseError::UnsupportedVersion(other)),
    };
    if function == 0 || function > max {
        return Err(SrvlocPacketParseError::UnsupportedFunction { version, function });
    }
    Ok(())
}

/// Validates that `slice` is valid UTF-8 and returns it borrowed as `&str`.
///
/// Zero-copy: no allocation, the returned `&str` points into the original packet.
pub fn validate_utf8<'a>(
    slice: &'a [u8],
    field: &'static str,
) -> Result<&'a str, SrvlocPacketParseError> {
    core::str::from_utf8(slice).map_err(|_| SrvlocPacketParseError::InvalidUtf8(field))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_packet_not_empty_ok() {
        assert!(validate_packet_not_empty(&[1u8]).is_ok());
    }

    #[test]
    fn test_validate_packet_not_empty_err() {
        assert!(matches!(
            validate_packet_not_empty(&[]),
            Err(SrvlocPacketParseError::InvalidPacketLength)
        ));
    }

    #[test]
    fn test_ensure_len_ok() {
        assert!(ensure_len(&[0u8; 4], 4).is_ok());
    }

    #[test]
    fn test_ensure_len_truncated() {
        assert!(matches!(
            ensure_len(&[0u8; 3], 5),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 5,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_validate_utf8_ok_is_borrowed() {
        let bytes = b"en";
        let s = validate_utf8(bytes, "lang_tag").expect("utf-8 valide");
        assert_eq!(s, "en");
        // zero-copy : la reference pointe dans le buffer d'origine
        assert_eq!(s.as_ptr(), bytes.as_ptr());
    }

    #[test]
    fn test_validate_utf8_invalid() {
        assert!(matches!(
            validate_utf8(&[0xFF, 0xFE], "url"),
            Err(SrvlocPacketParseError::InvalidUtf8("url"))
        ));
    }
}
