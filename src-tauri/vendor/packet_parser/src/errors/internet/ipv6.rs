use thiserror::Error;

/// Errors that can occur when parsing or processing IPv6 packets
#[derive(Debug, Error, PartialEq)]
pub enum Ipv6Error {
    /// The packet is too short to be a valid IPv6 packet
    #[error("Invalid IPv6 packet length: expected at least {expected} bytes, got {actual} bytes")]
    InvalidLength {
        /// The minimum expected length in bytes
        expected: usize,
        /// The actual length of the packet in bytes
        actual: usize,
    },

    /// The IP version is not 6
    #[error("Invalid IP version: expected 6, got {0}")]
    InvalidVersion(u8),

    /// The payload length is invalid
    #[error(
        "Invalid payload length: expected {expected} bytes, but packet only has {actual} bytes"
    )]
    InvalidPayloadLength {
        /// The expected payload length from the header
        expected: u16,
        /// The actual available payload length
        actual: usize,
    },

    /// The hop limit has expired
    #[error("Hop limit expired: {0}")]
    HopLimitExpired(u8),

    /// The next header is not supported
    #[error("Unsupported next header: {0}")]
    UnsupportedNextHeader(u8),

    /// The extension header is invalid or not supported
    #[error("Invalid extension header: {0}")]
    InvalidExtensionHeader(String),

    /// The packet exceeds the maximum allowed size
    #[error("Packet too large: {0} bytes (maximum allowed: {1} bytes)")]
    PacketTooLarge(usize, usize),

    /// Invalid IPv6 address
    #[error("Invalid IPv6 address: {0}")]
    InvalidAddress(String),
}
