// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::copt::CotpParseError;

pub fn validate_min_len(data: &[u8]) -> Result<(), CotpParseError> {
    if data.len() < 3 {
        return Err(CotpParseError::PacketTooShort {
            expected: 3,
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn validate_declared_len(data_len: usize, declared_end: usize) -> Result<(), CotpParseError> {
    if data_len < declared_end {
        return Err(CotpParseError::LengthExceedsPacket {
            declared: declared_end,
            actual: data_len,
        });
    }

    Ok(())
}

pub fn validate_connection_header_len(
    declared_end: usize,
    expected: usize,
) -> Result<(), CotpParseError> {
    if declared_end < expected {
        return Err(CotpParseError::ConnectionHeaderTooShort {
            expected,
            actual: declared_end,
        });
    }

    Ok(())
}

pub fn validate_parameter_header(declared_end: usize, offset: usize) -> Result<(), CotpParseError> {
    if offset + 1 >= declared_end {
        return Err(CotpParseError::ParameterHeaderTruncated { offset });
    }

    Ok(())
}

pub fn validate_parameter_len(
    declared_end: usize,
    offset: usize,
    param_len: usize,
) -> Result<(), CotpParseError> {
    if offset + 2 + param_len > declared_end {
        return Err(CotpParseError::ParameterLengthExceedsPacket {
            offset,
            declared: param_len,
            available: declared_end.saturating_sub(offset + 2),
        });
    }

    Ok(())
}

pub fn validate_tpdu_number_not_empty(offset: usize, len: usize) -> Result<(), CotpParseError> {
    if len == 0 {
        return Err(CotpParseError::ParameterLengthExceedsPacket {
            offset,
            declared: 1,
            available: 0,
        });
    }

    Ok(())
}
