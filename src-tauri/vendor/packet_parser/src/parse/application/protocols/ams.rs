// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::convert::TryFrom;

use crate::{
    checks::application::ams::{
        AMS_HEADER_LEN, extract_cb_data, extract_cmd_id, extract_error_code, extract_invoke_id,
        extract_sender_net_id, extract_sender_port, extract_state_flags, extract_target_net_id,
        extract_target_port, validate_ams_header_length, validate_cb_data_length, validate_cmd_id,
        validate_state_flags,
    },
    errors::application::ams::AmsParseError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// AMS Packet
///
/// ```mermaid
/// ---
/// title: AmsPacket
/// ---
/// packet-beta
/// 0-47: "Target Net ID u48"
/// 48-63: "Target Port u16"
/// 64-111: "Sender Net ID u48"
/// 112-127: "Sender Port u16"
/// 128-143: "Command ID u16"
/// 144-159: "State Flags u16"
/// 160-191: "Data Length u32"
/// 192-223: "Error Code u32"
/// 224-255: "Invoke ID u32"
/// 256-319: "Data variable"
/// ```
///
/// Représente un paquet AMS (header + payload)
#[derive(Debug)]
pub struct AmsPacket<'a> {
    pub ams_target_net_id: [u8; 6], // ex: [0,0,0,0,0,130] => "0.0.0.0.0.130"
    pub ams_target_port: u16,

    pub ams_sender_net_id: [u8; 6],
    pub ams_sender_port: u16,

    pub cmd_id: u16,
    pub state_flags: u16,

    /// Longueur déclarée des données (cbData dans la spec AMS)
    pub cb_data: u32,

    pub error_code: u32,
    pub invoke_id: u32,

    /// Slice sur les données applicatives
    pub data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for AmsPacket<'a> {
    type Error = AmsParseError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let len = bytes.len();

        // 1) Longueur minimale
        validate_ams_header_length(len)?;

        // Layout AMS (32 octets, little-endian) :
        //  0..=5   TargetNetId (6 octets)
        //  6..=7   TargetPort (u16)
        //  8..=13  SourceNetId (6 octets)
        // 14..=15  SourcePort (u16)
        // 16..=17  CmdId (u16)
        // 18..=19  StateFlags (u16)
        // 20..=23  Length / cbData (u32)
        // 24..=27  ErrorCode (u32)
        // 28..=31  InvokeId (u32)
        // 32..     Data

        // 2) Extraction champ par champ, dans l'ordre du wire : chaque
        //    extract_* vérifie les octets de SON champ et retourne la valeur
        //    typée (jamais en erreur ici, le pré-check garantit 32 octets).
        let ams_target_net_id = extract_target_net_id(&bytes[0..6])?;
        let ams_target_port = extract_target_port(&bytes[6..8])?;

        let ams_sender_net_id = extract_sender_net_id(&bytes[8..14])?;
        let ams_sender_port = extract_sender_port(&bytes[14..16])?;

        let cmd_id = extract_cmd_id(&bytes[16..18])?;
        let state_flags = extract_state_flags(&bytes[18..20])?;

        let cb_data = extract_cb_data(&bytes[20..24])?;
        let error_code = extract_error_code(&bytes[24..28])?;
        let invoke_id = extract_invoke_id(&bytes[28..32])?;

        let data_start = AMS_HEADER_LEN;
        let actual_data_len = len - data_start;

        // Validations post-extraction. NOTE : leur ordre (cb_data → cmd_id →
        // state_flags) ne suit pas l'ordre du wire (cmd_id offset 16,
        // state_flags offset 18, cb_data offset 20) et est volontairement
        // conservé : fusionner validate_cmd_id / validate_state_flags dans
        // leurs extract_* respectifs les exécuterait avant
        // validate_cb_data_length, et un paquet où cb_data ET cmd_id sont
        // tous deux invalides retournerait UnknownCommand au lieu
        // d'InvalidCbDataLength. Le comportement observable (mêmes erreurs
        // pour les mêmes inputs) doit rester identique.

        // 3) Validation cb_data : la longueur déclarée doit coller à la réalité
        validate_cb_data_length(cb_data, actual_data_len)?;

        // 4) Validation cmd_id : doit faire partie des commandes connues
        validate_cmd_id(cmd_id)?;

        // 5) Validation des state_flags : pas de bits réservés
        validate_state_flags(state_flags)?;

        let data = &bytes[data_start..data_start + actual_data_len];

        Ok(AmsPacket {
            ams_target_net_id,
            ams_target_port,
            ams_sender_net_id,
            ams_sender_port,
            cmd_id,
            state_flags,
            cb_data,
            error_code,
            invoke_id,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Header AMS valide : cmd_id 1, state_flags 0x04, cb_data = data.len()
    fn build_ams(cmd_id: u16, state_flags: u16, cb_data: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + data.len());
        bytes.extend_from_slice(&[0, 0, 0, 0, 0, 130]); // target net id
        bytes.extend_from_slice(&851u16.to_le_bytes()); // target port
        bytes.extend_from_slice(&[192, 168, 1, 1, 1, 1]); // sender net id
        bytes.extend_from_slice(&33000u16.to_le_bytes()); // sender port
        bytes.extend_from_slice(&cmd_id.to_le_bytes());
        bytes.extend_from_slice(&state_flags.to_le_bytes());
        bytes.extend_from_slice(&cb_data.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes()); // error code
        bytes.extend_from_slice(&7u32.to_le_bytes()); // invoke id
        bytes.extend_from_slice(data);
        bytes
    }

    #[test]
    fn test_parse_valid_ams_packet() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        let bytes = build_ams(0x0002, 0x0004, 4, &data);

        let ams = AmsPacket::try_from(bytes.as_slice()).expect("paquet AMS valide");
        assert_eq!(ams.ams_target_net_id, [0, 0, 0, 0, 0, 130]);
        assert_eq!(ams.ams_target_port, 851);
        assert_eq!(ams.ams_sender_net_id, [192, 168, 1, 1, 1, 1]);
        assert_eq!(ams.ams_sender_port, 33000);
        assert_eq!(ams.cmd_id, 2);
        assert_eq!(ams.state_flags, 4);
        assert_eq!(ams.cb_data, 4);
        assert_eq!(ams.error_code, 0);
        assert_eq!(ams.invoke_id, 7);
        assert_eq!(ams.data, &data);
    }

    #[test]
    fn test_parse_valid_ams_packet_without_data() {
        let bytes = build_ams(0x0001, 0x0000, 0, &[]);
        let ams = AmsPacket::try_from(bytes.as_slice()).expect("paquet AMS sans data");
        assert!(ams.data.is_empty());
    }

    #[test]
    fn test_header_too_short() {
        let bytes = [0u8; 31];
        assert!(matches!(
            AmsPacket::try_from(&bytes[..]),
            Err(AmsParseError::HeaderTooShort {
                expected: 32,
                actual: 31
            })
        ));
    }

    #[test]
    fn test_cb_data_mismatch() {
        // cb_data annonce 10 octets mais 4 fournis
        let bytes = build_ams(0x0001, 0x0004, 10, &[1, 2, 3, 4]);
        assert!(matches!(
            AmsPacket::try_from(bytes.as_slice()),
            Err(AmsParseError::InvalidCbDataLength {
                cb_data: 10,
                actual: 4
            })
        ));
    }

    #[test]
    fn test_unknown_command() {
        let bytes = build_ams(0x0000, 0x0004, 0, &[]);
        assert!(matches!(
            AmsPacket::try_from(bytes.as_slice()),
            Err(AmsParseError::UnknownCommand(0))
        ));

        let bytes = build_ams(0x000A, 0x0004, 0, &[]);
        assert!(matches!(
            AmsPacket::try_from(bytes.as_slice()),
            Err(AmsParseError::UnknownCommand(0x000A))
        ));
    }

    #[test]
    fn test_invalid_state_flags() {
        // bit réservé (0x0010) positionné
        let bytes = build_ams(0x0001, 0x0010, 0, &[]);
        assert!(matches!(
            AmsPacket::try_from(bytes.as_slice()),
            Err(AmsParseError::InvalidStateFlags(0x0010))
        ));
    }
}
