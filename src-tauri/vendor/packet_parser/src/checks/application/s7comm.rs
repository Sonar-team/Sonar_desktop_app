// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::s7comm::S7CommParseError;

pub fn validate_min_size(packet_len: usize, min_size: usize) -> Result<(), S7CommParseError> {
    if packet_len < min_size {
        return Err(S7CommParseError::PacketTooShort {
            expected: min_size,
            actual: packet_len,
        });
    }

    Ok(())
}

pub fn validate_tpkt_version(version: u8) -> Result<(), S7CommParseError> {
    if version != 0x03 {
        return Err(S7CommParseError::InvalidTpktVersion { version });
    }

    Ok(())
}

pub fn validate_cotp_header_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::InvalidCotpHeaderLength { expected, actual });
    }

    Ok(())
}

pub fn validate_s7_header_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::S7HeaderTooShort { expected, actual });
    }

    Ok(())
}

pub fn validate_parameter_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::InvalidParameterLength { expected, actual });
    }

    Ok(())
}

pub fn validate_data_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::InvalidDataLength { expected, actual });
    }

    Ok(())
}

pub fn validate_parameter_data_not_empty(data: &[u8]) -> Result<(), S7CommParseError> {
    if data.is_empty() {
        return Err(S7CommParseError::EmptyParameterData);
    }

    Ok(())
}

pub fn validate_parameter_item_header(
    offset: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    if offset + 2 > data_len {
        return Err(S7CommParseError::InvalidParameterItemHeader);
    }

    Ok(())
}

pub fn validate_parameter_item_length(
    offset: usize,
    length: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    if offset + 2 + length > data_len {
        return Err(S7CommParseError::InvalidParameterItemLength);
    }

    Ok(())
}

pub fn validate_s7any_length(offset: usize, data_len: usize) -> Result<(), S7CommParseError> {
    if offset + 12 > data_len {
        return Err(S7CommParseError::S7AnyParameterTooShort);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_min_size() {
        assert!(validate_min_size(17, 17).is_ok());
        assert!(validate_min_size(18, 17).is_ok());
        assert_eq!(
            validate_min_size(16, 17).unwrap_err(),
            S7CommParseError::PacketTooShort {
                expected: 17,
                actual: 16,
            }
        );
    }

    #[test]
    fn test_validate_tpkt_version() {
        assert!(validate_tpkt_version(0x03).is_ok());
        assert_eq!(
            validate_tpkt_version(0x02).unwrap_err(),
            S7CommParseError::InvalidTpktVersion { version: 0x02 }
        );
    }

    #[test]
    fn test_validate_cotp_header_length() {
        assert!(validate_cotp_header_length(7, 7).is_ok());
        assert!(validate_cotp_header_length(7, 20).is_ok());
        assert_eq!(
            validate_cotp_header_length(8, 7).unwrap_err(),
            S7CommParseError::InvalidCotpHeaderLength {
                expected: 8,
                actual: 7,
            }
        );
    }

    #[test]
    fn test_validate_s7_header_length() {
        assert!(validate_s7_header_length(17, 17).is_ok());
        assert_eq!(
            validate_s7_header_length(17, 16).unwrap_err(),
            S7CommParseError::S7HeaderTooShort {
                expected: 17,
                actual: 16,
            }
        );
    }

    #[test]
    fn test_validate_parameter_length() {
        assert!(validate_parameter_length(31, 31).is_ok());
        assert_eq!(
            validate_parameter_length(31, 25).unwrap_err(),
            S7CommParseError::InvalidParameterLength {
                expected: 31,
                actual: 25,
            }
        );
    }

    #[test]
    fn test_validate_data_length() {
        assert!(validate_data_length(22, 22).is_ok());
        assert_eq!(
            validate_data_length(22, 18).unwrap_err(),
            S7CommParseError::InvalidDataLength {
                expected: 22,
                actual: 18,
            }
        );
    }

    #[test]
    fn test_validate_parameter_data_not_empty() {
        assert!(validate_parameter_data_not_empty(&[0x04]).is_ok());
        assert_eq!(
            validate_parameter_data_not_empty(&[]).unwrap_err(),
            S7CommParseError::EmptyParameterData
        );
    }

    #[test]
    fn test_validate_parameter_item_header() {
        // offset + 2 bytes of item header must fit in the parameter data
        assert!(validate_parameter_item_header(2, 4).is_ok());
        assert_eq!(
            validate_parameter_item_header(2, 3).unwrap_err(),
            S7CommParseError::InvalidParameterItemHeader
        );
    }

    #[test]
    fn test_validate_parameter_item_length() {
        // item at offset 2 with declared length 10 needs 14 bytes of data
        assert!(validate_parameter_item_length(2, 10, 14).is_ok());
        assert_eq!(
            validate_parameter_item_length(2, 10, 13).unwrap_err(),
            S7CommParseError::InvalidParameterItemLength
        );
    }

    #[test]
    fn test_validate_s7any_length() {
        assert!(validate_s7any_length(2, 14).is_ok());
        assert_eq!(
            validate_s7any_length(2, 13).unwrap_err(),
            S7CommParseError::S7AnyParameterTooShort
        );
    }
}
