// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::ams::AmsParseError;

pub const AMS_HEADER_LEN: usize = 32;

pub fn validate_ams_header_length(len: usize) -> Result<(), AmsParseError> {
    if len < AMS_HEADER_LEN {
        return Err(AmsParseError::HeaderTooShort {
            expected: AMS_HEADER_LEN,
            actual: len,
        });
    }

    Ok(())
}

pub fn validate_cb_data_length(cb_data: u32, actual: usize) -> Result<(), AmsParseError> {
    if actual != cb_data as usize {
        return Err(AmsParseError::InvalidCbDataLength { cb_data, actual });
    }

    Ok(())
}

pub fn validate_cmd_id(cmd_id: u16) -> Result<(), AmsParseError> {
    if !matches!(cmd_id, 0x0001..=0x0009) {
        return Err(AmsParseError::UnknownCommand(cmd_id));
    }

    Ok(())
}

pub fn validate_state_flags(flags: u16) -> Result<(), AmsParseError> {
    let reserved_mask: u16 = !0x000F;
    if flags & reserved_mask != 0 {
        return Err(AmsParseError::InvalidStateFlags(flags));
    }

    Ok(())
}

/// Vérifie que la slice du champ Target Net ID contient bien 6 octets et
/// retourne l'identifiant brut. Inatteignable en erreur après
/// `validate_ams_header_length` (le champ occupe les octets 0..6 du header).
pub fn extract_target_net_id(payload: &[u8]) -> Result<[u8; 6], AmsParseError> {
    if payload.len() < 6 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 6,
            actual: payload.len(),
        });
    }
    Ok([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5],
    ])
}

/// Vérifie que la slice du champ Target Port contient bien 2 octets et
/// retourne le port décodé en little-endian (octets 6..8 du header).
pub fn extract_target_port(payload: &[u8]) -> Result<u16, AmsParseError> {
    if payload.len() < 2 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 2,
            actual: payload.len(),
        });
    }
    Ok(u16::from_le_bytes([payload[0], payload[1]]))
}

/// Vérifie que la slice du champ Sender Net ID contient bien 6 octets et
/// retourne l'identifiant brut (octets 8..14 du header).
pub fn extract_sender_net_id(payload: &[u8]) -> Result<[u8; 6], AmsParseError> {
    if payload.len() < 6 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 6,
            actual: payload.len(),
        });
    }
    Ok([
        payload[0], payload[1], payload[2], payload[3], payload[4], payload[5],
    ])
}

/// Vérifie que la slice du champ Sender Port contient bien 2 octets et
/// retourne le port décodé en little-endian (octets 14..16 du header).
pub fn extract_sender_port(payload: &[u8]) -> Result<u16, AmsParseError> {
    if payload.len() < 2 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 2,
            actual: payload.len(),
        });
    }
    Ok(u16::from_le_bytes([payload[0], payload[1]]))
}

/// Vérifie que la slice du champ Command ID contient bien 2 octets et
/// retourne la valeur décodée en little-endian (octets 16..18 du header).
/// Décodage seul : l'appartenance aux commandes connues reste vérifiée par
/// `validate_cmd_id`, appelée après l'extraction pour préserver la
/// précédence d'erreurs.
pub fn extract_cmd_id(payload: &[u8]) -> Result<u16, AmsParseError> {
    if payload.len() < 2 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 2,
            actual: payload.len(),
        });
    }
    Ok(u16::from_le_bytes([payload[0], payload[1]]))
}

/// Vérifie que la slice du champ State Flags contient bien 2 octets et
/// retourne la valeur décodée en little-endian (octets 18..20 du header).
/// Décodage seul : les bits réservés restent vérifiés par
/// `validate_state_flags`, appelée après l'extraction pour préserver la
/// précédence d'erreurs.
pub fn extract_state_flags(payload: &[u8]) -> Result<u16, AmsParseError> {
    if payload.len() < 2 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 2,
            actual: payload.len(),
        });
    }
    Ok(u16::from_le_bytes([payload[0], payload[1]]))
}

