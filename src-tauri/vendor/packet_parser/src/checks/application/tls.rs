// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::tls::TlsError;

pub const TLS_RECORD_HEADER_LEN: usize = 5;

pub fn validate_tls_header_length(buf: &[u8]) -> Result<(), TlsError> {
    if buf.len() < TLS_RECORD_HEADER_LEN {
        return Err(TlsError::TooShort);
    }

    Ok(())
}

pub fn validate_tls_payload_length(length: u16, available: usize) -> Result<(), TlsError> {
    if available < length as usize {
        return Err(TlsError::InconsistentLength {
            declared: length,
            available,
        });
    }

    Ok(())
}

/// Validates that a full TLS record (header + declared payload) fits in the
/// `remaining` bytes of the buffer.
///
/// Used by the record iteration loop to detect a truncated trailing record.
pub fn validate_tls_record_complete(remaining: usize, length: u16) -> Result<(), TlsError> {
    let record_total_len = TLS_RECORD_HEADER_LEN + length as usize;
    if remaining < record_total_len {
        return Err(TlsError::InconsistentLength {
            declared: length,
            available: remaining.saturating_sub(TLS_RECORD_HEADER_LEN),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tls_header_length_ok() {
        assert!(validate_tls_header_length(&[0u8; TLS_RECORD_HEADER_LEN]).is_ok());
        assert!(validate_tls_header_length(&[0u8; 10]).is_ok());
    }

    #[test]
    fn test_validate_tls_header_length_too_short() {
        let err = validate_tls_header_length(&[0u8; TLS_RECORD_HEADER_LEN - 1]).unwrap_err();
        assert_eq!(err, TlsError::TooShort);
    }

    #[test]
    fn test_validate_tls_payload_length_ok() {
        assert!(validate_tls_payload_length(5, 5).is_ok());
        assert!(validate_tls_payload_length(5, 6).is_ok());
        assert!(validate_tls_payload_length(0, 0).is_ok());
    }

    #[test]
    fn test_validate_tls_payload_length_truncated() {
        let err = validate_tls_payload_length(6, 5).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InconsistentLength {
                declared: 6,
                available: 5
            }
        ));
    }

    #[test]
    fn test_validate_tls_record_complete_ok() {
        // header (5) + payload (3) exactly fits in 8 remaining bytes
        assert!(validate_tls_record_complete(8, 3).is_ok());
        assert!(validate_tls_record_complete(9, 3).is_ok());
    }

    #[test]
    fn test_validate_tls_record_complete_truncated() {
        // header (5) + payload (4) needs 9 bytes, only 6 remaining
        let err = validate_tls_record_complete(6, 4).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InconsistentLength {
                declared: 4,
                available: 1
            }
        ));
    }

    #[test]
    fn test_validate_tls_record_complete_remaining_shorter_than_header() {
        let err = validate_tls_record_complete(3, 0).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InconsistentLength {
                declared: 0,
                available: 0
            }
        ));
    }
}
