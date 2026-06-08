use crate::errors::transport::tcp::TcpError;

const TCP_MIN_LENGTH: usize = 20;

pub fn validate_tcp_min_length(packet: &[u8]) -> Result<(), TcpError> {
    if packet.len() < TCP_MIN_LENGTH {
        return Err(TcpError::PacketTooShort);
    }
    Ok(())
}
