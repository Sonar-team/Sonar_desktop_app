// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::ams::AmsParseError;

pub const AMS_HEADER_LEN: usize = 32;

pub fn validate_ams_header_length(len: usize) -> Result<(), AmsParseError> {
    if len < AMS_HEADER_LEN {
        return Err(AmsParseError::HeaderTooShort {
            expected: AMS_HEADER_LEN,
            actual: len,
        });
    }

    Ok(())
}

pub fn validate_cb_data_length(cb_data: u32, actual: usize) -> Result<(), AmsParseError> {
    if actual != cb_data as usize {
        return Err(AmsParseError::InvalidCbDataLength { cb_data, actual });
    }

    Ok(())
}

pub fn validate_cmd_id(cmd_id: u16) -> Result<(), AmsParseError> {
    if !matches!(cmd_id, 0x0001..=0x0009) {
        return Err(AmsParseError::UnknownCommand(cmd_id));
    }

    Ok(())
}

pub fn validate_state_flags(flags: u16) -> Result<(), AmsParseError> {
    let reserved_mask: u16 = !0x000F;
    if flags & reserved_mask != 0 {
        return Err(AmsParseError::InvalidStateFlags(flags));
    }

    Ok(())
}
