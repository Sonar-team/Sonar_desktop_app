use thiserror::Error;

/// Errors that can occur when parsing or processing ARP packets
#[derive(Debug, Error, PartialEq)]
pub enum ArpError {
    /// The packet is too short to be a valid ARP packet
    #[error("Invalid ARP packet length: expected at least {expected} bytes, got {actual} bytes")]
    InvalidLength { expected: usize, actual: usize },

    /// The hardware type is not supported (only Ethernet is currently supported)
    #[error("Unsupported hardware type: {0}")]
    UnsupportedHardwareType(u16),

    /// The protocol type is not supported (only IPv4 is currently supported)
    #[error("Unsupported protocol type: {0:#06x}")]
    UnsupportedProtocolType(u16),

    /// The hardware length is invalid (expected 6 for Ethernet)
    #[error("Invalid hardware length: expected {expected}, got {actual}")]
    InvalidHardwareLength { expected: u8, actual: u8 },

    /// The protocol length is invalid (expected 4 for IPv4)
    #[error("Invalid protocol length: expected {expected}, got {actual}")]
    InvalidProtocolLength { expected: u8, actual: u8 },

    /// The operation is not supported (only request and reply are supported)
    #[error("Unsupported ARP operation: {0}")]
    UnsupportedOperation(u16),
}
