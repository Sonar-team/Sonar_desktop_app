// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::modbus_tcp::ModbusTcpError;

pub const MBAP_MIN_SIZE: usize = 7;

pub fn validate_mbap_min_size(value: &[u8]) -> Result<(), ModbusTcpError> {
    if value.len() < MBAP_MIN_SIZE {
        return Err(ModbusTcpError::BufferTooSmall {
            needed: MBAP_MIN_SIZE,
            actual: value.len(),
        });
    }

    Ok(())
}

pub fn validate_protocol_identifier(protocol_identifier: u16) -> Result<(), ModbusTcpError> {
    if protocol_identifier != 0 {
        return Err(ModbusTcpError::InvalidProtocolIdentifier {
            got: protocol_identifier,
        });
    }

    Ok(())
}

pub fn validate_length_field(length: u16) -> Result<(), ModbusTcpError> {
    if length < 1 {
        return Err(ModbusTcpError::InvalidLengthField { got: length });
    }

    Ok(())
}

pub fn validate_declared_total_length(value: &[u8], length: u16) -> Result<usize, ModbusTcpError> {
    let expected_total = 6usize + length as usize;
    if value.len() < expected_total {
        return Err(ModbusTcpError::LengthMismatch {
            expected: expected_total,
            actual: value.len(),
        });
    }

    Ok(expected_total)
}

pub fn validate_pdu_not_empty(pdu: &[u8]) -> Result<(), ModbusTcpError> {
    if pdu.is_empty() {
        return Err(ModbusTcpError::PduTooSmall {
            needed: 1,
            actual: pdu.len(),
        });
    }

    Ok(())
}

/// Vérifie que la progression de la boucle multi-ADU n'est pas nulle.
///
/// Conservée pour compatibilité d'API : le parseur ne l'appelle plus, car
/// `extract_length` garantit `length >= 1`, donc `consumed = 6 + length >= 7`
/// et la branche d'erreur est inatteignable depuis le parseur.
pub fn validate_consumed_length(consumed: usize, length: u16) -> Result<(), ModbusTcpError> {
    if consumed == 0 {
        return Err(ModbusTcpError::InvalidLengthField { got: length });
    }

    Ok(())
}

/// Vérifie que les octets 0-1 de l'en-tête MBAP sont présents et extrait le
/// Transaction Identifier (big-endian).
pub fn extract_transaction_identifier(value: &[u8]) -> Result<u16, ModbusTcpError> {
    if value.len() < 2 {
        return Err(ModbusTcpError::BufferTooSmall {
            needed: 2,
            actual: value.len(),
        });
    }

    Ok(u16::from_be_bytes([value[0], value[1]]))
}

/// Vérifie que le Protocol Identifier (octets 2-3, big-endian) vaut 0 et le
/// retourne.
pub fn extract_protocol_identifier(value: &[u8]) -> Result<u16, ModbusTcpError> {
    if value.len() < 4 {
        return Err(ModbusTcpError::BufferTooSmall {
            needed: 4,
            actual: value.len(),
        });
    }

    let protocol_identifier = u16::from_be_bytes([value[2], value[3]]);
    validate_protocol_identifier(protocol_identifier)?;

    Ok(protocol_identifier)
}

/// Vérifie que le champ Length (octets 4-5, big-endian) vaut au moins 1 et le
/// retourne.
pub fn extract_length(value: &[u8]) -> Result<u16, ModbusTcpError> {
    if value.len() < 6 {
        return Err(ModbusTcpError::BufferTooSmall {
            needed: 6,
            actual: value.len(),
        });
    }

    let length = u16::from_be_bytes([value[4], value[5]]);
    validate_length_field(length)?;

    Ok(length)
}

/// Vérifie que l'octet 6 de l'en-tête MBAP est présent et extrait le
/// Unit Identifier.
pub fn extract_unit_identifier(value: &[u8]) -> Result<u8, ModbusTcpError> {
    if value.len() < MBAP_MIN_SIZE {
        return Err(ModbusTcpError::BufferTooSmall {
            needed: MBAP_MIN_SIZE,
            actual: value.len(),
        });
    }

    Ok(value[6])
}

/// Vérifie que la PDU n'est pas vide puis la scinde en (function code,
/// données) sans copie.
pub fn extract_function_code(pdu: &[u8]) -> Result<(u8, &[u8]), ModbusTcpError> {
    validate_pdu_not_empty(pdu)?;

    Ok((pdu[0], &pdu[1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// En-tête MBAP + PDU de la trame 2 (requête Read Input Registers) :
    /// TI=0, PI=0, Length=6, Unit=255, Function=4, Reference=2258, WordCount=2.
    const FRAME_2: [u8; 12] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0xFF, 0x04, 0x08, 0xD2, 0x00, 0x02,
    ];

    #[test]
    fn test_extract_transaction_identifier() {
        assert_eq!(extract_transaction_identifier(&FRAME_2), Ok(0));
        assert!(matches!(
            extract_transaction_identifier(&[0x00]),
            Err(ModbusTcpError::BufferTooSmall {
                needed: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn test_extract_protocol_identifier() {
        assert_eq!(extract_protocol_identifier(&FRAME_2), Ok(0));
        assert!(matches!(
            extract_protocol_identifier(&[0x00, 0x01, 0x00, 0x02, 0x00, 0x01, 0x01]),
            Err(ModbusTcpError::InvalidProtocolIdentifier { got: 2 })
        ));
        assert!(matches!(
            extract_protocol_identifier(&[0x00, 0x00, 0x00]),
            Err(ModbusTcpError::BufferTooSmall {
                needed: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_extract_length() {
        assert_eq!(extract_length(&FRAME_2), Ok(6));
        assert!(matches!(
            extract_length(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF]),
            Err(ModbusTcpError::InvalidLengthField { got: 0 })
        ));
        assert!(matches!(
            extract_length(&[0x00, 0x00, 0x00, 0x00, 0x00]),
            Err(ModbusTcpError::BufferTooSmall {
                needed: 6,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_extract_unit_identifier() {
        assert_eq!(extract_unit_identifier(&FRAME_2), Ok(0xFF));
        assert!(matches!(
            extract_unit_identifier(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x06]),
            Err(ModbusTcpError::BufferTooSmall {
                needed: MBAP_MIN_SIZE,
                actual: 6
            })
        ));
    }

    #[test]
    fn test_extract_function_code() {
        // PDU de la trame 2 : function 0x04, données 08 D2 00 02.
        let (function_code, pdu_data) = extract_function_code(&FRAME_2[7..]).expect("PDU valide");
        assert_eq!(function_code, 0x04);
        assert_eq!(pdu_data, &[0x08, 0xD2, 0x00, 0x02]);

        assert!(matches!(
            extract_function_code(&[]),
            Err(ModbusTcpError::PduTooSmall {
                needed: 1,
                actual: 0
            })
        ));
    }
}
