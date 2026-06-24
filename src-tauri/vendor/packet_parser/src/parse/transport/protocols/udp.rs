// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::convert::TryFrom;

use crate::{
    checks::transport::udp::{UDP_HEADER_SIZE, validate_udp_length, validate_udp_min_length},
    errors::transport::udp::UdpError,
};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// UDP Packet
///
/// ```mermaid
/// ---
/// title: UdpPacket
/// ---
/// packet-beta
/// 0-15: "Source Port u16"
/// 16-31: "Destination Port u16"
/// 32-47: "Length u16"
/// 48-63: "Checksum u16"
/// 64-127: "Payload variable"
/// ```
#[derive(Debug)]
pub struct UdpPacket<'a> {
    /// Source port
    pub source_port: u16,
    /// Destination port
    pub destination_port: u16,
    /// Length of the UDP header and data in bytes
    pub length: u16,
    /// Checksum for error-checking of the header and data
    pub checksum: u16,
    /// The payload of the UDP packet
    pub payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for UdpPacket<'a> {
    type Error = UdpError;

    /// Attempts to parse a byte slice into a UdpPacket
    ///
    /// # Arguments
    /// * `data` - The byte slice containing the UDP packet
    ///
    /// # Returns
    /// * `Result<UdpPacket, TransportError>` - The parsed UDP packet or an error
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        validate_udp_min_length(data)?;

        let source_port = u16::from_be_bytes([data[0], data[1]]);
        let destination_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);

        validate_udp_length(length, data.len())?;

        // The payload is everything after the 8-byte header
        let payload = &data[UDP_HEADER_SIZE..];

        Ok(UdpPacket {
            source_port,
            destination_port,
            length,
            checksum,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_packet_parsing() {
        // A sample UDP packet (source port 1234, dest port 80, length 20, checksum 0x1234)
        let data = [
            0x04, 0xD2, // source port 1234
            0x00, 0x50, // dest port 80
            0x00, 0x14, // length 20
            0x12, 0x34, // checksum
            0x01, 0x02, 0x03, 0x04, 0x05, // payload
            0x06, 0x07, 0x08, 0x09, 0x0A, // more payload
            0x0B, 0x0C, // more payload
        ];

        let udp_packet = UdpPacket::try_from(&data[..]).unwrap();

        assert_eq!(udp_packet.source_port, 1234);
        assert_eq!(udp_packet.destination_port, 80);
        assert_eq!(udp_packet.length, 20);
        assert_eq!(udp_packet.checksum, 0x1234);
        assert_eq!(
            udp_packet.payload,
            &[
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C
            ]
        );
    }

    #[test]
    fn test_udp_packet_too_short() {
        let data = [0x04, 0xD2, 0x00, 0x50]; // Only 4 bytes
        let result = UdpPacket::try_from(&data[..]);
        assert!(matches!(
            result,
            Err(UdpError::PacketTooShort {
                expected: 8,
                actual: 4
            })
        ));
    }

    #[test]
    fn test_udp_packet_invalid_length() {
        let data = [
            0x04, 0xD2, // source port
            0x00, 0x50, // dest port
            0x00, 0x10, // length 16 (but actual data is longer)
            0x12, 0x34, // checksum
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
        ];

        let result = UdpPacket::try_from(&data[..]);
        assert!(matches!(
            result,
            Err(UdpError::InvalidLength {
                length: 16,
                actual: 18
            })
        ));
    }
}
