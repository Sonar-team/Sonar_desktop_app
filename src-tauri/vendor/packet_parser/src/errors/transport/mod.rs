use thiserror::Error;

use crate::errors::transport::{tcp::TcpError, udp::UdpError};

pub mod tcp;
pub mod udp;

/// Errors that can occur when parsing transport layer packets
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Packet is too short to be a valid transport packet")]
    PacketTooShort,

    #[error("Invalid TCP packet: {0}")]
    InvalidTcpPacket(String),

    #[error("UDP error: {0}")]
    UdpError(#[from] UdpError),

    #[error("TCP error: {0}")]
    TcpError(#[from] TcpError),

    #[error("Unsupported transport protocol")]
    UnsupportedProtocol,
}
