use std::convert::TryFrom;
use thiserror::Error;

/// Taille fixe de l'entête AMS (en octets)
pub const AMS_HEADER_LEN: usize = 32;

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

/// Commandes AMS/ADS connues.
/// (Tu peux compléter si besoin)
fn is_known_cmd_id(cmd_id: u16) -> bool {
    matches!(cmd_id, 0x0001..=0x0009)
}

/// Bits réservés dans state_flags.
/// Ici on suppose que seuls les 4 bits de poids faible sont utilisés.
/// (Tu peux adapter le masque selon la spec que tu suis.)
fn has_reserved_state_bits(flags: u16) -> bool {
    let reserved_mask: u16 = !0x000F; // tout sauf les 4 bits LSB
    flags & reserved_mask != 0
}

#[derive(Debug, Error)]
pub enum AmsParseError {
    #[error("AMS header too short: expected at least {expected} bytes, got {actual}")]
    HeaderTooShort { expected: usize, actual: usize },

    #[error("AMS payload length ({cb_data}) does not match actual data length ({actual})")]
    InvalidCbDataLength { cb_data: u32, actual: usize },

    #[error("Unknown AMS command id: 0x{0:04x}")]
    UnknownCommand(u16),

    #[error("Invalid AMS state flags: reserved bits set (0x{0:04x})")]
    InvalidStateFlags(u16),
}

impl<'a> TryFrom<&'a [u8]> for AmsPacket<'a> {
    type Error = AmsParseError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let len = bytes.len();

        // 1) Longueur minimale
        if len < AMS_HEADER_LEN {
            return Err(AmsParseError::HeaderTooShort {
                expected: AMS_HEADER_LEN,
                actual: len,
            });
        }

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

        let ams_target_net_id: [u8; 6] = bytes[0..6].try_into().unwrap();
        let ams_target_port = u16::from_le_bytes(bytes[6..8].try_into().unwrap());

        let ams_sender_net_id: [u8; 6] = bytes[8..14].try_into().unwrap();
        let ams_sender_port = u16::from_le_bytes(bytes[14..16].try_into().unwrap());

        let cmd_id = u16::from_le_bytes(bytes[16..18].try_into().unwrap());
        let state_flags = u16::from_le_bytes(bytes[18..20].try_into().unwrap());

        let cb_data = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
        let error_code = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let invoke_id = u32::from_le_bytes(bytes[28..32].try_into().unwrap());

        let data_start = AMS_HEADER_LEN;
        let actual_data_len = len - data_start;

        // 2) Validation cb_data : la longueur déclarée doit coller à la réalité
        if actual_data_len != cb_data as usize {
            return Err(AmsParseError::InvalidCbDataLength {
                cb_data,
                actual: actual_data_len,
            });
        }

        // 3) Validation cmd_id : doit faire partie des commandes connues
        if !is_known_cmd_id(cmd_id) {
            return Err(AmsParseError::UnknownCommand(cmd_id));
        }

        // 4) Validation des state_flags : pas de bits réservés
        if has_reserved_state_bits(state_flags) {
            return Err(AmsParseError::InvalidStateFlags(state_flags));
        }

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
