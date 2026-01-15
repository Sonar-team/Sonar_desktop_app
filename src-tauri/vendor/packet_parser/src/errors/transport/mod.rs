use thiserror::Error;

/// Errors specific to UDP packet parsing
#[derive(Error, Debug, PartialEq)]
pub enum UdpError {
    /// The packet is too short to be a valid UDP packet
    #[error("UDP packet too short: expected at least {expected} bytes, got {actual} bytes")]
    PacketTooShort {
        /// The expected minimum size in bytes
        expected: usize,
        /// The actual size of the packet
        actual: usize,
    },

    /// The length field in the UDP header doesn't match the actual packet length
    #[error("UDP length field ({length}) doesn't match actual packet length ({actual})")]
    InvalidLength {
        /// The length specified in the UDP header
        length: u16,
        /// The actual length of the packet
        actual: usize,
    },

    /// The checksum in the UDP header is invalid
    #[error("Invalid UDP checksum")]
    InvalidChecksum,
}

/// Errors that can occur when parsing transport layer packets
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Packet is too short to be a valid transport packet")]
    PacketTooShort,

    #[error("Invalid TCP packet: {0}")]
    InvalidTcpPacket(String),

    #[error("UDP error: {0}")]
    UdpError(#[from] UdpError),

    #[error("Unsupported transport protocol")]
    UnsupportedProtocol,
}
