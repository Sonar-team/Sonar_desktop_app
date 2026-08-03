// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::s7comm::S7CommParseError,
    parse::application::protocols::s7comm::{CotpHeader, S7Header, S7ParameterItem, TpktHeader},
};

/// Length of the fixed TPKT header (RFC 1006).
const TPKT_HEADER_LENGTH: usize = 4;

/// End offset (exclusive) of the fixed COTP part read by the parser
/// (offsets 4..=10 of the packet).
const COTP_FIXED_PART_END: usize = 11;

pub fn validate_min_size(packet_len: usize, min_size: usize) -> Result<(), S7CommParseError> {
    if packet_len < min_size {
        return Err(S7CommParseError::PacketTooShort {
            expected: min_size,
            actual: packet_len,
        });
    }

    Ok(())
}

pub fn validate_tpkt_version(version: u8) -> Result<(), S7CommParseError> {
    if version != 0x03 {
        return Err(S7CommParseError::InvalidTpktVersion { version });
    }

    Ok(())
}

pub fn validate_cotp_header_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::InvalidCotpHeaderLength { expected, actual });
    }

    Ok(())
}

pub fn validate_s7_header_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::S7HeaderTooShort { expected, actual });
    }

    Ok(())
}

pub fn validate_parameter_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::InvalidParameterLength { expected, actual });
    }

    Ok(())
}

pub fn validate_data_length(expected: usize, actual: usize) -> Result<(), S7CommParseError> {
    if expected > actual {
        return Err(S7CommParseError::InvalidDataLength { expected, actual });
    }

    Ok(())
}

pub fn validate_parameter_data_not_empty(data: &[u8]) -> Result<(), S7CommParseError> {
    if data.is_empty() {
        return Err(S7CommParseError::EmptyParameterData);
    }

    Ok(())
}

pub fn validate_parameter_item_header(
    offset: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    if offset + 2 > data_len {
        return Err(S7CommParseError::InvalidParameterItemHeader);
    }

    Ok(())
}

pub fn validate_parameter_item_length(
    offset: usize,
    length: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    if offset + 2 + length > data_len {
        return Err(S7CommParseError::InvalidParameterItemLength);
    }

    Ok(())
}

pub fn validate_s7any_length(offset: usize, data_len: usize) -> Result<(), S7CommParseError> {
    if offset + 12 > data_len {
        return Err(S7CommParseError::S7AnyParameterTooShort);
    }

    Ok(())
}

/// Checks the TPKT header bytes (offsets 0..4) and returns the typed header.
///
/// Verifies the TPKT version (must be 0x03) via [`validate_tpkt_version`].
/// The length guard only protects direct callers: `S7CommPacket::parse` has
/// already validated the packet minimum size, so it never fires from there.
pub fn extract_tpkt_header(packet: &[u8]) -> Result<TpktHeader, S7CommParseError> {
    validate_min_size(packet.len(), TPKT_HEADER_LENGTH)?;
    validate_tpkt_version(packet[0])?;

    Ok(TpktHeader {
        version: packet[0],
        reserved: packet[1],
        length: u16::from_be_bytes([packet[2], packet[3]]),
    })
}

/// Checks the COTP header bytes (starting at offset 4) and returns the typed
/// header.
///
/// Verifies that the announced COTP length fits in the packet via
/// [`validate_cotp_header_length`], then reads the fields the PDU actually
/// carries:
///
/// - DT TPDU (`0xF0`, length 2) : length, type, TPDU number/EOT — no
///   destination/source references, the EOT bit lives at offset 6;
/// - connection PDUs (CR/CC/DR/DC) with length >= 6 : references at
///   offsets 6..10;
/// - anything shorter keeps zeroed reference/flag fields.
///
/// The historical extraction read offsets 6..11 unconditionally: on a DT
/// TPDU those bytes belong to the S7 header, so the references and the EOT
/// flag came out garbage (verified against Wireshark on
/// pcaps_exemple/protocols/s7comm/s7comm_varservice_libnodavedemo.pcap).
/// The length guard only protects direct callers: `S7CommPacket::parse` has
/// already validated the packet minimum size.
pub fn extract_cotp_header(packet: &[u8]) -> Result<CotpHeader, S7CommParseError> {
    validate_min_size(packet.len(), COTP_FIXED_PART_END)?;

    let cotp_len = packet[4] as usize;
    validate_cotp_header_length(4 + cotp_len + 1, packet.len())?;

    let (destination_reference, source_reference, last_data_unit) = match packet[5] {
        // DT : longueur(1) + type(1) + numero de TPDU/EOT(1).
        0xF0 if cotp_len >= 2 => (0, 0, (packet[6] & 0x80) != 0),
        // PDU de connexion : dst_ref et src_ref suivent le type.
        _ if cotp_len >= 6 => (
            u16::from_be_bytes([packet[6], packet[7]]),
            u16::from_be_bytes([packet[8], packet[9]]),
            false,
        ),
        _ => (0, 0, false),
    };

    Ok(CotpHeader {
        length: packet[4],
        pdu_type: packet[5],
        destination_reference,
        source_reference,
        last_data_unit,
    })
}