/// Vérifie que la slice du champ cbData contient bien 4 octets et retourne
/// la longueur déclarée décodée en little-endian (octets 20..24 du header).
/// La cohérence avec la longueur réelle des données reste vérifiée par
/// `validate_cb_data_length`.
pub fn extract_cb_data(payload: &[u8]) -> Result<u32, AmsParseError> {
    if payload.len() < 4 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 4,
            actual: payload.len(),
        });
    }
    Ok(u32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

/// Vérifie que la slice du champ Error Code contient bien 4 octets et
/// retourne le code décodé en little-endian (octets 24..28 du header).
pub fn extract_error_code(payload: &[u8]) -> Result<u32, AmsParseError> {
    if payload.len() < 4 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 4,
            actual: payload.len(),
        });
    }
    Ok(u32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

/// Vérifie que la slice du champ Invoke ID contient bien 4 octets et
/// retourne l'identifiant décodé en little-endian (octets 28..32 du header).
pub fn extract_invoke_id(payload: &[u8]) -> Result<u32, AmsParseError> {
    if payload.len() < 4 {
        return Err(AmsParseError::HeaderTooShort {
            expected: 4,
            actual: payload.len(),
        });
    }
    Ok(u32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_net_ids_valid() {
        assert_eq!(
            extract_target_net_id(&[0, 0, 0, 0, 0, 130]).unwrap(),
            [0, 0, 0, 0, 0, 130]
        );
        assert_eq!(
            extract_sender_net_id(&[192, 168, 1, 1, 1, 1]).unwrap(),
            [192, 168, 1, 1, 1, 1]
        );
    }

    #[test]
    fn test_extract_net_ids_too_short() {
        assert!(matches!(
            extract_target_net_id(&[0, 0, 0]),
            Err(AmsParseError::HeaderTooShort {
                expected: 6,
                actual: 3
            })
        ));
        assert!(matches!(
            extract_sender_net_id(&[]),
            Err(AmsParseError::HeaderTooShort {
                expected: 6,
                actual: 0
            })
        ));
    }

    #[test]
    fn test_extract_ports_little_endian() {
        assert_eq!(extract_target_port(&851u16.to_le_bytes()).unwrap(), 851);
        assert_eq!(extract_sender_port(&33000u16.to_le_bytes()).unwrap(), 33000);
    }

    #[test]
    fn test_extract_u16_fields_too_short() {
        assert!(matches!(
            extract_target_port(&[0x01]),
            Err(AmsParseError::HeaderTooShort {
                expected: 2,
                actual: 1
            })
        ));
        assert!(matches!(
            extract_sender_port(&[]),
            Err(AmsParseError::HeaderTooShort {
                expected: 2,
                actual: 0
            })
        ));
        assert!(matches!(
            extract_cmd_id(&[]),
            Err(AmsParseError::HeaderTooShort {
                expected: 2,
                actual: 0
            })
        ));
        assert!(matches!(
            extract_state_flags(&[0x04]),
            Err(AmsParseError::HeaderTooShort {
                expected: 2,
                actual: 1
            })
        ));
    }

    #[test]
    fn test_extract_cmd_id_and_state_flags_decode_only() {
        // Décodage seul : la validation sémantique reste portée par
        // validate_cmd_id / validate_state_flags.
        assert_eq!(extract_cmd_id(&0xFFFFu16.to_le_bytes()).unwrap(), 0xFFFF);
        assert_eq!(
            extract_state_flags(&0x0010u16.to_le_bytes()).unwrap(),
            0x0010
        );
    }

    #[test]
    fn test_extract_u32_fields_little_endian() {
        assert_eq!(extract_cb_data(&4u32.to_le_bytes()).unwrap(), 4);
        assert_eq!(
            extract_error_code(&0xDEAD_BEEFu32.to_le_bytes()).unwrap(),
            0xDEAD_BEEF
        );
        assert_eq!(extract_invoke_id(&7u32.to_le_bytes()).unwrap(), 7);
    }

    #[test]
    fn test_extract_u32_fields_too_short() {
        assert!(matches!(
            extract_cb_data(&[1, 2, 3]),
            Err(AmsParseError::HeaderTooShort {
                expected: 4,
                actual: 3
            })
        ));
        assert!(matches!(
            extract_error_code(&[]),
            Err(AmsParseError::HeaderTooShort {
                expected: 4,
                actual: 0
            })
        ));
        assert!(matches!(
            extract_invoke_id(&[0; 2]),
            Err(AmsParseError::HeaderTooShort {
                expected: 4,
                actual: 2
            })
        ));
    }
}
