use thiserror::Error;

/// Errors that can occur when parsing or processing IPv4 packets
#[derive(Debug, Error, PartialEq)]
pub enum Ipv4Error {
    /// The packet is too short to be a valid IPv4 packet
    #[error("Invalid IPv4 packet length: expected at least {expected} bytes, got {actual} bytes")]
    InvalidLength {
        /// The minimum expected length in bytes
        expected: usize,
        /// The actual length of the packet in bytes
        actual: usize,
    },
    /// The IP version is not 4
    #[error("Invalid IP version: expected 4, got {0}")]
    InvalidVersion(u8),
    /// The header length is invalid (must be at least 20 bytes and a multiple of 4)
    #[error("Invalid header length: {0} bytes (must be between 20-60 bytes and a multiple of 4)")]
    InvalidHeaderLength(usize),
    /// The total length field is invalid
    #[error("Invalid total length: expected {expected} bytes (header: {min_header_len} + data: {}), but got {actual} bytes", expected - min_header_len)]
    InvalidTotalLength {
        /// The expected total length from the header
        expected: usize,
        /// The actual available data length
        actual: usize,
        /// The length of the header
        min_header_len: usize,
    },
    /// The header checksum is invalid
    #[error("Invalid header checksum: expected {expected:#06x}, got {actual:#06x}")]
    InvalidChecksum {
        /// The expected checksum value
        expected: u16,
        /// The actual checksum value
        actual: u16,
    },
    /// The TTL (Time To Live) has expired
    #[error("TTL expired: {0}")]
    TtlExpired(u8),
    /// The protocol is not supported
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(u8),
    /// Invalid IP option
    #[error("Invalid IP option: {0}")]
    InvalidOption(String),
    /// The packet is a fragment and requires reassembly
    #[error("Fragmented packet - requires reassembly (offset: {0}, more fragments: {1})")]
    FragmentedPacket(u16, bool),
    /// The packet exceeds the maximum allowed size
    #[error("Packet too large: {0} bytes (maximum allowed: {1} bytes)")]
    PacketTooLarge(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = Ipv4Error::InvalidLength {
            expected: 20,
            actual: 10,
        };
        assert_eq!(
            err.to_string(),
            "Invalid IPv4 packet length: expected at least 20 bytes, got 10 bytes"
        );

        let err = Ipv4Error::InvalidVersion(6);
        assert_eq!(err.to_string(), "Invalid IP version: expected 4, got 6");

        let err = Ipv4Error::InvalidHeaderLength(19);
        assert_eq!(
            err.to_string(),
            "Invalid header length: 19 bytes (must be between 20-60 bytes and a multiple of 4)"
        );

        let err = Ipv4Error::InvalidTotalLength {
            expected: 100,
            actual: 50,
            min_header_len: 20,
        };
        assert_eq!(
            err.to_string(),
            "Invalid total length: expected 100 bytes (header: 20 + data: 80), but got 50 bytes"
        );

        let err = Ipv4Error::InvalidChecksum {
            expected: 0x1234,
            actual: 0x5678,
        };
        assert_eq!(
            err.to_string(),
            "Invalid header checksum: expected 0x1234, got 0x5678"
        );

        let err = Ipv4Error::TtlExpired(0);
        assert_eq!(err.to_string(), "TTL expired: 0");

        let err = Ipv4Error::UnsupportedProtocol(123);
        assert_eq!(err.to_string(), "Unsupported protocol: 123");

        let err = Ipv4Error::InvalidOption("invalid option".to_string());
        assert_eq!(err.to_string(), "Invalid IP option: invalid option");

        let err = Ipv4Error::FragmentedPacket(1480, true);
        assert_eq!(
            err.to_string(),
            "Fragmented packet - requires reassembly (offset: 1480, more fragments: true)"
        );

        let err = Ipv4Error::PacketTooLarge(65536, 1500);
        assert_eq!(
            err.to_string(),
            "Packet too large: 65536 bytes (maximum allowed: 1500 bytes)"
        );
    }
}
