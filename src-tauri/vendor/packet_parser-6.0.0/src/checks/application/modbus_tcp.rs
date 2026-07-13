// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::modbus_tcp::ModbusTcpError;

pub const MBAP_MIN_SIZE: usize = 7;

pub fn validate_mbap_min_size(value: &[u8]) -> Result<(), ModbusTcpError> {
    if value.len() < MBAP_MIN_SIZE {
        return Err(ModbusTcpError::BufferTooSmall {
            needed: MBAP_MIN_SIZE,
            actual: value.len(),
        });
    }

    Ok(())
}

pub fn validate_protocol_identifier(protocol_identifier: u16) -> Result<(), ModbusTcpError> {
    if protocol_identifier != 0 {
        return Err(ModbusTcpError::InvalidProtocolIdentifier {
            got: protocol_identifier,
        });
    }

    Ok(())
}

pub fn validate_length_field(length: u16) -> Result<(), ModbusTcpError> {
    if length < 1 {
        return Err(ModbusTcpError::InvalidLengthField { got: length });
    }

    Ok(())
}

pub fn validate_declared_total_length(value: &[u8], length: u16) -> Result<usize, ModbusTcpError> {
    let expected_total = 6usize + length as usize;
    if value.len() < expected_total {
        return Err(ModbusTcpError::LengthMismatch {
            expected: expected_total,
            actual: value.len(),
        });
    }

    Ok(expected_total)
}

pub fn validate_pdu_not_empty(pdu: &[u8]) -> Result<(), ModbusTcpError> {
    if pdu.is_empty() {
        return Err(ModbusTcpError::PduTooSmall {
            needed: 1,
            actual: pdu.len(),
        });
    }

    Ok(())
}

pub fn validate_consumed_length(consumed: usize, length: u16) -> Result<(), ModbusTcpError> {
    if consumed == 0 {
        return Err(ModbusTcpError::InvalidLengthField { got: length });
    }

    Ok(())
}
