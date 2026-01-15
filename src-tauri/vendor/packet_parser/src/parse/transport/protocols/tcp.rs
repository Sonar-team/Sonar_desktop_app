use std::convert::TryFrom;
use thiserror::Error;

/// Represents a TCP header
#[derive(Debug, PartialEq)]
pub struct TcpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub sequence_number: u32,
    pub acknowledgment_number: u32,
    pub data_offset: u8, // 4 bits
    pub reserved: u8,    // 3 bits
    pub ns: bool,        // 1 bit
    pub cwr: bool,       // 1 bit
    pub ece: bool,       // 1 bit
    pub urg: bool,       // 1 bit
    pub ack: bool,       // 1 bit
    pub psh: bool,       // 1 bit
    pub rst: bool,       // 1 bit
    pub syn: bool,       // 1 bit
    pub fin: bool,       // 1 bit
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
    pub options: Vec<u8>,
}

/// Represents a TCP packet
#[derive(Debug)]
pub struct TcpPacket<'a> {
    pub header: TcpHeader,
    pub payload: &'a [u8],
}

#[derive(Error, Debug)]
pub enum TcpError {
    #[error("Packet too short to be a valid TCP header")]
    PacketTooShort,

    #[error("Invalid data offset: {0}")]
    InvalidDataOffset(u8),

    #[error("Invalid TCP header length")]
    InvalidHeaderLength,
}

