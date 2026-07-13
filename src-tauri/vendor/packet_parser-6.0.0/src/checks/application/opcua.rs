// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::opcua::OpcuaParseError;

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
}
