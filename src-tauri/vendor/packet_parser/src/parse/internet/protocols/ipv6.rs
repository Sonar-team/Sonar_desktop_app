// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    checks::internet::ipv6::{
        IPV6_HEADER_LEN, validate_ipv6_header_length, validate_ipv6_payload_length,
        validate_ipv6_version,
    },
    errors::internet::ipv6::Ipv6Error,
    parse::internet::dscp_ecn::{Dscp, Ecn},
};
use std::convert::TryFrom;
use std::net::Ipv6Addr;

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// IPv6 Packet Structure
///
/// Represents an Internet Protocol version 6 packet
///
/// ```mermaid
/// ---
/// title: Ipv6Packet
/// ---
/// packet-beta
/// 0-3: "Version u4"
/// 4-11: "Traffic Class u8"
/// 12-31: "Flow Label u20"
/// 32-47: "Payload Length u16"
/// 48-55: "Next Header u8"
/// 56-63: "Hop Limit u8"
/// 64-191: "Source IPv6 u128"
/// 192-319: "Destination IPv6 u128"
/// 320-383: "Extension Headers / Payload variable"
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Ipv6Packet<'a> {
    /// Version (6 for IPv6), Traffic Class, and Flow Label
    pub version_tc_flow: [u8; 4],
    /// Payload Length
    pub payload_length: u16,
    /// Next Header of the fixed header. This may designate an extension
    /// header; see [`Ipv6Packet::transport_protocol`] for the upper-layer
    /// protocol reached after the extension chain.
    pub next_header: u8,
    /// Hop Limit (similar to IPv4's TTL)
    pub hop_limit: u8,
    /// Source Address
    pub source_addr: Ipv6Addr,
    /// Destination Address
    pub dest_addr: Ipv6Addr,
    /// Raw bytes of the extension header chain (empty if none).
    pub extension_headers: &'a [u8],
    /// Upper-layer protocol after the extension chain.
    ///
    /// `None` when the packet is a fragment (parsing L4 safely would require
    /// reassembly) or when the chain ends with No Next Header (59).
    pub transport_protocol: Option<u8>,
    /// Payload data, past any extension headers.
    pub payload: &'a [u8],
    /// Whether a Fragment extension header is present.
    fragmented: bool,
}

/// IPv6 extension headers chained via the Next Header field (RFC 8200).
const HOP_BY_HOP: u8 = 0;
const ROUTING: u8 = 43;
const FRAGMENT: u8 = 44;
const AUTH_HEADER: u8 = 51;
const DEST_OPTIONS: u8 = 60;
const NO_NEXT_HEADER: u8 = 59;

fn is_extension_header(next_header: u8) -> bool {
    matches!(
        next_header,
        HOP_BY_HOP | ROUTING | FRAGMENT | AUTH_HEADER | DEST_OPTIONS
    )
}

impl<'a> Ipv6Packet<'a> {
    /// Returns the IP version (should be 6 for IPv6)
    pub fn version(&self) -> u8 {
        self.version_tc_flow[0] >> 4
    }

    /// Returns the Traffic Class
    pub fn traffic_class(&self) -> u8 {
        ((self.version_tc_flow[0] & 0x0F) << 4) | (self.version_tc_flow[1] >> 4)
    }

    /// Returns the Differentiated Services Code Point carried by the
    /// Traffic Class (same DS field layout as IPv4, RFC 2474).
    pub fn dscp(&self) -> Dscp {
        Dscp::from_ds_field(self.traffic_class())
    }

    /// Returns the Explicit Congestion Notification field of the
    /// Traffic Class.
    pub fn ecn(&self) -> Ecn {
        Ecn::from_ds_field(self.traffic_class())
    }

    /// Returns the Flow Label
    pub fn flow_label(&self) -> u32 {
        ((self.version_tc_flow[1] as u32 & 0x0F) << 16)
            | ((self.version_tc_flow[2] as u32) << 8)
            | (self.version_tc_flow[3] as u32)
    }

    /// Returns true when a Fragment extension header is present. Like IPv4
    /// fragments, the L4 payload then requires reassembly to be parsed.
    pub fn is_fragmented(&self) -> bool {
        self.fragmented
    }
}

