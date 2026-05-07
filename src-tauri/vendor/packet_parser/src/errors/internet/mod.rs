use arp::ArpError;
use thiserror::Error;
pub mod arp;
pub mod ipv4;
pub mod ipv6;
/// Errors that can occur when parsing or processing internet layer protocols
#[derive(Debug, Error)]
pub enum InternetError {
    /// Error related to ARP protocol
    #[error("ARP error: {0}")]
    ArpError(#[from] ArpError),

    /// The packet is too short to be a valid internet protocol packet
    #[error("Invalid packet length: expected at least {expected} bytes, got {actual} bytes")]
    InvalidLength { expected: usize, actual: usize },

    /// The packet is empty
    #[error("Empty packet")]
    EmptyPacket,

    /// The protocol is not supported
    #[error("Unsupported transport protocol")]
    UnsupportedProtocol,

    /// The packet format is invalid
    #[error("Invalid packet format: {0}")]
    InvalidFormat(String),

    /// The checksum is invalid
    #[error("Invalid checksum")]
    InvalidChecksum,
}
