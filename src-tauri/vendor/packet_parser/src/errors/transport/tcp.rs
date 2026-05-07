use thiserror::Error;

#[derive(Error, Debug)]
pub enum TcpError {
    #[error("Packet too short to be a valid TCP header")]
    PacketTooShort,

    #[error("Invalid data offset: {0}")]
    InvalidDataOffset(u8),

    #[error("Invalid TCP header length")]
    InvalidHeaderLength,
}
