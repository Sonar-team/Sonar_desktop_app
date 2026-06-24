// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::ethernet_ip::EtherNetIpError,
    parse::application::protocols::ethernet_ip::EtherNetIpCommand,
};

pub const ENCAPSULATION_HEADER_LEN: usize = 24;
pub const REGISTER_SESSION_DATA_LEN: usize = 4;
pub const COMMON_PACKET_FORMAT_MIN_LEN: usize = 8;

pub fn validate_min_length(packet: &[u8]) -> Result<(), EtherNetIpError> {
    if packet.len() < ENCAPSULATION_HEADER_LEN {
        return Err(EtherNetIpError::PacketTooShort {
            expected: ENCAPSULATION_HEADER_LEN,
            actual: packet.len(),
        });
    }

    Ok(())
}

pub fn validate_declared_length(
    declared_length: u16,
    actual_len: usize,
) -> Result<(), EtherNetIpError> {
    let expected = ENCAPSULATION_HEADER_LEN + declared_length as usize;
    if expected != actual_len {
        return Err(EtherNetIpError::LengthMismatch {
            expected,
            actual: actual_len,
        });
    }

    Ok(())
}

pub fn validate_options(options: u32) -> Result<(), EtherNetIpError> {
    if options != 0 {
        return Err(EtherNetIpError::InvalidOptions { options });
    }

    Ok(())
}

pub fn validate_empty_command_data(
    command: EtherNetIpCommand,
    data: &[u8],
) -> Result<(), EtherNetIpError> {
    if !data.is_empty() {
        return Err(EtherNetIpError::UnexpectedCommandData {
            command: command.name(),
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn validate_register_session_length(data: &[u8]) -> Result<(), EtherNetIpError> {
    if data.len() != REGISTER_SESSION_DATA_LEN {
        return Err(EtherNetIpError::InvalidRegisterSessionLength {
            expected: REGISTER_SESSION_DATA_LEN,
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn validate_protocol_version(version: u16) -> Result<(), EtherNetIpError> {
    if version != 1 {
        return Err(EtherNetIpError::UnsupportedProtocolVersion { version });
    }

    Ok(())
}

pub fn validate_register_options_flags(flags: u16) -> Result<(), EtherNetIpError> {
    if flags != 0 {
        return Err(EtherNetIpError::InvalidRegisterSessionOptions { flags });
    }

    Ok(())
}

pub fn validate_common_packet_format_min_length(data: &[u8]) -> Result<(), EtherNetIpError> {
    if data.len() < COMMON_PACKET_FORMAT_MIN_LEN {
        return Err(EtherNetIpError::Truncated {
            field: "common_packet_format",
            needed: COMMON_PACKET_FORMAT_MIN_LEN,
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn ensure_available(
    field: &'static str,
    actual: usize,
    needed: usize,
) -> Result<(), EtherNetIpError> {
    if actual < needed {
        return Err(EtherNetIpError::Truncated {
            field,
            needed,
            actual,
        });
    }

    Ok(())
}

pub fn validate_interface_handle(interface_handle: u32) -> Result<(), EtherNetIpError> {
    if interface_handle != 0 {
        return Err(EtherNetIpError::InvalidInterfaceHandle { interface_handle });
    }

    Ok(())
}

pub fn validate_cpf_consumed(consumed: usize, actual: usize) -> Result<(), EtherNetIpError> {
    if consumed != actual {
        return Err(EtherNetIpError::TrailingCpfData { consumed, actual });
    }

    Ok(())
}
