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

pub fn check_minimum_length(payload: &[u8]) -> Result<(), BitcoinError> {
    if payload.len() < 24 {
        return Err(BitcoinError::PacketTooShort {
            actual: payload.len(),
        });
    }

    Ok(())
}

pub fn check_magic_number(payload: &[u8]) -> Result<u32, BitcoinError> {
    let magic = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    if VALID_MAGIC_NUMBERS.contains(&magic) {
        Ok(magic)
    } else {
        Err(BitcoinError::InvalidMagic { magic })
    }
}

pub fn validate_total_length(packet: &[u8], payload_len: u32) -> Result<(), BitcoinError> {
    let expected = 24usize + payload_len as usize;
    if packet.len() != expected {
        return Err(BitcoinError::LengthMismatch {
            declared: payload_len,
            actual_payload_len: packet.len() - 24,
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
