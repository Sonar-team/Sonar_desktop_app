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

/// Length of the fixed SLPv1 header (version through language), in bytes.
const SRVLOC_V1_HEADER_LENGTH: usize = 8;
/// Length of the fixed SLPv2 header (version through lang tag length), in bytes.
const SRVLOC_V2_HEADER_LENGTH: usize = 14;

/// Validates that the packet holds at least the fixed SLPv1 header (8 bytes).
pub fn validate_v1_header_length(payload: &[u8]) -> Result<(), SrvlocPacketParseError> {
    ensure_len(payload, SRVLOC_V1_HEADER_LENGTH)
}

/// Validates that the packet holds at least the fixed SLPv2 header (14 bytes).
pub fn validate_v2_header_length(payload: &[u8]) -> Result<(), SrvlocPacketParseError> {
    ensure_len(payload, SRVLOC_V2_HEADER_LENGTH)
}

/// Reads a big-endian `u16` at `offset` and advances the cursor on success.
pub fn read_u16(buf: &[u8], offset: &mut usize) -> Result<u16, SrvlocPacketParseError> {
    ensure_len(buf, *offset + 2)?;
    let v = u16::from_be_bytes([buf[*offset], buf[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

/// Reads a big-endian 3-byte integer at `offset` into a `u32` and advances
/// the cursor on success.
pub fn read_u24(buf: &[u8], offset: &mut usize) -> Result<u32, SrvlocPacketParseError> {
    ensure_len(buf, *offset + 3)?;
    let v = ((buf[*offset] as u32) << 16)
        | ((buf[*offset + 1] as u32) << 8)
        | (buf[*offset + 2] as u32);
    *offset += 3;
    Ok(v)
}

/// Reads `len` bytes at `offset`, validates them as UTF-8 and advances the
/// cursor.
///
/// Zero-copy: the returned `&str` borrows from `buf`.
pub fn read_str<'a>(
    buf: &'a [u8],
    offset: &mut usize,
    len: usize,
    field: &'static str,
) -> Result<&'a str, SrvlocPacketParseError> {
    ensure_len(buf, *offset + len)?;
    let slice = &buf[*offset..*offset + len];
    *offset += len;
    validate_utf8(slice, field)
}

/// Extracts the SLP version byte (byte 0), checking the packet is not empty.
pub fn extract_version(payload: &[u8]) -> Result<u8, SrvlocPacketParseError> {
    validate_packet_not_empty(payload)?;
    Ok(payload[0])
}

/// Extracts the function code after validating it against the SLP version
/// (RFC 2165 for v1, RFC 2608 for v2).
pub fn extract_function(version: u8, function: u8) -> Result<u8, SrvlocPacketParseError> {
    validate_function(version, function)?;
    Ok(function)
}

/// Extracts the SLPv1 declared packet length (big-endian `u16`) from the two
/// bytes at wire offsets 2-3.
pub fn extract_packet_length_v1(bytes: &[u8]) -> Result<u16, SrvlocPacketParseError> {
    ensure_len(bytes, 2)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

/// Extracts the SLPv1 flags byte. No bit is constrained by the parser.
pub fn extract_flags_v1(flags: u8) -> Result<u8, SrvlocPacketParseError> {
    Ok(flags)
}

/// Extracts the SLPv1 dialect byte. No value is constrained by the parser.
pub fn extract_dialect(dialect: u8) -> Result<u8, SrvlocPacketParseError> {
    Ok(dialect)
}

/// Extracts the SLPv1 language field (2 bytes) as validated UTF-8.
pub fn extract_language(bytes: &[u8]) -> Result<&str, SrvlocPacketParseError> {
    validate_utf8(bytes, "language")
}

/// Extracts the SLPv1 character encoding byte at `offset` and advances the
/// cursor.
pub fn extract_encoding(buf: &[u8], offset: &mut usize) -> Result<u8, SrvlocPacketParseError> {
    ensure_len(buf, *offset + 1)?;
    let encoding = buf[*offset];
    *offset += 1;
    Ok(encoding)
}

/// Extracts the SLPv1 transaction id (big-endian `u16`) at `offset` and
/// advances the cursor.
pub fn extract_transaction_id(
    buf: &[u8],
    offset: &mut usize,
) -> Result<u16, SrvlocPacketParseError> {
    read_u16(buf, offset)
}

/// Extracts the SLPv1 error code (big-endian `u16`) at `offset` and advances
/// the cursor.
pub fn extract_error_code(buf: &[u8], offset: &mut usize) -> Result<u16, SrvlocPacketParseError> {
    read_u16(buf, offset)
}

/// Extracts the SLPv1 URL: a `u16` length prefix followed by that many bytes
/// of validated UTF-8. Returns `(url_length, url)`; the `&str` borrows from
/// `buf`.
pub fn extract_url<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<(u16, &'a str), SrvlocPacketParseError> {
    let url_length = read_u16(buf, offset)?;
    let url = read_str(buf, offset, url_length as usize, "url")?;
    Ok((url_length, url))
}

/// Extracts the SLPv1 scope list: a `u16` length prefix followed by that many
/// bytes of validated UTF-8. Returns `(scope_list_length, scope_list)`.
pub fn extract_scope_list<'a>(
    buf: &'a [u8],
    offset: &mut usize,
) -> Result<(u16, &'a str), SrvlocPacketParseError> {
    let scope_list_length = read_u16(buf, offset)?;
    let scope_list = read_str(buf, offset, scope_list_length as usize, "scope_list")?;
    Ok((scope_list_length, scope_list))
}

/// Extracts the SLPv2 declared packet length (big-endian `u24`) at `offset`
/// and advances the cursor.
pub fn extract_packet_length_v2(
    buf: &[u8],
    offset: &mut usize,
) -> Result<u32, SrvlocPacketParseError> {
    read_u24(buf, offset)
}

/// Extracts the SLPv2 flags (big-endian `u16`) at `offset` and advances the
/// cursor. No bit is constrained by the parser.
pub fn extract_flags_v2(buf: &[u8], offset: &mut usize) -> Result<u16, SrvlocPacketParseError> {
    read_u16(buf, offset)
}

/// Extracts the SLPv2 next extension offset (big-endian `u24`) at `offset`
/// and advances the cursor.
pub fn extract_next_extension_offset(
    buf: &[u8],
    offset: &mut usize,
) -> Result<u32, SrvlocPacketParseError> {
    read_u24(buf, offset)
}

/// Extracts the SLPv2 XID (big-endian `u16`) at `offset` and advances the
/// cursor.
pub fn extract_xid(buf: &[u8], offset: &mut usize) -> Result<u16, SrvlocPacketParseError> {
    read_u16(buf, offset)
}

/// Extracts the SLPv2 language tag length (big-endian `u16`) at `offset` and
/// advances the cursor.
pub fn extract_lang_tag_len(buf: &[u8], offset: &mut usize) -> Result<u16, SrvlocPacketParseError> {
    read_u16(buf, offset)
}

/// Extracts the SLPv2 language tag: `len` bytes of validated UTF-8 at
/// `offset`, advancing the cursor.
pub fn extract_lang_tag<'a>(
    buf: &'a [u8],
    offset: &mut usize,
    len: usize,
) -> Result<&'a str, SrvlocPacketParseError> {
    read_str(buf, offset, len, "lang_tag")
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

    #[test]
    fn test_validate_v1_header_length() {
        assert!(validate_v1_header_length(&[0u8; 8]).is_ok());
        assert!(matches!(
            validate_v1_header_length(&[0u8; 7]),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 8,
                actual: 7
            })
        ));
    }

    #[test]
    fn test_validate_v2_header_length() {
        assert!(validate_v2_header_length(&[0u8; 14]).is_ok());
        assert!(matches!(
            validate_v2_header_length(&[0u8; 13]),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 14,
                actual: 13
            })
        ));
    }

    #[test]
    fn test_extract_version_ok_and_empty() {
        assert_eq!(extract_version(&[1u8, 8]), Ok(1));
        assert!(matches!(
            extract_version(&[]),
            Err(SrvlocPacketParseError::InvalidPacketLength)
        ));
    }

    #[test]
    fn test_extract_function_valid_and_invalid() {
        assert_eq!(extract_function(1, 8), Ok(8));
        assert!(matches!(
            extract_function(1, 11),
            Err(SrvlocPacketParseError::UnsupportedFunction {
                version: 1,
                function: 11
            })
        ));
        assert!(matches!(
            extract_function(2, 0),
            Err(SrvlocPacketParseError::UnsupportedFunction {
                version: 2,
                function: 0
            })
        ));
    }

    #[test]
    fn test_extract_packet_length_v1() {
        assert_eq!(extract_packet_length_v1(&[0x00, 0x16]), Ok(22));
        assert!(matches!(
            extract_packet_length_v1(&[0x00]),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn test_extract_unconstrained_bytes() {
        assert_eq!(extract_flags_v1(0x20), Ok(0x20));
        assert_eq!(extract_dialect(0), Ok(0));
    }

    #[test]
    fn test_extract_language() {
        assert_eq!(extract_language(b"en"), Ok("en"));
        assert!(matches!(
            extract_language(&[0xC0, 0x00]),
            Err(SrvlocPacketParseError::InvalidUtf8("language"))
        ));
    }

    #[test]
    fn test_extract_encoding_advances_offset() {
        let buf = [3u8, 0xAA];
        let mut offset = 0;
        assert_eq!(extract_encoding(&buf, &mut offset), Ok(3));
        assert_eq!(offset, 1);

        // Truncated une fois le curseur en fin de buffer.
        let mut offset = 2;
        assert!(matches!(
            extract_encoding(&buf, &mut offset),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 3,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_read_u16_and_u24_advance_offset() {
        let buf = [0x12, 0x34, 0x56, 0x78, 0x9A];
        let mut offset = 0;
        assert_eq!(read_u16(&buf, &mut offset), Ok(0x1234));
        assert_eq!(offset, 2);
        assert_eq!(read_u24(&buf, &mut offset), Ok(0x56789A));
        assert_eq!(offset, 5);
        assert!(matches!(
            read_u16(&buf, &mut offset),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 7,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_extract_url_length_prefixed() {
        // longueur u16 = 3 puis "svc"
        let buf = [0x00, 0x03, b's', b'v', b'c'];
        let mut offset = 0;
        assert_eq!(extract_url(&buf, &mut offset), Ok((3, "svc")));
        assert_eq!(offset, 5);
    }

    #[test]
    fn test_extract_url_truncated_and_invalid_utf8() {
        let mut offset = 0;
        assert!(matches!(
            extract_url(&[0x00, 0x09, b'a'], &mut offset),
            Err(SrvlocPacketParseError::Truncated {
                expected_at_least: 11,
                actual: 3
            })
        ));

        let mut offset = 0;
        assert!(matches!(
            extract_url(&[0x00, 0x01, 0xFF], &mut offset),
            Err(SrvlocPacketParseError::InvalidUtf8("url"))
        ));
    }

    #[test]
    fn test_extract_scope_list_length_prefixed() {
        let buf = [0x00, 0x02, b's', b'c'];
        let mut offset = 0;
        assert_eq!(extract_scope_list(&buf, &mut offset), Ok((2, "sc")));
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_extract_v2_header_fields() {
        // packet length u24, flags u16, next extension offset u24, xid u16,
        // lang tag len u16 : la sequence complete du header v2 apres function.
        let buf = [
            0x00, 0x00, 0x10, // packet length = 16
            0x20, 0x00, // flags
            0x00, 0x00, 0x00, // next extension offset
            0x42, 0x42, // xid
            0x00, 0x02, // lang tag len
        ];
        let mut offset = 0;
        assert_eq!(extract_packet_length_v2(&buf, &mut offset), Ok(16));
        assert_eq!(extract_flags_v2(&buf, &mut offset), Ok(0x2000));
        assert_eq!(extract_next_extension_offset(&buf, &mut offset), Ok(0));
        assert_eq!(extract_xid(&buf, &mut offset), Ok(0x4242));
        assert_eq!(extract_lang_tag_len(&buf, &mut offset), Ok(2));
        assert_eq!(offset, 12);
    }

    #[test]
    fn test_extract_lang_tag() {
        let buf = *b"en";
        let mut offset = 0;
        assert_eq!(extract_lang_tag(&buf, &mut offset, 2), Ok("en"));
        assert_eq!(offset, 2);

        let mut offset = 0;
        assert!(matches!(
            extract_lang_tag(&[0xFF, 0xFE], &mut offset, 2),
            Err(SrvlocPacketParseError::InvalidUtf8("lang_tag"))
        ));
    }
}
