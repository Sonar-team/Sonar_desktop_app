// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::transport::tcp::TcpError;

const TCP_MIN_LENGTH: usize = 20;

pub fn validate_tcp_min_length(packet: &[u8]) -> Result<(), TcpError> {
    if packet.len() < TCP_MIN_LENGTH {
        return Err(TcpError::PacketTooShort);
    }
    Ok(())
}

pub fn validate_tcp_data_offset_words(data_offset_words: u8) -> Result<(), TcpError> {
    if !(5..=15).contains(&data_offset_words) {
        return Err(TcpError::InvalidDataOffset(data_offset_words));
    }
    Ok(())
}

pub fn validate_tcp_data_offset_available(
    packet_len: usize,
    data_offset: usize,
) -> Result<(), TcpError> {
    if packet_len < data_offset {
        return Err(TcpError::PacketTooShort);
    }
    Ok(())
}

pub fn validate_tcp_reserved(reserved: u8) -> Result<(), TcpError> {
    if reserved != 0 {
        return Err(TcpError::InvalidHeaderLength);
    }
    Ok(())
}

pub fn validate_tcp_flags(flags: u8) -> Result<(), TcpError> {
    if (flags & 0x03) == 0x03 {
        return Err(TcpError::InvalidHeaderLength);
    }
    Ok(())
}
