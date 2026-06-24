// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    checks::internet::arp::{
        ARP_IPV4_PROTOCOL_TYPE, ARP_IPV6_PROTOCOL_TYPE, validate_arp_min_length,
        validate_dynamic_arp_length, validate_hardware_len, validate_hardware_type,
        validate_operation, validate_protocol_len, validate_protocol_type,
    },
    errors::internet::arp::ArpError,
};
use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// ARP Packet Structure
///
/// Represents an Address Resolution Protocol (ARP) packet
///
/// ```mermaid
/// ---
/// title: ArpPacket
/// ---
/// packet-beta
/// 0-15: "Hardware Type u16"
/// 16-31: "Protocol Type u16"
/// 32-39: "Hardware Length u8"
/// 40-47: "Protocol Length u8"
/// 48-63: "Operation u16"
/// 64-111: "Sender Hardware Address"
/// 112-143: "Sender Protocol Address"
/// 144-191: "Target Hardware Address"
/// 192-223: "Target Protocol Address"
/// ```
#[derive(Debug, PartialEq)]
pub struct ArpPacket {
    /// Hardware type (e.g., 1 for Ethernet)
    pub hardware_type: u16,
    /// Protocol type (e.g., 0x0800 for IPv4)
    pub protocol_type: u16,
    /// Length of hardware address in bytes (e.g., 6 for Ethernet)
    pub hardware_len: u8,
    /// Length of protocol address in bytes (e.g., 4 for IPv4)
    pub protocol_len: u8,
    /// Operation (1 for request, 2 for reply)
    pub operation: u16,
    /// Sender hardware address (MAC address for Ethernet)
    pub sender_hardware_addr: [u8; 6],
    /// Sender protocol address (IPv4 or IPv6 address)
    pub sender_protocol_addr: IpAddr,
    /// Target hardware address (MAC address for Ethernet)
    pub target_hardware_addr: [u8; 6],
    /// Target protocol address (IPv4 or IPv6 address)
    pub target_protocol_addr: IpAddr,
}

impl TryFrom<&[u8]> for ArpPacket {
    type Error = ArpError;

    /// Attempts to parse a byte slice into an Arp packet
    ///
    /// # Arguments
    /// * `data` - The byte slice containing the ARP packet
    ///
    /// # Returns
    /// * `Result<Arp, InternetError>` - The parsed ARP packet or an error
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        validate_arp_min_length(data)?;

        let hardware_type = u16::from_be_bytes([data[0], data[1]]);
        validate_hardware_type(hardware_type)?;

        let protocol_type = u16::from_be_bytes([data[2], data[3]]);
        validate_protocol_type(protocol_type)?;

        let hardware_len = data[4];
        validate_hardware_len(hardware_len)?;

        let protocol_len = data[5];
        validate_dynamic_arp_length(data.len(), hardware_len, protocol_len)?;
        validate_protocol_len(protocol_type, protocol_len)?;

        let operation = u16::from_be_bytes([data[6], data[7]]);
        validate_operation(operation)?;

        let sender_hardware_addr = [data[8], data[9], data[10], data[11], data[12], data[13]];

        let sender_protocol_addr = match protocol_type {
            ARP_IPV4_PROTOCOL_TYPE => {
                IpAddr::V4(Ipv4Addr::new(data[14], data[15], data[16], data[17]))
            }
            ARP_IPV6_PROTOCOL_TYPE => {
                let mut addr = [0u8; 16];
                addr.copy_from_slice(&data[14..30]);
                IpAddr::V6(Ipv6Addr::from(addr))
            }
            _ => unreachable!(),
        };

        let target_hardware_addr = [
            data[14 + protocol_len as usize],
            data[15 + protocol_len as usize],
            data[16 + protocol_len as usize],
            data[17 + protocol_len as usize],
            data[18 + protocol_len as usize],
            data[19 + protocol_len as usize],
        ];

        let target_protocol_addr = match protocol_type {
            ARP_IPV4_PROTOCOL_TYPE => IpAddr::V4(Ipv4Addr::new(
                data[20 + protocol_len as usize],
                data[21 + protocol_len as usize],
                data[22 + protocol_len as usize],
                data[23 + protocol_len as usize],
            )),
            ARP_IPV6_PROTOCOL_TYPE => {
                let start = 20 + protocol_len as usize;
                let end = start + 16;
                let mut addr = [0u8; 16];
                addr.copy_from_slice(&data[start..end]);
                IpAddr::V6(Ipv6Addr::from(addr))
            }
            _ => unreachable!(), // We already checked protocol_type above
        };

        Ok(ArpPacket {
            hardware_type,
            protocol_type,
            hardware_len,
            protocol_len,
            operation,
            sender_hardware_addr,
            sender_protocol_addr,
            target_hardware_addr,
            target_protocol_addr,
        })
    }
}
