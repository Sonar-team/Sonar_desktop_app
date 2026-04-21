// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Module for parsing DHCPv6 packets.

use crate::errors::application::dhcpv6::Dhcpv6PacketParseError;
use std::fmt;

/// The `Dhcpv6Packet` struct represents a parsed DHCPv6 packet.
#[derive(Debug, PartialEq)]
pub struct Dhcpv6Packet<'a> {
    pub message_type: u8,
    pub transaction_id: u32,
    pub options: &'a [u8],
}

impl<'a> fmt::Display for Dhcpv6Packet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DHCPv6 Packet: message_type={}, transaction_id={:06X}, options={:02X?}",
            self.message_type, self.transaction_id, self.options
        )
    }
}

impl<'a> TryFrom<&'a [u8]> for Dhcpv6Packet<'a> {
    type Error = Dhcpv6PacketParseError;

    fn try_from(payload: &'a [u8]) -> Result<Self, Self::Error> {
        // The standard DHCPv6 Client/Server message is at least 4 bytes long.
        // (1 byte message type + 3 bytes transaction ID)
        if payload.len() < 4 {
            return Err(Dhcpv6PacketParseError::PacketLength);
        }

        let message_type = payload[0];

        // Allowed message types in DHCPv6 go from 1 to 13 (including Relay Agents 12 and 13)
        if !(1..=13).contains(&message_type) {
            return Err(Dhcpv6PacketParseError::MessageType { message_type });
        }

        // Transaction ID is 24 bits (3 bytes), so we pad it with a 0 to make a u32
        let transaction_id = u32::from_be_bytes([0, payload[1], payload[2], payload[3]]);

        // The rest are options (zero-copy slice)
        let options = &payload[4..];

        Ok(Dhcpv6Packet {
            message_type,
            transaction_id,
            options,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dhcpv6_packet() {
        // Example DHCPv6 Solicit (Type 1)
        let payload = vec![
            0x01, // msg-type: SOLICIT
            0x12, 0x34, 0x56, // transaction-id
            0x00, 0x01, 0x00, 0x0A, // options
            0x00, 0x03, 0x00, 0x01, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
        ];

        match Dhcpv6Packet::try_from(payload.as_slice()) {
            Ok(packet) => {
                assert_eq!(packet.message_type, 1);
                assert_eq!(packet.transaction_id, 0x123456);
                assert_eq!(
                    packet.options,
                    &[
                        0x00, 0x01, 0x00, 0x0A, 0x00, 0x03, 0x00, 0x01, 0x00, 0x11, 0x22, 0x33,
                        0x44, 0x55
                    ]
                );
            }
            Err(_) => panic!("Expected valid DHCPv6 packet"),
        }
    }

    #[test]
    fn test_parse_dhcpv6_packet_short_payload() {
        let short_payload = vec![0x01, 0x12, 0x34]; // Only 3 bytes
        match Dhcpv6Packet::try_from(short_payload.as_slice()) {
            Ok(_) => panic!("Expected invalid DHCPv6 packet due to short payload"),
            Err(e) => assert_eq!(e, Dhcpv6PacketParseError::PacketLength),
        }
    }

    #[test]
    fn test_parse_dhcpv6_packet_invalid_type() {
        // Unknown message type 14 (0x0E)
        let invalid_payload = vec![
            0x0E, // msg-type: UNKNOWN
            0x12, 0x34, 0x56, // transaction-id
            0x00, 0x00, 0x00, 0x00, // Options
        ];
        match Dhcpv6Packet::try_from(invalid_payload.as_slice()) {
            Ok(_) => panic!("Expected invalid DHCPv6 packet due to invalid message type"),
            Err(e) => assert_eq!(e, Dhcpv6PacketParseError::MessageType { message_type: 14 }),
        }
    }

    #[test]
    fn test_parse_dhcpv6_relay_agent() {
        // Relay-forward message (Type 12)
        let relay_payload = vec![
            0x0C, // msg-type: RELAY-FORW
            0x01, 0x02, 0x03, // pseudo transaction-id (mapped from hop-count + logic)
            0x00, 0x00, 0x00, 0x00, // Options
        ];
        match Dhcpv6Packet::try_from(relay_payload.as_slice()) {
            Ok(packet) => {
                assert_eq!(packet.message_type, 12);
            }
            Err(e) => panic!("Expected valid DHCPv6 Relay packet, got {:?}", e),
        }
    }
}