/// Checks the S7 header bytes starting at `s7_start` and returns the typed
/// header, error class/code included when the packet carries them.
///
/// `s7_start` is the absolute offset computed from the COTP header
/// (`4 + cotp.length + 1`); the reported error values keep the absolute
/// packet offsets, like the historical inline extraction.
pub fn extract_s7_header(packet: &[u8], s7_start: usize) -> Result<S7Header, S7CommParseError> {
    validate_s7_header_length(s7_start + 10, packet.len())?;

    let rosctr = packet[s7_start + 1];

    // ACK/Error fields: behaviour kept strictly identical to the historical
    // parser — when rosctr == 0x03 but the two error bytes are absent, they
    // are silently skipped instead of raising an error (semantic follow-up).
    let (error_class, error_code) = if rosctr == 0x03 && s7_start + 11 < packet.len() {
        (Some(packet[s7_start + 10]), Some(packet[s7_start + 11]))
    } else {
        (None, None)
    };

    Ok(S7Header {
        protocol_id: packet[s7_start],
        rosctr,
        reserved: u16::from_be_bytes([packet[s7_start + 2], packet[s7_start + 3]]),
        pduref: u16::from_be_bytes([packet[s7_start + 4], packet[s7_start + 5]]),
        parameter_length: u16::from_be_bytes([packet[s7_start + 6], packet[s7_start + 7]]),
        data_length: u16::from_be_bytes([packet[s7_start + 8], packet[s7_start + 9]]),
        error_class,
        error_code,
    })
}

