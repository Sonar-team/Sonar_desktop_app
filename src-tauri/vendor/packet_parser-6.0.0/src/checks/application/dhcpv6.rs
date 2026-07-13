// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::dhcpv6::Dhcpv6PacketParseError;

pub const DHCPV6_MIN_CLIENT_SERVER_LEN: usize = 4;

pub fn validate_dhcpv6_min_length(payload: &[u8]) -> Result<(), Dhcpv6PacketParseError> {
    if payload.len() < DHCPV6_MIN_CLIENT_SERVER_LEN {
        return Err(Dhcpv6PacketParseError::PacketLength);
    }
    Ok(())
}

pub fn validate_dhcpv6_message_type(message_type: u8) -> Result<(), Dhcpv6PacketParseError> {
    if !(1..=13).contains(&message_type) {
        return Err(Dhcpv6PacketParseError::MessageType { message_type });
    }
    Ok(())
}