impl<'a> TryFrom<&'a [u8]> for TcpPacket<'a> {
    type Error = TcpError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        // Minimum TCP header size is 20 bytes
        if packet.len() < 20 {
            return Err(TcpError::PacketTooShort);
        }

        let data_offset_words = packet[12] >> 4;

        // Validate data offset (must be between 5 and 15, inclusive)
        if !(5..=15).contains(&data_offset_words) {
            return Err(TcpError::InvalidDataOffset(data_offset_words));
        }

        let data_offset = (data_offset_words * 4) as usize;

        // Ensure packet is long enough for the header
        if packet.len() < data_offset {
            return Err(TcpError::PacketTooShort);
        }

        // Validate reserved bits (must be 0)
        let reserved = (packet[12] >> 1) & 0x07;
        if reserved != 0 {
            return Err(TcpError::InvalidHeaderLength);
        }

        // Check for invalid flag combinations that would never appear in real traffic
        let flags = packet[13];

        // SYN and FIN should never be set together in a valid TCP packet
        if (flags & 0x03) == 0x03 {
            // FIN and SYN both set
            return Err(TcpError::InvalidHeaderLength);
        }

        // If SYN is set, the sequence number should be the initial sequence number (ISN)
        // which is typically a random value, but we can check for all zeros which is invalid
        if (flags & 0x02) != 0
            && u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]) == 0
        {
            return Err(TcpError::InvalidHeaderLength);
        }

        // If ACK is not set, the acknowledgment number must be zero
        if (flags & 0x10) == 0
            && u32::from_be_bytes([packet[8], packet[9], packet[10], packet[11]]) != 0
        {
            return Err(TcpError::InvalidHeaderLength);
        }

        // Check for invalid window size (0 is technically valid but usually indicates a problem)
        let window_size = u16::from_be_bytes([packet[14], packet[15]]);
        if window_size == 0 && (flags & 0x02) == 0 {
            // Zero window size is only valid for SYN packets
            return Err(TcpError::InvalidHeaderLength);
        }

        // The rest of the header parsing...
        let header = TcpHeader {
            source_port: u16::from_be_bytes([packet[0], packet[1]]),
            destination_port: u16::from_be_bytes([packet[2], packet[3]]),
            sequence_number: u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]),
            acknowledgment_number: u32::from_be_bytes([
                packet[8], packet[9], packet[10], packet[11],
            ]),
            data_offset: data_offset_words,
            reserved,
            ns: (packet[12] & 0x01) != 0,
            cwr: (flags & 0x80) != 0,
            ece: (flags & 0x40) != 0,
            urg: (flags & 0x20) != 0,
            ack: (flags & 0x10) != 0,
            psh: (flags & 0x08) != 0,
            rst: (flags & 0x04) != 0,
            syn: (flags & 0x02) != 0,
            fin: (flags & 0x01) != 0,
            window_size,
            checksum: u16::from_be_bytes([packet[16], packet[17]]),
            urgent_pointer: u16::from_be_bytes([packet[18], packet[19]]),
            options: if data_offset > 20 {
                packet[20..data_offset].to_vec()
            } else {
                Vec::new()
            },
        };

        let payload = &packet[data_offset..];

        Ok(TcpPacket { header, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_packet_parsing() {
        // A simple TCP packet with no options
        let tcp_packet = [
            // Source port (1234)
            0x04, 0xD2, // Destination port (80)
            0x00, 0x50, // Sequence number (1)
            0x00, 0x00, 0x00, 0x01, // Acknowledgment number (0)
            0x00, 0x00, 0x00, 0x00,
            // Data offset (5 * 4 = 20 bytes), Reserved, NS=0, CWR=0, ECE=0, URG=0, ACK=1, PSH=0, RST=0, SYN=1, FIN=0
            0x50, 0x12, // Window size (8192)
            0x20, 0x00, // Checksum (0 for test)
            0x00, 0x00, // Urgent pointer (0)
            0x00, 0x00, // Payload (4 bytes)
            0x01, 0x02, 0x03, 0x04,
        ];

        let tcp = TcpPacket::try_from(&tcp_packet[..]).unwrap();

        assert_eq!(tcp.header.source_port, 1234);
        assert_eq!(tcp.header.destination_port, 80);
        assert_eq!(tcp.header.sequence_number, 1);
        assert_eq!(tcp.header.acknowledgment_number, 0);
        assert_eq!(tcp.header.data_offset, 5);
        assert!(tcp.header.ack);
        assert!(tcp.header.syn);
        assert!(!tcp.header.fin);
        assert_eq!(tcp.header.window_size, 8192);
        assert_eq!(tcp.header.checksum, 0);
        assert_eq!(tcp.header.urgent_pointer, 0);
        assert!(tcp.header.options.is_empty());
        assert_eq!(tcp.payload, &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_tcp_packet_too_short() {
        // Only 19 bytes (minimum is 20)
        let short_packet = [0u8; 19];
        let result = TcpPacket::try_from(&short_packet[..]);
        assert!(matches!(result, Err(TcpError::PacketTooShort)));
    }

    #[test]
    fn test_tcp_invalid_data_offset() {
        // Create a packet with invalid data offset (1 * 4 = 4 bytes, which is less than minimum 20)
        let mut packet = [0u8; 20];
        packet[12] = 0x10; // Data offset = 1 (4 bytes)
        let result = TcpPacket::try_from(&packet[..]);
        assert!(matches!(result, Err(TcpError::InvalidDataOffset(1))));
    }

    #[test]
    fn test_invalid_tcp_packet_form_icmp() {
        // This is an invalid TCP packet that should fail parsing
        let hex_str = "85001c45000000000101a2afb2c15c03";
        let packet_data = hex::decode(hex_str).expect("Failed to decode hex string");

        // The packet is too short (15 bytes) to be a valid TCP header (minimum 20 bytes)
        let result = TcpPacket::try_from(packet_data.as_slice());
        assert!(matches!(result, Err(TcpError::PacketTooShort)));
    }

    #[test]
    fn test_invalid_tcp_packet_form_icmp_with_data() {
        // This is an invalid TCP packet that should fail parsing
        let hex_str = "c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9fafbfcfdfeff000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f7071727374757677";
        let packet_data = hex::decode(hex_str).expect("Failed to decode hex string");

        // The packet is too short (15 bytes) to be a valid TCP header (minimum 20 bytes)
        let result = TcpPacket::try_from(packet_data.as_slice());
        println!("result: {:?}", result);
        assert!(matches!(result, Err(TcpError::InvalidHeaderLength)));
    }
}
