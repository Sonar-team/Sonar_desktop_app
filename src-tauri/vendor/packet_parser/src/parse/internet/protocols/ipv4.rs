// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    checks::internet::ipv4::{
        IPV4_MIN_HEADER_LEN, validate_ipv4_header_available, validate_ipv4_header_length,
        validate_ipv4_min_length, validate_ipv4_total_length, validate_ipv4_version,
    },
    errors::internet::ipv4::Ipv4Error,
};
use std::convert::TryFrom;
use std::net::Ipv4Addr;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// IPv4 Packet Structure
///
/// Represents an Internet Protocol version 4 packet
///
/// ```mermaid
/// ---
/// title: Ipv4Packet
/// ---
/// packet-beta
/// 0-3: "Version u4"
/// 4-7: "IHL u4"
/// 8-15: "DSCP/ECN u8"
/// 16-31: "Total Length u16"
/// 32-47: "Identification u16"
/// 48-50: "Flags u3"
/// 51-63: "Fragment Offset u13"
/// 64-71: "TTL u8"
/// 72-79: "Protocol u8"
/// 80-95: "Header Checksum u16"
/// 96-127: "Source IPv4 u32"
/// 128-159: "Destination IPv4 u32"
/// 160-191: "Options / Payload variable"
/// ```
#[derive(Debug, PartialEq)]
pub struct Ipv4Packet<'a> {
    /// Version (4 for IPv4) and Internet Header Length (IHL)
    pub version_ihl: u8,
    /// Type of Service
    pub dscp_ecn: u8,
    /// Total Length of the packet (header + data)
    pub total_length: u16,
    /// Identification
    pub identification: u16,
    /// Flags and Fragment Offset
    pub flags_fragment: u16,
    /// Time to Live
    pub ttl: u8,
    /// Protocol (e.g., 6 for TCP, 17 for UDP)
    pub protocol: u8,
    /// Header Checksum
    pub header_checksum: u16,
    /// Source IP Address
    pub source_addr: Ipv4Addr,
    /// Destination IP Address
    pub dest_addr: Ipv4Addr,
    /// Options (if any, variable length)
    pub options: &'a [u8],
    /// Payload data
    pub payload: &'a [u8],
}

impl<'a> Ipv4Packet<'a> {
    /// Returns the IP version (should be 4 for IPv4)
    pub fn version(&self) -> u8 {
        self.version_ihl >> 4
    }

    /// Returns the Internet Header Length (IHL) in 32-bit words
    pub fn ihl(&self) -> u8 {
        self.version_ihl & 0x0F
    }

    /// Returns the header length in bytes
    pub fn header_length(&self) -> usize {
        (self.ihl() as usize) * 4
    }

    /// Returns the Differentiated Services Code Point (DSCP)
    pub fn dscp(&self) -> u8 {
        self.dscp_ecn >> 2
    }

    /// Returns the Explicit Congestion Notification (ECN)
    pub fn ecn(&self) -> u8 {
        self.dscp_ecn & 0x03
    }

    /// Returns the flags (3 bits)
    pub fn flags(&self) -> u8 {
        (self.flags_fragment >> 13) as u8
    }

    /// Returns the fragment offset (13 bits)
    pub fn fragment_offset(&self) -> u16 {
        self.flags_fragment & 0x1FFF
    }
}

impl<'a> TryFrom<&'a [u8]> for Ipv4Packet<'a> {
    type Error = Ipv4Error;

    /// Attempts to parse a byte slice into an IPv4 packet
    ///
    /// # Arguments
    /// * `data` - The byte slice containing the IPv4 packet
    ///
    /// # Returns
    /// * `Result<Ipv4Packet, Ipv4Error>` - The parsed IPv4 packet or an error
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        validate_ipv4_min_length(data)?;

        let version_ihl = data[0];
        let version = version_ihl >> 4;
        let ihl = version_ihl & 0x0F;

        validate_ipv4_version(version)?;

        let header_len = (ihl as usize) * 4;
        validate_ipv4_header_length(header_len)?;
        validate_ipv4_header_available(data.len(), header_len)?;

        let dscp_ecn = data[1];
        let total_length = u16::from_be_bytes([data[2], data[3]]);

        validate_ipv4_total_length(total_length, header_len, data.len())?;

        let identification = u16::from_be_bytes([data[4], data[5]]);
        let flags_fragment = u16::from_be_bytes([data[6], data[7]]);
        let ttl = data[8];
        let protocol = data[9];
        let header_checksum = u16::from_be_bytes([data[10], data[11]]);

        let source_addr = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
        let dest_addr = Ipv4Addr::new(data[16], data[17], data[18], data[19]);

        // Extract options if present
        let options = if header_len > IPV4_MIN_HEADER_LEN {
            &data[IPV4_MIN_HEADER_LEN..header_len]
        } else {
            &[]
        };

        // Extract payload
        let payload =
            if (total_length as usize) > header_len && (total_length as usize) <= data.len() {
                &data[header_len..(total_length as usize)]
            } else if (total_length as usize) > data.len() {
                // If the total length exceeds the available data, use what we have
                &data[header_len..]
            } else {
                // Empty slice if no payload
                &[]
            };

        Ok(Ipv4Packet {
            version_ihl,
            dscp_ecn,
            total_length,
            identification,
            flags_fragment,
            ttl,
            protocol,
            header_checksum,
            source_addr,
            dest_addr,
            options,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ipv4_packet_parsing() {
        // Example IPv4 packet (truncated for brevity)
        let data = [
            0x45, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8,
            0x01, 0x01, 0xc0, 0xa8, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x50, 0x02, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let packet = Ipv4Packet::try_from(&data[..]).unwrap();

        assert_eq!(packet.version(), 4);
        assert_eq!(packet.ihl(), 5);
        assert_eq!(packet.header_length(), 20);
        assert_eq!(packet.total_length, 60);
        assert_eq!(packet.protocol, 6); // TCP
        assert_eq!(packet.source_addr, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(packet.dest_addr, Ipv4Addr::new(192, 168, 1, 2));
        assert!(packet.options.is_empty());
    }

    #[test]
    fn test_invalid_version() {
        // Version 6 (should be 4)
        let data = [
            0x65, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8,
            0x01, 0x01, 0xc0, 0xa8, 0x01, 0x02,
        ];

        let result = Ipv4Packet::try_from(&data[..]);
        assert!(matches!(result, Err(Ipv4Error::InvalidVersion(6))));
    }

    #[test]
    fn test_invalid_header_length() {
        // IHL = 1 (less than minimum 5)
        let data = [
            0x41, 0x00, 0x00, 0x3c, 0x1c, 0x46, 0x40, 0x00, 0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8,
            0x01, 0x01, 0xc0, 0xa8, 0x01, 0x02,
        ];

        let result = Ipv4Packet::try_from(&data[..]);
        assert!(matches!(result, Err(Ipv4Error::InvalidHeaderLength(4))));
    }
}
