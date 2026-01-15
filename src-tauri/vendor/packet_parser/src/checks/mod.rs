// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::ParsedPacketError;
pub mod application;
pub mod data_link;
pub fn validate_packet_length(packets: &[u8]) -> Result<(), ParsedPacketError> {
    if packets.len() < 14 {
        return Err(ParsedPacketError::PacketTooShort(packets.len() as u8));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_packet_length;
    use crate::errors::ParsedPacketError;

    #[test]
    fn test_validate_packet_length_too_short() {
        let short_packet = vec![0x00, 0x11, 0x22]; // Only 3 bytes, should fail
        let result = validate_packet_length(&short_packet);
        assert!(matches!(result, Err(ParsedPacketError::PacketTooShort(_))));
    }

    #[test]
    fn test_validate_packet_length_valid() {
        let valid_packet = vec![0x00; 14]; // Exactly 14 bytes, should pass
        let result = validate_packet_length(&valid_packet);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_packet_length_long() {
        let long_packet = vec![0x00; 100]; // More than 14 bytes, should pass
        let result = validate_packet_length(&long_packet);
        assert!(result.is_ok());
    }
}
