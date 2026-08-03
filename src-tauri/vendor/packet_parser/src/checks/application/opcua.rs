// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::opcua::OpcuaParseError,
    parse::application::protocols::opcua::{OpcuaChunkType, OpcuaMessageType},
};

pub const OPCUA_TCP_HEADER_LEN: usize = 8;

pub fn validate_tcp_header_length(len: usize) -> Result<(), OpcuaParseError> {
    if len < OPCUA_TCP_HEADER_LEN {
        return Err(OpcuaParseError::PacketTooShort {
            expected: OPCUA_TCP_HEADER_LEN,
            actual: len,
        });
    }

    Ok(())
}

pub fn validate_message_size(message_size: u32) -> Result<(), OpcuaParseError> {
    if message_size < OPCUA_TCP_HEADER_LEN as u32 {
        return Err(OpcuaParseError::InvalidMessageSize { size: message_size });
    }

    Ok(())
}

pub fn validate_body_len(actual: usize, expected: usize) -> Result<(), OpcuaParseError> {
    if actual < expected {
        return Err(OpcuaParseError::BodyTooShort { expected, actual });
    }

    Ok(())
}

pub fn validate_ua_string_len(length: i32) -> Result<(), OpcuaParseError> {
    if length < -1 {
        return Err(OpcuaParseError::InvalidStringLength { length });
    }

    Ok(())
}

pub fn validate_ua_string_available(actual: usize, expected: usize) -> Result<(), OpcuaParseError> {
    if actual < expected {
        return Err(OpcuaParseError::TruncatedString { expected, actual });
    }

    Ok(())
}

/// Validates that the declared chunk `message_size` fits within the bytes
/// still `available` in the buffer.
///
/// The chunk loop uses a failure here to classify the chunk as partial
/// instead of aborting the whole parse.
pub fn validate_chunk_available(
    available: usize,
    message_size: usize,
) -> Result<(), OpcuaParseError> {
    if available < message_size {
        return Err(OpcuaParseError::TruncatedChunk {
            expected: message_size,
            actual: available,
        });
    }

    Ok(())
}

/// Checks the 3-byte message type field against the known OPC UA TCP message
/// types (`HEL`, `ACK`, `ERR`, `RHE`, `OPN`, `MSG`, `CLO`) and returns the
/// typed variant.
pub fn extract_message_type(bytes: [u8; 3]) -> Result<OpcuaMessageType, OpcuaParseError> {
    match &bytes {
        b"HEL" => Ok(OpcuaMessageType::Hello),
        b"ACK" => Ok(OpcuaMessageType::Acknowledge),
        b"ERR" => Ok(OpcuaMessageType::ErrorMessage),
        b"RHE" => Ok(OpcuaMessageType::ReverseHello),
        b"OPN" => Ok(OpcuaMessageType::OpenSecureChannel),
        b"MSG" => Ok(OpcuaMessageType::Message),
        b"CLO" => Ok(OpcuaMessageType::CloseSecureChannel),
        _ => Err(OpcuaParseError::UnknownMessageType(bytes)),
    }
}

/// Checks the chunk type byte (`F` final, `C` intermediate, `A` abort) and
/// returns the typed variant.
pub fn extract_chunk_type(byte: u8) -> Result<OpcuaChunkType, OpcuaParseError> {
    match byte {
        b'F' => Ok(OpcuaChunkType::Final),
        b'C' => Ok(OpcuaChunkType::Intermediate),
        b'A' => Ok(OpcuaChunkType::Abort),
        _ => Err(OpcuaParseError::UnknownChunkType(byte)),
    }
}

