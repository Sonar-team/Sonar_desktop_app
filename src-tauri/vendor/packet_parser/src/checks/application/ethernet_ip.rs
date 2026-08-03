// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::ethernet_ip::EtherNetIpError,
    parse::application::protocols::ethernet_ip::EtherNetIpCommand,
};

pub const ENCAPSULATION_HEADER_LEN: usize = 24;
pub const REGISTER_SESSION_DATA_LEN: usize = 4;
pub const COMMON_PACKET_FORMAT_MIN_LEN: usize = 8;
/// Un item CPF pèse au minimum 4 octets : type_id 2 + length 2.
pub const CPF_ITEM_HEADER_LEN: usize = 4;

pub fn validate_min_length(packet: &[u8]) -> Result<(), EtherNetIpError> {
    if packet.len() < ENCAPSULATION_HEADER_LEN {
        return Err(EtherNetIpError::PacketTooShort {
            expected: ENCAPSULATION_HEADER_LEN,
            actual: packet.len(),
        });
    }

    Ok(())
}

pub fn validate_declared_length(
    declared_length: u16,
    actual_len: usize,
) -> Result<(), EtherNetIpError> {
    let expected = ENCAPSULATION_HEADER_LEN + declared_length as usize;
    if expected != actual_len {
        return Err(EtherNetIpError::LengthMismatch {
            expected,
            actual: actual_len,
        });
    }

    Ok(())
}

pub fn validate_options(options: u32) -> Result<(), EtherNetIpError> {
    if options != 0 {
        return Err(EtherNetIpError::InvalidOptions { options });
    }

    Ok(())
}