impl<'a> TryFrom<&'a [u8]> for Ipv6Packet<'a> {
    type Error = Ipv6Error;

    /// Attempts to parse a byte slice into an IPv6 packet
    ///
    /// # Arguments
    /// * `data` - The byte slice containing the IPv6 packet
    ///
    /// # Returns
    /// * `Result<Ipv6Packet, Ipv6Error>` - The parsed IPv6 packet or an error
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        validate_ipv6_header_length(data)?;

        let version = data[0] >> 4;
        validate_ipv6_version(version)?;

        let version_tc_flow = [data[0], data[1], data[2], data[3]];
        let payload_length = u16::from_be_bytes([data[4], data[5]]);
        let next_header = data[6];
        let hop_limit = data[7];

        // Parse source and destination addresses (16 bytes each)
        let source_addr = Ipv6Addr::new(
            u16::from_be_bytes([data[8], data[9]]),
            u16::from_be_bytes([data[10], data[11]]),
            u16::from_be_bytes([data[12], data[13]]),
            u16::from_be_bytes([data[14], data[15]]),
            u16::from_be_bytes([data[16], data[17]]),
            u16::from_be_bytes([data[18], data[19]]),
            u16::from_be_bytes([data[20], data[21]]),
            u16::from_be_bytes([data[22], data[23]]),
        );

        let dest_addr = Ipv6Addr::new(
            u16::from_be_bytes([data[24], data[25]]),
            u16::from_be_bytes([data[26], data[27]]),
            u16::from_be_bytes([data[28], data[29]]),
            u16::from_be_bytes([data[30], data[31]]),
            u16::from_be_bytes([data[32], data[33]]),
            u16::from_be_bytes([data[34], data[35]]),
            u16::from_be_bytes([data[36], data[37]]),
            u16::from_be_bytes([data[38], data[39]]),
        );

        let total_expected_len = validate_ipv6_payload_length(data.len(), payload_length)?;
        let full_payload = &data[IPV6_HEADER_LEN..total_expected_len];

        // Walk the extension header chain (RFC 8200 §4) until an upper-layer
        // protocol is reached. Each extension starts with (next header,
        // length), so the walk is bounded by `full_payload`.
        let mut current = next_header;
        let mut offset = 0usize;
        let mut fragmented = false;

        while is_extension_header(current) {
            let rest = &full_payload[offset..];
            if rest.len() < 2 {
                return Err(Ipv6Error::InvalidExtensionHeader(format!(
                    "truncated extension header (type {current})"
                )));
            }
            let header_len = match current {
                // Fragment header has a fixed 8-byte size (its second byte is
                // reserved, not a length).
                FRAGMENT => 8,
                // AH expresses its length in 4-byte units, minus 2 (RFC 4302).
                AUTH_HEADER => (rest[1] as usize + 2) * 4,
                // Hop-by-Hop, Routing and Destination Options use 8-byte
                // units, not counting the first 8 bytes.
                _ => (rest[1] as usize + 1) * 8,
            };
            if rest.len() < header_len {
                return Err(Ipv6Error::InvalidExtensionHeader(format!(
                    "extension header (type {current}) longer than payload: \
                     {header_len} > {} bytes",
                    rest.len()
                )));
            }
            if current == FRAGMENT {
                fragmented = true;
            }
            current = rest[0];
            offset += header_len;
        }

        let extension_headers = &full_payload[..offset];
        let payload = &full_payload[offset..];
        let transport_protocol = if fragmented || current == NO_NEXT_HEADER {
            None
        } else {
            Some(current)
        };

        Ok(Ipv6Packet {
            version_tc_flow,
            payload_length,
            next_header,
            hop_limit,
            source_addr,
            dest_addr,
            extension_headers,
            transport_protocol,
            payload,
            fragmented,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;

    #[test]
    fn test_ipv6_packet_parsing() {
        // Example IPv6 packet (truncated for brevity)
        let data = [
            // Version (6), Traffic Class, Flow Label (0x12345)
            0x60, 0x12, 0x34, 0x50, // Payload Length (32 bytes)
            0x00, 0x20, // Next Header (17 = UDP), Hop Limit (64)
            0x11, 0x40, // Source Address (::1)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, // Destination Address (::1)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, // Payload (32 bytes of zeros for this test)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let packet = Ipv6Packet::try_from(&data[..]).unwrap();

        assert_eq!(packet.version(), 6);
        assert_eq!(packet.traffic_class(), 0x01);
        assert_eq!(packet.flow_label(), 0x23450);
        assert_eq!(packet.payload_length, 32);
        assert_eq!(packet.next_header, 0x11); // UDP
        assert_eq!(packet.hop_limit, 0x40); // 64
        assert_eq!(packet.source_addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(packet.dest_addr, Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(packet.payload.len(), 32);
    }

    #[test]
    fn test_invalid_version() {
        // Invalid version (4 instead of 6)
        let data = [
            0x40, 0x00, 0x00, 0x00, // Version 4
            0x00, 0x00, 0x11, 0x40, // Rest of header
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ];

        let result = Ipv6Packet::try_from(&data[..]);
        assert!(matches!(result, Err(Ipv6Error::InvalidVersion(4))));
    }

    #[test]
    fn test_invalid_length() {
        // Packet too short (only 39 bytes)
        let data = [0u8; 39];
        let result = Ipv6Packet::try_from(&data[..]);
        assert!(matches!(
            result,
            Err(Ipv6Error::InvalidLength {
                expected: 40,
                actual: 39
            })
        ));
    }

    /// Fixed IPv6 header with the given next header and payload length.
    fn ipv6_header(next_header: u8, payload_length: u16) -> Vec<u8> {
        let mut data = vec![0u8; 40];
        data[0] = 0x60;
        data[4] = (payload_length >> 8) as u8;
        data[5] = payload_length as u8;
        data[6] = next_header;
        data[7] = 64;
        data
    }

    #[test]
    fn test_hop_by_hop_extension_header_reaches_udp() {
        // Hop-by-Hop (8 bytes: next=17/UDP, len=0, padding) then 4 payload bytes.
        let mut data = ipv6_header(0, 12);
        data.extend_from_slice(&[17, 0, 0, 0, 0, 0, 0, 0]);
        data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let packet = Ipv6Packet::try_from(&data[..]).unwrap();

        assert_eq!(packet.next_header, 0);
        assert_eq!(packet.transport_protocol, Some(17));
        assert_eq!(packet.extension_headers.len(), 8);
        assert_eq!(packet.payload, &[0xde, 0xad, 0xbe, 0xef]);
        assert!(!packet.is_fragmented());
    }

    #[test]
    fn test_chained_extension_headers() {
        // Hop-by-Hop → Destination Options (16 bytes) → TCP.
        let mut data = ipv6_header(0, 8 + 16 + 4);
        data.extend_from_slice(&[60, 0, 0, 0, 0, 0, 0, 0]); // HbH → DestOpts
        data.extend_from_slice(&[6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // DestOpts → TCP
        data.extend_from_slice(&[1, 2, 3, 4]);

        let packet = Ipv6Packet::try_from(&data[..]).unwrap();

        assert_eq!(packet.transport_protocol, Some(6));
        assert_eq!(packet.extension_headers.len(), 24);
        assert_eq!(packet.payload, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_fragment_header_disables_transport_protocol() {
        // Fragment header (8 bytes, next=17/UDP, offset 185, more fragments).
        let mut data = ipv6_header(44, 12);
        data.extend_from_slice(&[17, 0, 0x05, 0xc9, 0x00, 0x00, 0x00, 0x01]);
        data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let packet = Ipv6Packet::try_from(&data[..]).unwrap();

        assert_eq!(packet.transport_protocol, None);
        assert!(packet.is_fragmented());
    }

    #[test]
    fn test_no_next_header_yields_no_transport_protocol() {
        let data = ipv6_header(59, 0);

        let packet = Ipv6Packet::try_from(&data[..]).unwrap();

        assert_eq!(packet.transport_protocol, None);
        assert!(!packet.is_fragmented());
    }

    #[test]
    fn test_truncated_extension_header_is_rejected() {
        // Hop-by-Hop claiming 8 bytes but only 4 available.
        let mut data = ipv6_header(0, 4);
        data.extend_from_slice(&[17, 0, 0, 0]);

        let result = Ipv6Packet::try_from(&data[..]);

        assert!(matches!(result, Err(Ipv6Error::InvalidExtensionHeader(_))));
    }

    #[test]
    fn test_invalid_payload_length() {
        // Packet with payload length longer than actual data
        let mut data = [0u8; 40];
        // Set version to 6
        data[0] = 0x60;
        // Set payload length to 100 bytes (but we only have 40)
        data[4] = 0x00;
        data[5] = 100;

        let result = Ipv6Packet::try_from(&data[..]);
        assert!(matches!(
            result,
            Err(Ipv6Error::InvalidPayloadLength {
                expected: 100,
                actual: 0
            })
        ));
    }
}
