// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::internet::arp::ArpError;

pub const ARP_IPV4_ETHERNET_MIN_LEN: usize = 28;
pub const ARP_ETHERNET_HARDWARE_TYPE: u16 = 1;
pub const ARP_ETHERNET_HARDWARE_LEN: u8 = 6;
pub const ARP_IPV4_PROTOCOL_TYPE: u16 = 0x0800;
pub const ARP_IPV6_PROTOCOL_TYPE: u16 = 0x86DD;
pub const ARP_IPV4_PROTOCOL_LEN: u8 = 4;
pub const ARP_IPV6_PROTOCOL_LEN: u8 = 16;

pub fn validate_arp_min_length(data: &[u8]) -> Result<(), ArpError> {
    if data.len() < ARP_IPV4_ETHERNET_MIN_LEN {
        return Err(ArpError::InvalidLength {
            expected: ARP_IPV4_ETHERNET_MIN_LEN,
            actual: data.len(),
        });
    }
    Ok(())
}

pub fn validate_hardware_type(hardware_type: u16) -> Result<(), ArpError> {
    if hardware_type != ARP_ETHERNET_HARDWARE_TYPE {
        return Err(ArpError::UnsupportedHardwareType(hardware_type));
    }
    Ok(())
}

pub fn validate_hardware_len(hardware_len: u8) -> Result<(), ArpError> {
    if hardware_len != ARP_ETHERNET_HARDWARE_LEN {
        return Err(ArpError::InvalidHardwareLength {
            expected: ARP_ETHERNET_HARDWARE_LEN,
            actual: hardware_len,
        });
    }
    Ok(())
}

pub fn validate_dynamic_arp_length(
    data_len: usize,
    hardware_len: u8,
    protocol_len: u8,
) -> Result<(), ArpError> {
    let expected = 8 + (2 * hardware_len as usize) + (2 * protocol_len as usize);
    if data_len < expected {
        return Err(ArpError::InvalidLength {
            expected,
            actual: data_len,
        });
    }
    Ok(())
}

pub fn validate_operation(operation: u16) -> Result<(), ArpError> {
    if operation != 1 && operation != 2 {
        return Err(ArpError::UnsupportedOperation(operation));
    }
    Ok(())
}

pub fn validate_protocol_type(protocol_type: u16) -> Result<(), ArpError> {
    match protocol_type {
        ARP_IPV4_PROTOCOL_TYPE | ARP_IPV6_PROTOCOL_TYPE => Ok(()),
        _ => Err(ArpError::UnsupportedProtocolType(protocol_type)),
    }
}

pub fn validate_protocol_len(protocol_type: u16, protocol_len: u8) -> Result<(), ArpError> {
    let expected = match protocol_type {
        ARP_IPV4_PROTOCOL_TYPE => ARP_IPV4_PROTOCOL_LEN,
        ARP_IPV6_PROTOCOL_TYPE => ARP_IPV6_PROTOCOL_LEN,
        _ => return Err(ArpError::UnsupportedProtocolType(protocol_type)),
    };

    if protocol_len != expected {
        return Err(ArpError::InvalidProtocolLength {
            expected,
            actual: protocol_len,
        });
    }

    Ok(())
}
