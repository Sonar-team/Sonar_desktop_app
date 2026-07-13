// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::transport::udp::UdpError;

pub const UDP_HEADER_SIZE: usize = 8;

pub fn validate_udp_min_length(data: &[u8]) -> Result<(), UdpError> {
    if data.len() < UDP_HEADER_SIZE {
        return Err(UdpError::PacketTooShort {
            expected: UDP_HEADER_SIZE,
            actual: data.len(),
        });
    }
    Ok(())
}

pub fn validate_udp_length(length: u16, actual: usize) -> Result<(), UdpError> {
    if length as usize != actual {
        return Err(UdpError::InvalidLength { length, actual });
    }
    Ok(())
}
