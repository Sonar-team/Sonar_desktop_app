// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    checks::application::bitcoin::{
        check_magic_number, check_minimum_length, validate_command_bytes, validate_total_length,
    },
    errors::application::bitcoin::BitcoinError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// Bitcoin Network Packet
///
/// ```mermaid
/// ---
/// title: BitcoinPacket
/// ---
/// packet-beta
/// 0-31: "Magic u32"
/// 32-127: "Command bytes[12]"
/// 128-159: "Payload Length u32"
/// 160-191: "Checksum bytes[4]"
/// 192-255: "Payload variable"
/// ```
///
/// The `BitcoinPacket` struct represents a parsed Bitcoin packet.
#[derive(Debug)]
pub struct BitcoinPacket {
    pub magic: u32,
    pub command: String,
    pub length: u32,
    pub checksum: [u8; 4],
    pub payload: Vec<u8>,
}

/// Extracts the command string from the payload (12 bytes, null-padded ASCII)
fn extract_command(payload: &[u8]) -> Result<String, BitcoinError> {
    let bytes = &payload[4..16];
    validate_command_bytes(bytes)?;

    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let command = std::str::from_utf8(&bytes[..end])
        .map_err(|_| BitcoinError::InvalidCommandBytes)?
        .to_string();

    Ok(command)
}

/// Extracts the length of the payload from the header (4 bytes)
fn extract_length(payload: &[u8]) -> u32 {
    u32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]])
}

/// Extracts the checksum from the header (4 bytes)
fn extract_checksum(payload: &[u8]) -> [u8; 4] {
    [payload[20], payload[21], payload[22], payload[23]]
}

/// Extracts the actual payload data
fn extract_payload(payload: &[u8]) -> Vec<u8> {
    payload[24..].to_vec()
}

/// Parses a Bitcoin packet from a given payload.
///
/// # Arguments
///
/// * `payload` - A byte slice representing the raw Bitcoin packet data.
///
/// # Returns
///
/// * `Result<BitcoinPacket, BitcoinError>` - Returns `Ok(BitcoinPacket)` if parsing is successful,
///   otherwise returns a typed `BitcoinError`.
impl TryFrom<&[u8]> for BitcoinPacket {
    type Error = BitcoinError;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        check_minimum_length(payload)?;
        let magic = check_magic_number(payload)?;
        let command = extract_command(payload)?;
        let checksum = extract_checksum(payload);

        let length = extract_length(payload);
        validate_total_length(payload, length)?;
        let actual_payload = extract_payload(payload);

        Ok(BitcoinPacket {
            magic,
            command,
            length,
            checksum,
            payload: actual_payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_checksum() {
        // Test with a valid payload containing a known checksum
        let payload = vec![
            0xF9, 0xBE, 0xB4, 0xD9, // Magic number (mainnet)
            0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, // Command ("verack")
            0x00, 0x00, 0x00, 0x00, // Length (0)
            0x5D, 0xF6, 0xE0, 0xE2, // Checksum (example)
        ];
        let expected_checksum = [0x5D, 0xF6, 0xE0, 0xE2];
        let extracted_checksum = extract_checksum(&payload);
        assert_eq!(extracted_checksum, expected_checksum);
    }

    #[test]
    fn test_extract_checksum_incorrect_length() {
        // Test with a payload shorter than required length for checksum extraction
        let payload = vec![0xF9, 0xBE, 0xB4]; // Only 3 bytes, should fail
        let result = std::panic::catch_unwind(|| extract_checksum(&payload));
        assert!(
            result.is_err(),
            "Expected panic due to short payload length"
        );
    }

    /// Tests for the `parse_bitcoin_packet` function.
    #[test]
    fn test_valid_bitcoin_packet() {
        // Test with a valid Bitcoin packet (simplified example)
        let bitcoin_payload = vec![
            0xF9, 0xBE, 0xB4, 0xD9, // Magic number (mainnet)
            0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, // Command ("verack")
            0x00, 0x00, 0x00, 0x00, // Length (0)
            0x5D, 0xF6, 0xE0, 0xE2, // Checksum (example)
        ];
        match BitcoinPacket::try_from(bitcoin_payload.as_slice()) {
            Ok(packet) => {
                assert_eq!(packet.magic, 3652501241);
                assert_eq!(packet.command, "verack");
                assert_eq!(packet.length, 0);
                assert_eq!(packet.checksum, [0x5D, 0xF6, 0xE0, 0xE2]);
                assert_eq!(packet.payload.len(), 0);
            }
            Err(_) => panic!("Expected Bitcoin packet"),
        }
    }

    #[test]
    fn test_invalid_magic_number() {
        // Test with an invalid magic number
        let invalid_magic_number = vec![
            0x99, 0xBE, 0xB4, 0xD9, // Invalid magic number
            0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, // Command ("verack")
            0x00, 0x00, 0x00, 0x00, // Length (0)
            0x5D, 0xF6, 0xE0, 0xE2, // Checksum (example)
        ];
        let err = BitcoinPacket::try_from(invalid_magic_number.as_slice()).unwrap_err();
        assert!(matches!(
            err,
            BitcoinError::InvalidMagic { magic: 0xD9B4BE99 }
        ));
    }

    #[test]
    fn test_short_payload() {
        // Test with a payload length shorter than 24 bytes
        let short_payload = vec![0xF9, 0xBE, 0xB4]; // Only 3 bytes, should be at least 24
        match BitcoinPacket::try_from(short_payload.as_slice()) {
            Ok(_) => panic!("Expected non-Bitcoin packet due to short payload"),
            Err(is_bitcoin) => assert!(is_bitcoin == BitcoinError::PacketTooShort { actual: 3 }),
        }
    }

    #[test]
    fn test_invalid_length() {
        // header complet (24 bytes) mais length=5 et payload absent => mismatch
        let invalid_length = vec![
            0xF9, 0xBE, 0xB4, 0xD9, // Magic
            0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, // "verack"
            0x05, 0x00, 0x00, 0x00, // length=5
            0x00, 0x00, 0x00, 0x00, // checksum (dummy)
        ];

        let err = BitcoinPacket::try_from(invalid_length.as_slice()).unwrap_err();
        assert!(matches!(
            err,
            BitcoinError::LengthMismatch { declared: 5, .. }
        ));
    }

    #[test]
    fn test_check_minimum_length() {
        assert!(check_minimum_length(&[0u8; 24]).is_ok());
        assert!(check_minimum_length(&[0u8; 23]).is_err());
    }

    #[test]
    fn test_check_magic_number() {
        assert_eq!(
            check_magic_number(&[0xF9, 0xBE, 0xB4, 0xD9]).unwrap(),
            0xD9B4BEF9
        );
        assert!(check_magic_number(&[0x99, 0xBE, 0xB4, 0xD9]).is_err());
    }

    #[test]
    fn test_extract_command() {
        assert_eq!(
            extract_command(&[
                0xF9, 0xBE, 0xB4, 0xD9, 0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ])
            .unwrap(),
            "verack"
        );
    }

    #[test]
    fn test_extract_length() {
        assert_eq!(
            extract_length(&[
                0xF9, 0xBE, 0xB4, 0xD9, // Magic number (4 bytes)
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, // Command (12 bytes)
                0x05, 0x00, 0x00, 0x00, // Length (4 bytes, little-endian, 5 in this case)
            ]),
            5
        );
    }

    #[test]
    fn test_extract_payload() {
        assert_eq!(
            extract_payload(&[
                0xF9, 0xBE, 0xB4, 0xD9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04,
                0x05
            ]),
            vec![0x01, 0x02, 0x03, 0x04, 0x05]
        );
    }
}
