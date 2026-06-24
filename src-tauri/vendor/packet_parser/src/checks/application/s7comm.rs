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
