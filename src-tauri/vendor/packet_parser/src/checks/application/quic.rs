// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::ApplicationError;

pub const QUIC_V1: u32 = 1;

pub fn validate_long_header(header_form_long: bool) -> Result<(), ApplicationError> {
    if !header_form_long {
        return Err(ApplicationError::QuicParseError);
    }
    Ok(())
}

pub fn validate_fixed_bit(fixed_bit: bool) -> Result<(), ApplicationError> {
    if !fixed_bit {
        return Err(ApplicationError::QuicParseError);
    }
    Ok(())
}

pub fn validate_version(version: u32) -> Result<(), ApplicationError> {
    if version != QUIC_V1 {
        return Err(ApplicationError::QuicParseError);
    }
    Ok(())
}

pub fn validate_payload_available(
    available: usize,
    expected: usize,
) -> Result<(), ApplicationError> {
    if available < expected {
        return Err(ApplicationError::QuicParseError);
    }
    Ok(())
}

pub fn validate_length_field(
    length_field: u64,
    packet_number_length: u8,
) -> Result<usize, ApplicationError> {
    (length_field as usize)
        .checked_sub(packet_number_length as usize)
        .ok_or(ApplicationError::QuicParseError)
}
