// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::tls::TlsError,
    parse::application::protocols::tls::{TlsContentType, TlsVersion},
};

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

/// Checks the content type byte (offset 0 of the record header) and returns
/// the typed [`TlsContentType`].
///
/// Delegates to [`TlsContentType::try_from`], the single source of truth for
/// this validation. The caller is expected to have run
/// [`validate_tls_header_length`] first; the length guard here only protects
/// out-of-contract calls on a short slice.
pub fn extract_content_type(buf: &[u8]) -> Result<TlsContentType, TlsError> {
    if buf.is_empty() {
        return Err(TlsError::TooShort);
    }
    TlsContentType::try_from(buf[0])
}

/// Checks the protocol version bytes (offsets 1-2 of the record header) and
/// returns the typed [`TlsVersion`].
///
/// Delegates to [`TlsVersion::new`], the single source of truth for this
/// validation. The caller is expected to have run
/// [`validate_tls_header_length`] first; the length guard here only protects
/// out-of-contract calls on a short slice.
pub fn extract_version(buf: &[u8]) -> Result<TlsVersion, TlsError> {
    if buf.len() < 3 {
        return Err(TlsError::TooShort);
    }
    TlsVersion::new(buf[1], buf[2])
}

/// Reads the declared payload length (offsets 3-4 of the record header,
/// big-endian `u16`).
///
/// The value itself is unconstrained; its coherence with the bytes actually
/// available is checked separately by [`validate_tls_payload_length`]. The
/// caller is expected to have run [`validate_tls_header_length`] first; the
/// length guard here only protects out-of-contract calls on a short slice.
pub fn extract_length(buf: &[u8]) -> Result<u16, TlsError> {
    if buf.len() < TLS_RECORD_HEADER_LEN {
        return Err(TlsError::TooShort);
    }
    Ok(u16::from_be_bytes([buf[3], buf[4]]))
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
    fn test_extract_content_type_valid() {
        let header = [22u8, 3, 3, 0, 5];
        assert_eq!(extract_content_type(&header), Ok(TlsContentType::Handshake));
    }

    #[test]
    fn test_extract_content_type_invalid() {
        let header = [99u8, 3, 3, 0, 5];
        assert!(matches!(
            extract_content_type(&header),
            Err(TlsError::InvalidContentType(99))
        ));
    }

    #[test]
    fn test_extract_content_type_empty_buffer() {
        assert!(matches!(extract_content_type(&[]), Err(TlsError::TooShort)));
    }

    #[test]
    fn test_extract_version_valid() {
        let header = [22u8, 3, 3, 0, 5];
        assert_eq!(
            extract_version(&header),
            Ok(TlsVersion { major: 3, minor: 3 })
        );
    }

    #[test]
    fn test_extract_version_invalid() {
        let header = [22u8, 3, 9, 0, 5];
        assert!(matches!(
            extract_version(&header),
            Err(TlsError::InvalidVersion { major: 3, minor: 9 })
        ));
    }

    #[test]
    fn test_extract_version_buffer_too_short() {
        assert!(matches!(
            extract_version(&[22u8, 3]),
            Err(TlsError::TooShort)
        ));
    }

    #[test]
    fn test_extract_length_valid() {
        let header = [22u8, 3, 3, 0x01, 0x02];
        assert_eq!(extract_length(&header), Ok(0x0102));
    }

    #[test]
    fn test_extract_length_buffer_too_short() {
        assert!(matches!(
            extract_length(&[22u8, 3, 3, 0]),
            Err(TlsError::TooShort)
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
