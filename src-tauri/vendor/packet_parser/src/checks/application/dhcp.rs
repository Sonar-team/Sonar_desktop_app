// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::dhcp::DhcpParseError;

pub const DHCP_MIN_LEN: usize = 236;

pub fn validate_dhcp_min_length(payload: &[u8]) -> Result<(), DhcpParseError> {
    if payload.len() < DHCP_MIN_LEN {
        return Err(DhcpParseError::PacketTooShort {
            expected: DHCP_MIN_LEN,
            actual: payload.len(),
        });
    }

    Ok(())
}

pub fn validate_operation(op: u8) -> Result<(), DhcpParseError> {
    if !(op == 1 || op == 2) {
        return Err(DhcpParseError::InvalidOperation { op });
    }

    Ok(())
}

pub fn validate_hardware_type(htype: u8) -> Result<(), DhcpParseError> {
    if htype != 1 {
        return Err(DhcpParseError::UnsupportedHardwareType { htype });
    }

    Ok(())
}

pub fn validate_hardware_address_length(hlen: u8) -> Result<(), DhcpParseError> {
    if hlen != 6 {
        return Err(DhcpParseError::InvalidHardwareAddressLength { hlen });
    }

    Ok(())
}