/// Checks that 4 bytes are available at `offset` and returns the
/// little-endian `u32` field read there.
///
/// Re-checks the bound even when the caller already ran a grouped
/// `validate_body_len`, so the function can never panic on a hostile input.
pub fn extract_u32_le(bytes: &[u8], offset: usize) -> Result<u32, OpcuaParseError> {
    // saturating_add: an absurd offset saturates to usize::MAX and fails the
    // bound check below instead of overflowing.
    let end = offset.saturating_add(4);
    validate_body_len(bytes.len(), end)?;

    Ok(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

/// Checks an OPC UA string field (4-byte little-endian signed length prefix
/// followed by UTF-8 bytes) and returns the borrowed value together with the
/// number of bytes consumed.
///
/// A length of `-1` encodes a null string (`None`); any other negative
/// length, a truncated body or non-UTF-8 bytes are rejected.
pub fn extract_ua_string(bytes: &[u8]) -> Result<(Option<&str>, usize), OpcuaParseError> {
    const LEN_FIELD: usize = 4;
    validate_body_len(bytes.len(), LEN_FIELD)?;

    let length = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if length == -1 {
        return Ok((None, LEN_FIELD));
    }
    validate_ua_string_len(length)?;

    let length = length as usize;
    let end = LEN_FIELD + length;
    validate_ua_string_available(bytes.len(), end)?;

    let value =
        core::str::from_utf8(&bytes[LEN_FIELD..end]).map_err(|_| OpcuaParseError::InvalidUtf8)?;

    Ok((Some(value), end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tcp_header_length_ok() {
        assert!(validate_tcp_header_length(OPCUA_TCP_HEADER_LEN).is_ok());
        assert!(validate_tcp_header_length(OPCUA_TCP_HEADER_LEN + 1).is_ok());
    }

    #[test]
    fn test_validate_tcp_header_length_too_short() {
        let err = validate_tcp_header_length(OPCUA_TCP_HEADER_LEN - 1).unwrap_err();
        assert_eq!(
            err,
            OpcuaParseError::PacketTooShort {
                expected: OPCUA_TCP_HEADER_LEN,
                actual: OPCUA_TCP_HEADER_LEN - 1,
            }
        );
    }

    #[test]
    fn test_validate_message_size_ok() {
        assert!(validate_message_size(OPCUA_TCP_HEADER_LEN as u32).is_ok());
        assert!(validate_message_size(1024).is_ok());
    }

    #[test]
    fn test_validate_message_size_smaller_than_header() {
        let err = validate_message_size(OPCUA_TCP_HEADER_LEN as u32 - 1).unwrap_err();
        assert_eq!(err, OpcuaParseError::InvalidMessageSize { size: 7 });
    }

    #[test]
    fn test_validate_body_len_ok() {
        assert!(validate_body_len(20, 20).is_ok());
        assert!(validate_body_len(21, 20).is_ok());
    }

    #[test]
    fn test_validate_body_len_too_short() {
        let err = validate_body_len(19, 20).unwrap_err();
        assert_eq!(
            err,
            OpcuaParseError::BodyTooShort {
                expected: 20,
                actual: 19,
            }
        );
    }

    #[test]
    fn test_validate_ua_string_len_ok() {
        assert!(validate_ua_string_len(0).is_ok());
        assert!(validate_ua_string_len(42).is_ok());
        assert!(validate_ua_string_len(-1).is_ok());
    }

    #[test]
    fn test_validate_ua_string_len_negative() {
        let err = validate_ua_string_len(-2).unwrap_err();
        assert_eq!(err, OpcuaParseError::InvalidStringLength { length: -2 });
    }

    #[test]
    fn test_validate_ua_string_available_ok() {
        assert!(validate_ua_string_available(12, 12).is_ok());
        assert!(validate_ua_string_available(13, 12).is_ok());
    }

    #[test]
    fn test_validate_ua_string_available_truncated() {
        let err = validate_ua_string_available(11, 12).unwrap_err();
        assert_eq!(
            err,
            OpcuaParseError::TruncatedString {
                expected: 12,
                actual: 11,
            }
        );
    }

    #[test]
    fn test_validate_chunk_available_ok() {
        assert!(validate_chunk_available(28, 28).is_ok());
        assert!(validate_chunk_available(29, 28).is_ok());
    }

    #[test]
    fn test_validate_chunk_available_size_beyond_buffer() {
        let err = validate_chunk_available(8, 28).unwrap_err();
        assert_eq!(
            err,
            OpcuaParseError::TruncatedChunk {
                expected: 28,
                actual: 8,
            }
        );
    }

    #[test]
    fn test_extract_message_type_known() {
        assert_eq!(extract_message_type(*b"HEL"), Ok(OpcuaMessageType::Hello));
        assert_eq!(
            extract_message_type(*b"ACK"),
            Ok(OpcuaMessageType::Acknowledge)
        );
        assert_eq!(
            extract_message_type(*b"ERR"),
            Ok(OpcuaMessageType::ErrorMessage)
        );
        assert_eq!(
            extract_message_type(*b"RHE"),
            Ok(OpcuaMessageType::ReverseHello)
        );
        assert_eq!(
            extract_message_type(*b"OPN"),
            Ok(OpcuaMessageType::OpenSecureChannel)
        );
        assert_eq!(extract_message_type(*b"MSG"), Ok(OpcuaMessageType::Message));
        assert_eq!(
            extract_message_type(*b"CLO"),
            Ok(OpcuaMessageType::CloseSecureChannel)
        );
    }

    #[test]
    fn test_extract_message_type_unknown() {
        assert_eq!(
            extract_message_type(*b"BAD"),
            Err(OpcuaParseError::UnknownMessageType(*b"BAD"))
        );
    }

    #[test]
    fn test_extract_chunk_type_known() {
        assert_eq!(extract_chunk_type(b'F'), Ok(OpcuaChunkType::Final));
        assert_eq!(extract_chunk_type(b'C'), Ok(OpcuaChunkType::Intermediate));
        assert_eq!(extract_chunk_type(b'A'), Ok(OpcuaChunkType::Abort));
    }

    #[test]
    fn test_extract_chunk_type_unknown() {
        assert_eq!(
            extract_chunk_type(b'X'),
            Err(OpcuaParseError::UnknownChunkType(b'X'))
        );
    }

    #[test]
    fn test_extract_u32_le_ok() {
        let bytes = [0x01, 0x00, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12];
        assert_eq!(extract_u32_le(&bytes, 0), Ok(1));
        assert_eq!(extract_u32_le(&bytes, 4), Ok(0x1234_5678));
    }

    #[test]
    fn test_extract_u32_le_out_of_bounds() {
        let bytes = [0x01, 0x00, 0x00];
        assert_eq!(
            extract_u32_le(&bytes, 0),
            Err(OpcuaParseError::BodyTooShort {
                expected: 4,
                actual: 3,
            })
        );
        assert_eq!(
            extract_u32_le(&[0u8; 8], 5),
            Err(OpcuaParseError::BodyTooShort {
                expected: 9,
                actual: 8,
            })
        );
    }

    #[test]
    fn test_extract_ua_string_value() {
        let mut bytes = 3i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(b"opc");
        assert_eq!(extract_ua_string(&bytes), Ok((Some("opc"), 7)));
    }

    #[test]
    fn test_extract_ua_string_null() {
        let bytes = (-1i32).to_le_bytes();
        assert_eq!(extract_ua_string(&bytes), Ok((None, 4)));
    }

    #[test]
    fn test_extract_ua_string_negative_length() {
        let bytes = (-2i32).to_le_bytes();
        assert_eq!(
            extract_ua_string(&bytes),
            Err(OpcuaParseError::InvalidStringLength { length: -2 })
        );
    }

    #[test]
    fn test_extract_ua_string_truncated() {
        let mut bytes = 8i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(b"opc");
        assert_eq!(
            extract_ua_string(&bytes),
            Err(OpcuaParseError::TruncatedString {
                expected: 12,
                actual: 7,
            })
        );
    }

    #[test]
    fn test_extract_ua_string_missing_length_prefix() {
        assert_eq!(
            extract_ua_string(&[0x01, 0x02]),
            Err(OpcuaParseError::BodyTooShort {
                expected: 4,
                actual: 2,
            })
        );
    }

    #[test]
    fn test_extract_ua_string_invalid_utf8() {
        let mut bytes = 2i32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0xFF, 0xFE]);
        assert_eq!(extract_ua_string(&bytes), Err(OpcuaParseError::InvalidUtf8));
    }
}