pub fn validate_empty_command_data(
    command: EtherNetIpCommand,
    data: &[u8],
) -> Result<(), EtherNetIpError> {
    if !data.is_empty() {
        return Err(EtherNetIpError::UnexpectedCommandData {
            command: command.name(),
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn validate_register_session_length(data: &[u8]) -> Result<(), EtherNetIpError> {
    if data.len() != REGISTER_SESSION_DATA_LEN {
        return Err(EtherNetIpError::InvalidRegisterSessionLength {
            expected: REGISTER_SESSION_DATA_LEN,
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn validate_protocol_version(version: u16) -> Result<(), EtherNetIpError> {
    if version != 1 {
        return Err(EtherNetIpError::UnsupportedProtocolVersion { version });
    }

    Ok(())
}

pub fn validate_register_options_flags(flags: u16) -> Result<(), EtherNetIpError> {
    if flags != 0 {
        return Err(EtherNetIpError::InvalidRegisterSessionOptions { flags });
    }

    Ok(())
}

pub fn validate_common_packet_format_min_length(data: &[u8]) -> Result<(), EtherNetIpError> {
    if data.len() < COMMON_PACKET_FORMAT_MIN_LEN {
        return Err(EtherNetIpError::Truncated {
            field: "common_packet_format",
            needed: COMMON_PACKET_FORMAT_MIN_LEN,
            actual: data.len(),
        });
    }

    Ok(())
}

pub fn ensure_available(
    field: &'static str,
    actual: usize,
    needed: usize,
) -> Result<(), EtherNetIpError> {
    if actual < needed {
        return Err(EtherNetIpError::Truncated {
            field,
            needed,
            actual,
        });
    }

    Ok(())
}

pub fn validate_interface_handle(interface_handle: u32) -> Result<(), EtherNetIpError> {
    if interface_handle != 0 {
        return Err(EtherNetIpError::InvalidInterfaceHandle { interface_handle });
    }

    Ok(())
}

pub fn validate_cpf_consumed(consumed: usize, actual: usize) -> Result<(), EtherNetIpError> {
    if consumed != actual {
        return Err(EtherNetIpError::TrailingCpfData { consumed, actual });
    }

    Ok(())
}

/// Vérifie les octets 0..2 du paquet (u16 little-endian) et retourne le code
/// de commande typé, ou `UnknownCommand` si le code n'est pas reconnu.
pub fn extract_command(packet: &[u8]) -> Result<EtherNetIpCommand, EtherNetIpError> {
    ensure_available("command", packet.len(), 2)?;

    let command = u16::from_le_bytes([packet[0], packet[1]]);
    EtherNetIpCommand::try_from(command)
}

/// Vérifie les octets 2..4 (u16 little-endian) et retourne la longueur
/// annoncée après avoir contrôlé qu'elle correspond exactement à
/// `actual_len` (taille totale du paquet, en-tête compris).
pub fn extract_length(packet: &[u8], actual_len: usize) -> Result<u16, EtherNetIpError> {
    ensure_available("length", packet.len(), 4)?;

    let length = u16::from_le_bytes([packet[2], packet[3]]);
    validate_declared_length(length, actual_len)?;
    Ok(length)
}

/// Vérifie que les octets 4..8 sont présents et retourne le session handle
/// (u32 little-endian). Champ sans contrainte sémantique.
pub fn extract_session_handle(packet: &[u8]) -> Result<u32, EtherNetIpError> {
    ensure_available("session_handle", packet.len(), 8)?;

    Ok(u32::from_le_bytes([
        packet[4], packet[5], packet[6], packet[7],
    ]))
}

/// Vérifie que les octets 8..12 sont présents et retourne le statut
/// (u32 little-endian). Champ sans contrainte sémantique.
pub fn extract_status(packet: &[u8]) -> Result<u32, EtherNetIpError> {
    ensure_available("status", packet.len(), 12)?;

    Ok(u32::from_le_bytes([
        packet[8], packet[9], packet[10], packet[11],
    ]))
}

/// Vérifie que les octets 12..20 sont présents et retourne le sender context
/// emprunté au paquet (8 octets opaques, sans contrainte sémantique).
pub fn extract_sender_context(packet: &[u8]) -> Result<&[u8], EtherNetIpError> {
    ensure_available("sender_context", packet.len(), 20)?;

    Ok(&packet[12..20])
}

/// Vérifie les octets 20..24 (u32 little-endian) et retourne le champ
/// options, qui doit valoir 0 d'après la spécification d'encapsulation.
pub fn extract_options(packet: &[u8]) -> Result<u32, EtherNetIpError> {
    ensure_available("options", packet.len(), 24)?;

    let options = u32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]);
    validate_options(options)?;
    Ok(options)
}

/// Vérifie les octets 0..2 du command data RegisterSession (u16
/// little-endian) et retourne la version de protocole, qui doit valoir 1.
pub fn extract_protocol_version(data: &[u8]) -> Result<u16, EtherNetIpError> {
    ensure_available("protocol_version", data.len(), 2)?;

    let version = u16::from_le_bytes([data[0], data[1]]);
    validate_protocol_version(version)?;
    Ok(version)
}

/// Vérifie les octets 2..4 du command data RegisterSession (u16
/// little-endian) et retourne les options flags, qui doivent valoir 0.
pub fn extract_register_options_flags(data: &[u8]) -> Result<u16, EtherNetIpError> {
    ensure_available("register_options_flags", data.len(), 4)?;

    let flags = u16::from_le_bytes([data[2], data[3]]);
    validate_register_options_flags(flags)?;
    Ok(flags)
}

/// Vérifie les octets 0..4 du Common Packet Format (u32 little-endian) et
/// retourne l'interface handle, qui doit valoir 0 (interface CIP).
pub fn extract_interface_handle(data: &[u8]) -> Result<u32, EtherNetIpError> {
    ensure_available("interface_handle", data.len(), 4)?;

    let interface_handle = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    validate_interface_handle(interface_handle)?;
    Ok(interface_handle)
}

/// Vérifie que les octets 4..6 du Common Packet Format sont présents et
/// retourne le timeout (u16 little-endian). Champ sans contrainte sémantique.
pub fn extract_timeout(data: &[u8]) -> Result<u16, EtherNetIpError> {
    ensure_available("timeout", data.len(), 6)?;

    Ok(u16::from_le_bytes([data[4], data[5]]))
}

/// Vérifie que les octets 6..8 du Common Packet Format sont présents et
/// retourne le nombre d'items annoncé (u16 little-endian). La cohérence du
/// compte avec les octets restants est contrôlée item par item.
pub fn extract_item_count(data: &[u8]) -> Result<u16, EtherNetIpError> {
    ensure_available("item_count", data.len(), 8)?;

    Ok(u16::from_le_bytes([data[6], data[7]]))
}

/// Vérifie et extrait un item CPF (en-tête type_id + length, puis données) à
/// partir de `offset`, et retourne `(type_id, données, offset de fin)`.
pub fn extract_cpf_item(
    data: &[u8],
    offset: usize,
) -> Result<(u16, &[u8], usize), EtherNetIpError> {
    // Garde anti-débordement : le parseur maintient toujours
    // `offset <= data.len()`, mais cette fonction est publique. Un appel hors
    // contrat avec un offset géant ne doit pas faire déborder `offset + 4`.
    if offset > data.len() {
        return Err(EtherNetIpError::Truncated {
            field: "cpf_item_header",
            needed: offset.saturating_add(CPF_ITEM_HEADER_LEN),
            actual: data.len(),
        });
    }

    // Invariant prouvé ci-dessus : offset <= data.len() <= isize::MAX, donc
    // les additions suivantes (au plus + 4 + 65 535) ne peuvent pas déborder.
    let item_header_end = offset + CPF_ITEM_HEADER_LEN;
    ensure_available("cpf_item_header", data.len(), item_header_end)?;

    let type_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
    let length = u16::from_le_bytes([data[offset + 2], data[offset + 3]]) as usize;

    let item_end = item_header_end + length;
    ensure_available("cpf_item_data", data.len(), item_end)?;

    Ok((type_id, &data[item_header_end..item_end], item_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Paquet RegisterSession complet, repris de la fixture du parseur
    /// (`parse_register_session_request`).
    fn register_session_packet() -> Vec<u8> {
        hex::decode("65000400000000000000000001020304050607080000000001000000")
            .expect("invalid hex fixture")
    }

    /// Command data CPF (SendRRData) repris de la fixture du parseur
    /// (`parse_send_rr_data_common_packet_format`), en-tête d'encapsulation
    /// retiré : interface 0, timeout 0, 2 items.
    fn cpf_data() -> Vec<u8> {
        hex::decode("000000000000020000000000b20004004c022001").expect("invalid hex fixture")
    }

    #[test]
    fn test_extract_command() {
        let packet = register_session_packet();
        assert_eq!(
            extract_command(&packet),
            Ok(EtherNetIpCommand::RegisterSession)
        );

        assert_eq!(
            extract_command(&[0x99, 0x00]),
            Err(EtherNetIpError::UnknownCommand { command: 0x0099 })
        );

        assert_eq!(
            extract_command(&[0x65]),
            Err(EtherNetIpError::Truncated {
                field: "command",
                needed: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn test_extract_length() {
        let packet = register_session_packet();
        assert_eq!(extract_length(&packet, packet.len()), Ok(4));

        // Longueur annoncée (4) incompatible avec une taille totale de 27.
        assert_eq!(
            extract_length(&packet, 27),
            Err(EtherNetIpError::LengthMismatch {
                expected: 28,
                actual: 27
            })
        );
    }

    #[test]
    fn test_extract_header_fields_without_constraint() {
        let packet = register_session_packet();
        assert_eq!(extract_session_handle(&packet), Ok(0));
        assert_eq!(extract_status(&packet), Ok(0));
        assert_eq!(
            extract_sender_context(&packet),
            Ok(&[1u8, 2, 3, 4, 5, 6, 7, 8][..])
        );

        assert_eq!(
            extract_sender_context(&packet[..16]),
            Err(EtherNetIpError::Truncated {
                field: "sender_context",
                needed: 20,
                actual: 16
            })
        );
    }

    #[test]
    fn test_extract_options() {
        let packet = register_session_packet();
        assert_eq!(extract_options(&packet), Ok(0));

        let mut invalid = packet.clone();
        invalid[20] = 0x01;
        assert_eq!(
            extract_options(&invalid),
            Err(EtherNetIpError::InvalidOptions { options: 1 })
        );
    }

    #[test]
    fn test_extract_register_session_fields() {
        // Command data de la fixture RegisterSession : version 1, flags 0.
        let data = [0x01, 0x00, 0x00, 0x00];
        assert_eq!(extract_protocol_version(&data), Ok(1));
        assert_eq!(extract_register_options_flags(&data), Ok(0));

        let bad_version = [0x02, 0x00, 0x00, 0x00];
        assert_eq!(
            extract_protocol_version(&bad_version),
            Err(EtherNetIpError::UnsupportedProtocolVersion { version: 2 })
        );

        let bad_flags = [0x01, 0x00, 0x01, 0x00];
        assert_eq!(
            extract_register_options_flags(&bad_flags),
            Err(EtherNetIpError::InvalidRegisterSessionOptions { flags: 1 })
        );
    }

    #[test]
    fn test_extract_common_packet_format_fields() {
        let data = cpf_data();
        assert_eq!(extract_interface_handle(&data), Ok(0));
        assert_eq!(extract_timeout(&data), Ok(0));
        assert_eq!(extract_item_count(&data), Ok(2));

        let mut invalid = data.clone();
        invalid[0] = 0x01;
        assert_eq!(
            extract_interface_handle(&invalid),
            Err(EtherNetIpError::InvalidInterfaceHandle {
                interface_handle: 1
            })
        );
    }

    #[test]
    fn test_extract_cpf_item() {
        let data = cpf_data();

        let (type_id, item_data, offset) =
            extract_cpf_item(&data, 8).expect("valid null address item");
        assert_eq!(type_id, 0x0000);
        assert!(item_data.is_empty());
        assert_eq!(offset, 12);

        let (type_id, item_data, offset) =
            extract_cpf_item(&data, offset).expect("valid unconnected data item");
        assert_eq!(type_id, 0x00B2);
        assert_eq!(item_data, &[0x4C, 0x02, 0x20, 0x01]);
        assert_eq!(offset, 20);
    }

    #[test]
    fn test_extract_cpf_item_truncated() {
        let data = cpf_data();

        // En-tête d'item incomplet : il ne reste que 2 octets après l'offset.
        assert_eq!(
            extract_cpf_item(&data[..14], 12),
            Err(EtherNetIpError::Truncated {
                field: "cpf_item_header",
                needed: 16,
                actual: 14
            })
        );

        // Données d'item incomplètes : length annonce 4 octets, 2 présents.
        assert_eq!(
            extract_cpf_item(&data[..18], 12),
            Err(EtherNetIpError::Truncated {
                field: "cpf_item_data",
                needed: 20,
                actual: 18
            })
        );

        // Offset hostile au-delà de la slice : erreur typée, pas de panique.
        assert_eq!(
            extract_cpf_item(&data, usize::MAX),
            Err(EtherNetIpError::Truncated {
                field: "cpf_item_header",
                needed: usize::MAX,
                actual: data.len()
            })
        );
    }
}