/// Checks the parameter item bytes at `offset` inside the parameter section
/// and returns the typed item.
///
/// Verifies the 2-byte item header and the announced item length via
/// [`validate_parameter_item_header`] and [`validate_parameter_item_length`],
/// then decodes the S7ANY addressing fields (guarded by
/// [`validate_s7any_length`]) when the item is an S7ANY variable
/// specification (`spec_type == 0x12`, `syntax_id == 0x10`). Items using
/// another syntax id (e.g. 0xB2, symbolic S7-1200 addressing) keep zeroed
/// addressing fields and their raw bytes.
///
/// S7ANY layout, relative to the item start (verified against Wireshark on
/// pcaps_exemple/protocols/s7comm/s7comm_varservice_libnodavedemo.pcap,
/// frame 11):
/// `+0` spec type, `+1` length, `+2` syntax id, `+3` transport size,
/// `+4..+5` count, `+6..+7` DB number, `+8` area, `+9..+11` address.
pub fn extract_parameter_item(
    data: &[u8],
    offset: usize,
) -> Result<S7ParameterItem<'_>, S7CommParseError> {
    validate_parameter_item_header(offset, data.len())?;

    let spec_type = data[offset];
    let length = data[offset + 1] as usize;

    validate_parameter_item_length(offset, length, data.len())?;

    // `validate_parameter_item_length` a prouvé offset + 2 + length <= len et
    // length >= 0x0A implique que l'octet syntax id (+2) est lisible.
    if spec_type == 0x12 && length >= 0x0A && data[offset + 2] == 0x10 {
        validate_s7any_length(offset, data.len())?;

        let syntax_id = data[offset + 2];
        let transport_size = data[offset + 3];
        let count = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
        let db_number = u16::from_be_bytes([data[offset + 6], data[offset + 7]]);
        let area = data[offset + 8];
        let address = ((data[offset + 9] as u32) << 16)
            | ((data[offset + 10] as u32) << 8)
            | (data[offset + 11] as u32);

        Ok(S7ParameterItem {
            spec_type,
            length: length as u8,
            syntax_id,
            transport_size,
            count,
            db_number,
            area,
            address,
            raw: Some(&data[offset..offset + 2 + length]),
        })
    } else {
        Ok(S7ParameterItem {
            spec_type,
            length: length as u8,
            syntax_id: 0,
            transport_size: 0,
            count: 0,
            db_number: 0,
            area: 0,
            address: 0,
            raw: Some(&data[offset..offset + 2 + length]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_min_size() {
        assert!(validate_min_size(17, 17).is_ok());
        assert!(validate_min_size(18, 17).is_ok());
        assert_eq!(
            validate_min_size(16, 17).unwrap_err(),
            S7CommParseError::PacketTooShort {
                expected: 17,
                actual: 16,
            }
        );
    }

    #[test]
    fn test_validate_tpkt_version() {
        assert!(validate_tpkt_version(0x03).is_ok());
        assert_eq!(
            validate_tpkt_version(0x02).unwrap_err(),
            S7CommParseError::InvalidTpktVersion { version: 0x02 }
        );
    }

    #[test]
    fn test_validate_cotp_header_length() {
        assert!(validate_cotp_header_length(7, 7).is_ok());
        assert!(validate_cotp_header_length(7, 20).is_ok());
        assert_eq!(
            validate_cotp_header_length(8, 7).unwrap_err(),
            S7CommParseError::InvalidCotpHeaderLength {
                expected: 8,
                actual: 7,
            }
        );
    }

    #[test]
    fn test_validate_s7_header_length() {
        assert!(validate_s7_header_length(17, 17).is_ok());
        assert_eq!(
            validate_s7_header_length(17, 16).unwrap_err(),
            S7CommParseError::S7HeaderTooShort {
                expected: 17,
                actual: 16,
            }
        );
    }

    #[test]
    fn test_validate_parameter_length() {
        assert!(validate_parameter_length(31, 31).is_ok());
        assert_eq!(
            validate_parameter_length(31, 25).unwrap_err(),
            S7CommParseError::InvalidParameterLength {
                expected: 31,
                actual: 25,
            }
        );
    }

    #[test]
    fn test_validate_data_length() {
        assert!(validate_data_length(22, 22).is_ok());
        assert_eq!(
            validate_data_length(22, 18).unwrap_err(),
            S7CommParseError::InvalidDataLength {
                expected: 22,
                actual: 18,
            }
        );
    }

    #[test]
    fn test_validate_parameter_data_not_empty() {
        assert!(validate_parameter_data_not_empty(&[0x04]).is_ok());
        assert_eq!(
            validate_parameter_data_not_empty(&[]).unwrap_err(),
            S7CommParseError::EmptyParameterData
        );
    }

    #[test]
    fn test_validate_parameter_item_header() {
        // offset + 2 bytes of item header must fit in the parameter data
        assert!(validate_parameter_item_header(2, 4).is_ok());
        assert_eq!(
            validate_parameter_item_header(2, 3).unwrap_err(),
            S7CommParseError::InvalidParameterItemHeader
        );
    }

    #[test]
    fn test_validate_parameter_item_length() {
        // item at offset 2 with declared length 10 needs 14 bytes of data
        assert!(validate_parameter_item_length(2, 10, 14).is_ok());
        assert_eq!(
            validate_parameter_item_length(2, 10, 13).unwrap_err(),
            S7CommParseError::InvalidParameterItemLength
        );
    }

    #[test]
    fn test_validate_s7any_length() {
        assert!(validate_s7any_length(2, 14).is_ok());
        assert_eq!(
            validate_s7any_length(2, 13).unwrap_err(),
            S7CommParseError::S7AnyParameterTooShort
        );
    }

    /// Read Var request fixture already used by the parser tests
    /// (same bytes as `test_s7comm_try_from` in the s7comm parser).
    const READ_VAR_REQUEST: &str = "0300001f02f080320100000013000e00000401120a10020001000083000000";

    #[test]
    fn test_extract_tpkt_header_valid() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        let tpkt = extract_tpkt_header(&bytes).expect("valid TPKT header");
        assert_eq!(tpkt.version, 0x03);
        assert_eq!(tpkt.reserved, 0x00);
        assert_eq!(tpkt.length, 0x001f);
    }

    #[test]
    fn test_extract_tpkt_header_invalid_version() {
        let mut bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        bytes[0] = 0x02;
        assert_eq!(
            extract_tpkt_header(&bytes).unwrap_err(),
            S7CommParseError::InvalidTpktVersion { version: 0x02 }
        );
    }

    #[test]
    fn test_extract_tpkt_header_too_short() {
        assert_eq!(
            extract_tpkt_header(&[0x03, 0x00]).unwrap_err(),
            S7CommParseError::PacketTooShort {
                expected: 4,
                actual: 2,
            }
        );
    }

    #[test]
    fn test_extract_cotp_header_valid() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        let cotp = extract_cotp_header(&bytes).expect("valid COTP header");
        assert_eq!(cotp.length, 0x02);
        assert_eq!(cotp.pdu_type, 0xf0);
        // Un DT TPDU (3 octets) ne porte pas de references : les anciens
        // offsets fixes 6..11 lisaient des octets de l'en-tete S7
        // (0x8032/0x0100, soit protocol_id+rosctr et reserved).
        assert_eq!(cotp.destination_reference, 0);
        assert_eq!(cotp.source_reference, 0);
        // Le bit EOT vit a l'offset 6 (0x80) : cette trame est bien la
        // derniere unite de donnees.
        assert!(cotp.last_data_unit);
    }

    #[test]
    fn test_extract_cotp_header_connection_request_references() {
        // CR TPDU : longueur 6, references aux offsets 6..10.
        let bytes: &[u8] = &[
            0x03, 0x00, 0x00, 0x0b, // TPKT
            0x06, 0xe0, 0x12, 0x34, 0x56, 0x78, 0x00, // COTP CR
        ];
        let cotp = extract_cotp_header(bytes).expect("valid COTP header");
        assert_eq!(cotp.pdu_type, 0xe0);
        assert_eq!(cotp.destination_reference, 0x1234);
        assert_eq!(cotp.source_reference, 0x5678);
        assert!(!cotp.last_data_unit);
    }

    #[test]
    fn test_extract_cotp_header_announced_length_beyond_packet() {
        let mut bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        bytes[4] = 0xff;
        assert_eq!(
            extract_cotp_header(&bytes).unwrap_err(),
            S7CommParseError::InvalidCotpHeaderLength {
                expected: 260,
                actual: 31,
            }
        );
    }

    #[test]
    fn test_extract_s7_header_valid() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        let s7 = extract_s7_header(&bytes, 7).expect("valid S7 header");
        assert_eq!(s7.protocol_id, 0x32);
        assert_eq!(s7.rosctr, 0x01);
        assert_eq!(s7.reserved, 0x0000);
        assert_eq!(s7.pduref, 0x0013);
        assert_eq!(s7.parameter_length, 14);
        assert_eq!(s7.data_length, 0);
        assert_eq!(s7.error_class, None);
        assert_eq!(s7.error_code, None);
    }

    #[test]
    fn test_extract_s7_header_ack_data_error_fields() {
        // Ack-Data fixture already used by the parser tests
        // (same bytes as `test_parameter_request_download`).
        let bytes = hex::decode("0300001402f080320300000e000001000000001a").expect("valid hex");
        let s7 = extract_s7_header(&bytes, 7).expect("valid S7 header");
        assert_eq!(s7.rosctr, 0x03);
        assert_eq!(s7.error_class, Some(0x00));
        assert_eq!(s7.error_code, Some(0x00));
    }

    #[test]
    fn test_extract_s7_header_too_short() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        assert_eq!(
            extract_s7_header(&bytes[..12], 7).unwrap_err(),
            S7CommParseError::S7HeaderTooShort {
                expected: 17,
                actual: 12,
            }
        );
    }

    #[test]
    fn test_extract_parameter_item_s7any() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        // Parameter section of the fixture: offsets 17..31
        // (function 0x04, item count 0x01, then one S7ANY item).
        let data = &bytes[17..31];
        let item = extract_parameter_item(data, 2).expect("valid parameter item");
        assert_eq!(item.spec_type, 0x12);
        assert_eq!(item.length, 0x0a);
        assert_eq!(item.syntax_id, 0x10);
        assert_eq!(item.transport_size, 0x02);
        // Layout S7ANY reel : count(+4..+5), db(+6..+7), area(+8),
        // address(+9..+11) — l'ancien decodage enjambait le count de travers
        // et sortait db=0x0100, area=0x00, address=0x830000.
        assert_eq!(item.count, 1);
        assert_eq!(item.db_number, 0);
        assert_eq!(item.area, 0x83);
        assert_eq!(item.address, 0);
        assert_eq!(item.raw, Some(&data[2..14]));
    }

    #[test]
    fn test_extract_parameter_item_non_s7any_syntax_keeps_raw_only() {
        // Item 1200SYM (syntax id 0xb2) : les champs d'adressage S7ANY ne
        // s'appliquent pas, seul le raw est conserve.
        let data: &[u8] = &[
            0x12, 0x0e, 0xb2, 0xff, 0x00, 0x00, 0x00, 0x52, 0xea, 0x2d, 0xb0, 0xd9, 0x40, 0x00,
            0x00, 0x10,
        ];
        let item = extract_parameter_item(data, 0).expect("valid parameter item");
        assert_eq!(item.spec_type, 0x12);
        assert_eq!(item.length, 0x0e);
        assert_eq!(item.syntax_id, 0);
        assert_eq!(item.count, 0);
        assert_eq!(item.db_number, 0);
        assert_eq!(item.area, 0);
        assert_eq!(item.address, 0);
        assert_eq!(item.raw, Some(data));
    }

    #[test]
    fn test_extract_parameter_item_truncated_header() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        // Only 1 byte left after the item offset: 2-byte header cannot fit.
        let data = &bytes[17..20];
        assert_eq!(
            extract_parameter_item(data, 2).unwrap_err(),
            S7CommParseError::InvalidParameterItemHeader
        );
    }

    #[test]
    fn test_extract_parameter_item_truncated_length() {
        let bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        // Item declares length 0x0a but only 2 bytes follow its header.
        let data = &bytes[17..23];
        assert_eq!(
            extract_parameter_item(data, 2).unwrap_err(),
            S7CommParseError::InvalidParameterItemLength
        );
    }
}
