// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::bitcoin::BitcoinError;

const VALID_MAGIC_NUMBERS: [u32; 5] = [
    0xD9B4BEF9, // Mainnet
    0x0709110B, // Testnet
    0x0B110907, // Testnet3
    0xFABFB5DA, // Regtest
    0x40CF030A, // Signet
];

const BITCOIN_HEADER_LENGTH: usize = 24;

pub fn check_minimum_length(payload: &[u8]) -> Result<(), BitcoinError> {
    if payload.len() < BITCOIN_HEADER_LENGTH {
        return Err(BitcoinError::PacketTooShort {
            actual: payload.len(),
        });
    }

    Ok(())
}

pub fn check_magic_number(payload: &[u8]) -> Result<u32, BitcoinError> {
    if payload.len() < 4 {
        return Err(BitcoinError::PacketTooShort {
            actual: payload.len(),
        });
    }

    let magic = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    if VALID_MAGIC_NUMBERS.contains(&magic) {
        Ok(magic)
    } else {
        Err(BitcoinError::InvalidMagic { magic })
    }
}

pub fn validate_total_length(packet: &[u8], payload_len: u32) -> Result<(), BitcoinError> {
    let expected = BITCOIN_HEADER_LENGTH + payload_len as usize;
    if packet.len() != expected {
        return Err(BitcoinError::LengthMismatch {
            declared: payload_len,
            actual_payload_len: packet.len().saturating_sub(BITCOIN_HEADER_LENGTH),
            actual_total_len: packet.len(),
        });
    }

    Ok(())
}

pub fn validate_command_bytes(bytes: &[u8]) -> Result<(), BitcoinError> {
    let mut saw_nul = false;

    for &byte in bytes {
        if byte == 0 {
            saw_nul = true;
            continue;
        }

        if saw_nul {
            return Err(BitcoinError::NonZeroPaddingAfterNull);
        }

        if !byte.is_ascii_alphanumeric() {
            return Err(BitcoinError::InvalidCommandBytes);
        }
    }

    Ok(())
}

/// Validates the 12-byte NUL-padded ASCII command field and returns the
/// command borrowed as `&str`, with trailing NUL padding trimmed.
///
/// Zero-copy: no allocation, the returned `&str` points into the original packet.
pub fn parse_command_bytes(bytes: &[u8]) -> Result<&str, BitcoinError> {
    validate_command_bytes(bytes)?;

    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    core::str::from_utf8(&bytes[..end]).map_err(|_| BitcoinError::InvalidCommandBytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_minimum_length_boundaries() {
        assert!(check_minimum_length(&[0u8; 24]).is_ok());
        assert!(matches!(
            check_minimum_length(&[0u8; 23]),
            Err(BitcoinError::PacketTooShort { actual: 23 })
        ));
    }

    #[test]
    fn test_check_magic_number_valid_and_invalid() {
        assert_eq!(
            check_magic_number(&[0xF9, 0xBE, 0xB4, 0xD9]).unwrap(),
            0xD9B4BEF9
        );
        assert!(matches!(
            check_magic_number(&[0x99, 0xBE, 0xB4, 0xD9]),
            Err(BitcoinError::InvalidMagic { magic: 0xD9B4BE99 })
        ));
    }

    #[test]
    fn test_check_magic_number_too_short() {
        assert!(matches!(
            check_magic_number(&[0xF9, 0xBE]),
            Err(BitcoinError::PacketTooShort { actual: 2 })
        ));
    }

    #[test]
    fn test_validate_total_length_ok_and_mismatch() {
        assert!(validate_total_length(&[0u8; 24], 0).is_ok());
        assert!(validate_total_length(&[0u8; 29], 5).is_ok());
        assert!(matches!(
            validate_total_length(&[0u8; 24], 5),
            Err(BitcoinError::LengthMismatch {
                declared: 5,
                actual_payload_len: 0,
                actual_total_len: 24
            })
        ));
    }

    #[test]
    fn test_validate_total_length_shorter_than_header() {
        // saturating_sub: pas d'underflow si le paquet est plus court que le header
        assert!(matches!(
            validate_total_length(&[0u8; 10], 5),
            Err(BitcoinError::LengthMismatch {
                declared: 5,
                actual_payload_len: 0,
                actual_total_len: 10
            })
        ));
    }

    #[test]
    fn test_validate_command_bytes_valid() {
        assert!(validate_command_bytes(b"verack\0\0\0\0\0\0").is_ok());
        assert!(validate_command_bytes(&[0u8; 12]).is_ok());
    }

    #[test]
    fn test_validate_command_bytes_non_ascii() {
        let mut bytes = *b"verack\0\0\0\0\0\0";
        bytes[2] = 0xC3; // non-ASCII embarque dans la commande
        assert!(matches!(
            validate_command_bytes(&bytes),
            Err(BitcoinError::InvalidCommandBytes)
        ));
    }

    #[test]
    fn test_validate_command_bytes_non_zero_padding() {
        assert!(matches!(
            validate_command_bytes(b"verack\0x\0\0\0\0"),
            Err(BitcoinError::NonZeroPaddingAfterNull)
        ));
    }

    #[test]
    fn test_parse_command_bytes_trims_and_borrows() {
        let bytes = b"verack\0\0\0\0\0\0";
        let command = parse_command_bytes(bytes).expect("commande valide");
        assert_eq!(command, "verack");
        // zero-copy : la reference pointe dans le buffer d'origine
        assert_eq!(command.as_ptr(), bytes.as_ptr());
    }

    #[test]
    fn test_parse_command_bytes_full_width_no_nul() {
        assert_eq!(
            parse_command_bytes(b"abcdefghijkl").unwrap(),
            "abcdefghijkl"
        );
    }

    #[test]
    fn test_parse_command_bytes_invalid() {
        assert!(matches!(
            parse_command_bytes(&[0xFFu8; 12]),
            Err(BitcoinError::InvalidCommandBytes)
        ));
    }
}
