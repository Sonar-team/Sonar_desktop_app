use crate::errors::internet::ipv6::Ipv6Error;
use std::convert::TryFrom;
use std::net::Ipv6Addr;

/// IPv6 Packet Structure
///
/// Represents an Internet Protocol version 6 packet
#[derive(Debug, PartialEq)]
pub struct Ipv6Packet<'a> {
    /// Version (6 for IPv6), Traffic Class, and Flow Label
    pub version_tc_flow: [u8; 4],
    /// Payload Length
    pub payload_length: u16,
    /// Next Header (similar to IPv4's protocol field)
    pub next_header: u8,
    /// Hop Limit (similar to IPv4's TTL)
    pub hop_limit: u8,
    /// Source Address
    pub source_addr: Ipv6Addr,
    /// Destination Address
    pub dest_addr: Ipv6Addr,
    /// Extension Headers (if any)
    pub extension_headers: Vec<u8>,
    /// Payload data
    pub payload: &'a [u8],
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

    /// Returns the Flow Label
    pub fn flow_label(&self) -> u32 {
        ((self.version_tc_flow[1] as u32 & 0x0F) << 16)
            | ((self.version_tc_flow[2] as u32) << 8)
            | (self.version_tc_flow[3] as u32)
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
        // IPv6 header is fixed at 40 bytes
        const IPV6_HEADER_LEN: usize = 40;

        if data.len() < IPV6_HEADER_LEN {
            return Err(Ipv6Error::InvalidLength {
                expected: IPV6_HEADER_LEN,
                actual: data.len(),
            });
        }

        let version = data[0] >> 4;
        if version != 6 {
            return Err(Ipv6Error::InvalidVersion(version));
        }

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

        // Check if we have enough data for the payload
        let total_expected_len = IPV6_HEADER_LEN + payload_length as usize;
        if data.len() < total_expected_len {
            return Err(Ipv6Error::InvalidPayloadLength {
                expected: payload_length,
                actual: data.len().saturating_sub(IPV6_HEADER_LEN),
            });
        }

        // For simplicity, we're not parsing extension headers here
        // In a real implementation, you would parse them based on next_header
        let extension_headers = Vec::new();

        // Extract payload
        let payload = if payload_length > 0 {
            &data[IPV6_HEADER_LEN..total_expected_len]
        } else {
            &[]
        };

        Ok(Ipv6Packet {
            version_tc_flow,
            payload_length,
            next_header,
            hop_limit,
            source_addr,
            dest_addr,
            extension_headers,
            payload,
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
