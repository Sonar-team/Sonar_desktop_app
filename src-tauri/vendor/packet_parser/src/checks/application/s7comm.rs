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

/// Smallest RFC 1006 TPKT (4-byte TPKT plus a 3-byte DT TPDU).
const TPKT_MINIMUM_LENGTH: usize = 7;

/// A COTP Data TPDU used by S7Comm is `LI`, `0xf0`, then TPDU-NR/EOT.
const COTP_DT_LENGTH_INDICATOR: u8 = 2;

/// Length of the common S7 header, before the ACK error bytes.
pub const S7_BASE_HEADER_LENGTH: usize = 10;

/// Length of an ACK or ACK-Data S7 header.
pub const S7_ACK_HEADER_LENGTH: usize = 12;

/// Adds a wire offset and a field length without allowing `usize` wraparound.
pub fn checked_offset(
    offset: usize,
    length: usize,
    context: &'static str,
) -> Result<usize, S7CommParseError> {
    offset
        .checked_add(length)
        .ok_or(S7CommParseError::LengthOverflow { context })
}

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

pub fn validate_tpkt_reserved(reserved: u8) -> Result<(), S7CommParseError> {
    if reserved != 0 {
        return Err(S7CommParseError::InvalidTpktReserved { reserved });
    }

    Ok(())
}

/// Validates the first TPKT boundary while allowing later coalesced TPKTs in
/// the same TCP payload.
pub fn validate_tpkt_length(declared: usize, available: usize) -> Result<(), S7CommParseError> {
    if declared < TPKT_MINIMUM_LENGTH {
        return Err(S7CommParseError::InvalidTpktLength {
            declared,
            minimum: TPKT_MINIMUM_LENGTH,
        });
    }
    if declared > available {
        return Err(S7CommParseError::TruncatedTpkt {
            declared,
            actual: available,
        });
    }

    Ok(())
}

pub fn validate_cotp_length_indicator(length: u8) -> Result<(), S7CommParseError> {
    if length != COTP_DT_LENGTH_INDICATOR {
        return Err(S7CommParseError::InvalidCotpLengthIndicator { length });
    }

    Ok(())
}

pub fn validate_cotp_pdu_type(pdu_type: u8) -> Result<(), S7CommParseError> {
    if pdu_type != 0xf0 {
        return Err(S7CommParseError::InvalidCotpPduType { pdu_type });
    }

    Ok(())
}

pub fn validate_cotp_eot(last_data_unit: bool) -> Result<(), S7CommParseError> {
    if !last_data_unit {
        return Err(S7CommParseError::CotpNotLastDataUnit);
    }

    Ok(())
}

pub fn validate_cotp_tpdu_number(tpdu_number: u8) -> Result<(), S7CommParseError> {
    if tpdu_number != 0 {
        return Err(S7CommParseError::InvalidCotpTpduNumber { tpdu_number });
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

pub fn extract_parameter_item_count(data: &[u8]) -> Result<u8, S7CommParseError> {
    if data.len() < 2 {
        return Err(S7CommParseError::MissingParameterItemCount);
    }

    Ok(data[1])
}

pub fn validate_s7_protocol_id(protocol_id: u8) -> Result<(), S7CommParseError> {
    if protocol_id != 0x32 {
        return Err(S7CommParseError::InvalidS7ProtocolId { protocol_id });
    }

    Ok(())
}

pub fn validate_s7_rosctr(rosctr: u8) -> Result<(), S7CommParseError> {
    if !matches!(rosctr, 0x01 | 0x02 | 0x03 | 0x07) {
        return Err(S7CommParseError::InvalidS7Rosctr { rosctr });
    }

    Ok(())
}

pub fn validate_s7_reserved(reserved: u16) -> Result<(), S7CommParseError> {
    if reserved != 0 {
        return Err(S7CommParseError::InvalidS7Reserved { reserved });
    }

    Ok(())
}

pub fn s7_header_length(rosctr: u8) -> Result<usize, S7CommParseError> {
    validate_s7_rosctr(rosctr)?;
    Ok(if matches!(rosctr, 0x02 | 0x03) {
        S7_ACK_HEADER_LENGTH
    } else {
        S7_BASE_HEADER_LENGTH
    })
}

pub fn validate_section_lengths(
    sections_end: usize,
    tpkt_end: usize,
) -> Result<(), S7CommParseError> {
    if sections_end != tpkt_end {
        return Err(S7CommParseError::InconsistentSectionLengths {
            sections_end,
            tpkt_end,
        });
    }

    Ok(())
}

pub fn validate_parameter_items_consumed(
    consumed: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    if consumed != data_len {
        return Err(S7CommParseError::UnexpectedParameterBytes {
            remaining: data_len.saturating_sub(consumed),
        });
    }

    Ok(())
}

pub fn validate_parameter_item_padding(
    offset: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    let end = checked_offset(offset, 1, "parameter item padding")?;
    if end > data_len {
        return Err(S7CommParseError::MissingParameterItemPadding);
    }

    Ok(())
}

pub fn validate_parameter_item_header(
    offset: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    let end = checked_offset(offset, 2, "parameter item header")?;
    if end > data_len {
        return Err(S7CommParseError::InvalidParameterItemHeader);
    }

    Ok(())
}

pub fn validate_parameter_item_length(
    offset: usize,
    length: usize,
    data_len: usize,
) -> Result<(), S7CommParseError> {
    let item_data = checked_offset(offset, 2, "parameter item header")?;
    let end = checked_offset(item_data, length, "parameter item")?;
    if end > data_len {
        return Err(S7CommParseError::InvalidParameterItemLength);
    }

    Ok(())
}

pub fn validate_parameter_specification_type(spec_type: u8) -> Result<(), S7CommParseError> {
    if spec_type != 0x12 {
        return Err(S7CommParseError::InvalidParameterSpecificationType { spec_type });
    }

    Ok(())
}

pub fn validate_parameter_syntax_id_present(length: u8) -> Result<(), S7CommParseError> {
    if length == 0 {
        return Err(S7CommParseError::MissingParameterSyntaxId);
    }

    Ok(())
}

pub fn validate_s7any_length(length: u8) -> Result<(), S7CommParseError> {
    if length != 10 {
        return Err(S7CommParseError::InvalidS7AnyLength { length });
    }

    Ok(())
}

/// Checks the TPKT header bytes (offsets 0..4) and returns the typed header.
///
/// Verifies version 3, the zero reserved byte, and that the first announced
/// TPKT is complete. RFC 2126 permits receivers to ignore a non-zero reserved
/// byte; this classifier deliberately requires zero as part of its strict
/// protocol fingerprint. Bytes after the first boundary are allowed because
/// TCP can coalesce several TPKTs in one payload.
pub fn extract_tpkt_header(packet: &[u8]) -> Result<TpktHeader, S7CommParseError> {
    validate_min_size(packet.len(), TPKT_HEADER_LENGTH)?;
    validate_tpkt_version(packet[0])?;
    validate_tpkt_reserved(packet[1])?;

    let length = u16::from_be_bytes([packet[2], packet[3]]);
    validate_tpkt_length(usize::from(length), packet.len())?;

    Ok(TpktHeader {
        version: packet[0],
        reserved: packet[1],
        length,
    })
}

/// Checks the COTP header bytes (starting at offset 4) and returns the typed
/// header.
///
/// S7Comm data must use the three-byte COTP DT form: `02 f0 xx`. Connection
/// PDUs are COTP but are not S7Comm packets and are rejected here. For class 0,
/// the seven-bit TPDU-NR is required to be zero. The EOT bit must be set because
/// this parser does not reassemble segmented DT TPDUs.
pub fn extract_cotp_header(packet: &[u8]) -> Result<CotpHeader, S7CommParseError> {
    validate_min_size(packet.len(), TPKT_MINIMUM_LENGTH)?;

    let cotp_len = packet[4];
    validate_cotp_length_indicator(cotp_len)?;
    validate_cotp_pdu_type(packet[5])?;
    let cotp_end = checked_offset(TPKT_HEADER_LENGTH, usize::from(cotp_len) + 1, "COTP header")?;
    validate_cotp_header_length(cotp_end, packet.len())?;
    let last_data_unit = (packet[6] & 0x80) != 0;
    validate_cotp_eot(last_data_unit)?;
    let tpdu_number = packet[6] & 0x7f;
    validate_cotp_tpdu_number(tpdu_number)?;

    Ok(CotpHeader {
        length: cotp_len,
        pdu_type: packet[5],
        destination_reference: 0,
        source_reference: 0,
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
    let base_end = checked_offset(s7_start, S7_BASE_HEADER_LENGTH, "S7 base header")?;
    validate_s7_header_length(base_end, packet.len())?;

    let protocol_id = packet[s7_start];
    validate_s7_protocol_id(protocol_id)?;
    let rosctr = packet[s7_start + 1];
    let header_length = s7_header_length(rosctr)?;
    let header_end = checked_offset(s7_start, header_length, "S7 header")?;
    validate_s7_header_length(header_end, packet.len())?;

    let reserved = u16::from_be_bytes([packet[s7_start + 2], packet[s7_start + 3]]);
    validate_s7_reserved(reserved)?;

    let (error_class, error_code) = if matches!(rosctr, 0x02 | 0x03) {
        (Some(packet[s7_start + 10]), Some(packet[s7_start + 11]))
    } else {
        (None, None)
    };

    Ok(S7Header {
        protocol_id,
        rosctr,
        reserved,
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
/// Verifies the 2-byte item header, variable-specification type `0x12`, a
/// present syntax identifier, and the announced item length via
/// [`validate_parameter_item_header`] and [`validate_parameter_item_length`],
/// then decodes the S7ANY addressing fields (guarded by
/// [`validate_s7any_length`]) when the item is an S7ANY variable
/// specification (`spec_type == 0x12`, `syntax_id == 0x10`). S7ANY's length
/// is exactly 10; accepting longer records would reinterpret another format's
/// prefix as an S7ANY address. Items using another syntax id (e.g. 0xB2,
/// symbolic S7-1200 addressing) expose their real syntax id, keep zeroed
/// S7ANY-only fields, and retain their raw bytes.
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

    validate_parameter_specification_type(spec_type)?;
    validate_parameter_syntax_id_present(length as u8)?;
    validate_parameter_item_length(offset, length, data.len())?;

    let syntax_id = if length > 0 { data[offset + 2] } else { 0 };
    let raw_end = checked_offset(
        checked_offset(offset, 2, "parameter item header")?,
        length,
        "parameter item",
    )?;

    if spec_type == 0x12 && syntax_id == 0x10 {
        validate_s7any_length(length as u8)?;

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
            raw: Some(&data[offset..raw_end]),
        })
    } else {
        Ok(S7ParameterItem {
            spec_type,
            length: length as u8,
            syntax_id,
            transport_size: 0,
            count: 0,
            db_number: 0,
            area: 0,
            address: 0,
            raw: Some(&data[offset..raw_end]),
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
    fn test_validate_tpkt_reserved_and_length() {
        assert!(validate_tpkt_reserved(0).is_ok());
        assert_eq!(
            validate_tpkt_reserved(1).unwrap_err(),
            S7CommParseError::InvalidTpktReserved { reserved: 1 }
        );
        assert!(validate_tpkt_length(31, 31).is_ok());
        assert!(validate_tpkt_length(31, 62).is_ok());
        assert_eq!(
            validate_tpkt_length(6, 31).unwrap_err(),
            S7CommParseError::InvalidTpktLength {
                declared: 6,
                minimum: 7,
            }
        );
        assert_eq!(
            validate_tpkt_length(32, 31).unwrap_err(),
            S7CommParseError::TruncatedTpkt {
                declared: 32,
                actual: 31,
            }
        );
    }

    #[test]
    fn test_validate_cotp_dt_fields() {
        assert!(validate_cotp_length_indicator(2).is_ok());
        assert_eq!(
            validate_cotp_length_indicator(3).unwrap_err(),
            S7CommParseError::InvalidCotpLengthIndicator { length: 3 }
        );
        assert!(validate_cotp_pdu_type(0xf0).is_ok());
        assert_eq!(
            validate_cotp_pdu_type(0xe0).unwrap_err(),
            S7CommParseError::InvalidCotpPduType { pdu_type: 0xe0 }
        );
        assert!(validate_cotp_eot(true).is_ok());
        assert_eq!(
            validate_cotp_eot(false).unwrap_err(),
            S7CommParseError::CotpNotLastDataUnit
        );
        assert!(validate_cotp_tpdu_number(0).is_ok());
        assert_eq!(
            validate_cotp_tpdu_number(0x7f).unwrap_err(),
            S7CommParseError::InvalidCotpTpduNumber { tpdu_number: 0x7f }
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
    fn test_validate_s7_identity_and_header_lengths() {
        assert!(validate_s7_protocol_id(0x32).is_ok());
        assert_eq!(
            validate_s7_protocol_id(0x33).unwrap_err(),
            S7CommParseError::InvalidS7ProtocolId { protocol_id: 0x33 }
        );

        for rosctr in [0x01, 0x02, 0x03, 0x07] {
            assert!(validate_s7_rosctr(rosctr).is_ok());
        }
        assert_eq!(
            validate_s7_rosctr(0x99).unwrap_err(),
            S7CommParseError::InvalidS7Rosctr { rosctr: 0x99 }
        );
        assert_eq!(s7_header_length(0x01).unwrap(), 10);
        assert_eq!(s7_header_length(0x07).unwrap(), 10);
        assert_eq!(s7_header_length(0x02).unwrap(), 12);
        assert_eq!(s7_header_length(0x03).unwrap(), 12);

        assert!(validate_s7_reserved(0).is_ok());
        assert_eq!(
            validate_s7_reserved(1).unwrap_err(),
            S7CommParseError::InvalidS7Reserved { reserved: 1 }
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
        assert_eq!(
            validate_parameter_item_header(usize::MAX, usize::MAX).unwrap_err(),
            S7CommParseError::LengthOverflow {
                context: "parameter item header"
            }
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
    fn test_validate_parameter_item_identity() {
        assert!(validate_parameter_specification_type(0x12).is_ok());
        assert_eq!(
            validate_parameter_specification_type(0x13).unwrap_err(),
            S7CommParseError::InvalidParameterSpecificationType { spec_type: 0x13 }
        );
        assert!(validate_parameter_syntax_id_present(1).is_ok());
        assert_eq!(
            validate_parameter_syntax_id_present(0).unwrap_err(),
            S7CommParseError::MissingParameterSyntaxId
        );
    }

    #[test]
    fn test_validate_s7any_length() {
        assert!(validate_s7any_length(10).is_ok());
        assert_eq!(
            validate_s7any_length(11).unwrap_err(),
            S7CommParseError::InvalidS7AnyLength { length: 11 }
        );
    }

    #[test]
    fn test_checked_offset_never_wraps() {
        assert_eq!(checked_offset(7, 10, "test").unwrap(), 17);
        assert_eq!(
            checked_offset(usize::MAX, 1, "test").unwrap_err(),
            S7CommParseError::LengthOverflow { context: "test" }
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
    fn test_extract_tpkt_header_rejects_reserved_and_truncation() {
        let mut bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        bytes[1] = 1;
        assert_eq!(
            extract_tpkt_header(&bytes).unwrap_err(),
            S7CommParseError::InvalidTpktReserved { reserved: 1 }
        );

        bytes[1] = 0;
        bytes[2..4].copy_from_slice(&32_u16.to_be_bytes());
        assert_eq!(
            extract_tpkt_header(&bytes).unwrap_err(),
            S7CommParseError::TruncatedTpkt {
                declared: 32,
                actual: 31,
            }
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
    fn test_extract_cotp_header_rejects_connection_request() {
        let bytes: &[u8] = &[
            0x03, 0x00, 0x00, 0x0b, // TPKT
            0x06, 0xe0, 0x12, 0x34, 0x56, 0x78, 0x00, // COTP CR
        ];
        assert_eq!(
            extract_cotp_header(bytes).unwrap_err(),
            S7CommParseError::InvalidCotpLengthIndicator { length: 6 }
        );
    }

    #[test]
    fn test_extract_cotp_header_rejects_invalid_dt_fields() {
        let mut bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        bytes[4] = 0xff;
        assert_eq!(
            extract_cotp_header(&bytes).unwrap_err(),
            S7CommParseError::InvalidCotpLengthIndicator { length: 0xff }
        );

        bytes[4] = 2;
        bytes[5] = 0xe0;
        assert_eq!(
            extract_cotp_header(&bytes).unwrap_err(),
            S7CommParseError::InvalidCotpPduType { pdu_type: 0xe0 }
        );

        bytes[5] = 0xf0;
        bytes[6] = 0;
        assert_eq!(
            extract_cotp_header(&bytes).unwrap_err(),
            S7CommParseError::CotpNotLastDataUnit
        );

        bytes[6] = 0xff;
        assert_eq!(
            extract_cotp_header(&bytes).unwrap_err(),
            S7CommParseError::InvalidCotpTpduNumber { tpdu_number: 0x7f }
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
    fn test_extract_s7_header_ack_error_fields() {
        // Synthetic ACK with the mandatory error class/code bytes.
        let bytes = hex::decode("0300001302f080320200000001000000003456").expect("valid hex");
        let s7 = extract_s7_header(&bytes, 7).expect("valid S7 ACK header");
        assert_eq!(s7.rosctr, 0x02);
        assert_eq!(s7.error_class, Some(0x34));
        assert_eq!(s7.error_code, Some(0x56));
    }

    #[test]
    fn test_extract_s7_header_rejects_identity_reserved_and_truncated_ack() {
        let mut bytes = hex::decode(READ_VAR_REQUEST).expect("valid hex");
        bytes[7] = 0x33;
        assert_eq!(
            extract_s7_header(&bytes, 7).unwrap_err(),
            S7CommParseError::InvalidS7ProtocolId { protocol_id: 0x33 }
        );

        bytes[7] = 0x32;
        bytes[8] = 0x99;
        assert_eq!(
            extract_s7_header(&bytes, 7).unwrap_err(),
            S7CommParseError::InvalidS7Rosctr { rosctr: 0x99 }
        );

        bytes[8] = 0x01;
        bytes[9] = 1;
        assert_eq!(
            extract_s7_header(&bytes, 7).unwrap_err(),
            S7CommParseError::InvalidS7Reserved { reserved: 0x0100 }
        );

        let truncated_ack =
            hex::decode("0300001202f0803203000000010000000000").expect("synthetic truncated ACK");
        assert_eq!(
            extract_s7_header(&truncated_ack, 7).unwrap_err(),
            S7CommParseError::S7HeaderTooShort {
                expected: 19,
                actual: 18,
            }
        );
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
        assert_eq!(item.syntax_id, 0xb2);
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

    #[test]
    fn test_extract_parameter_item_rejects_invalid_type_and_missing_syntax() {
        assert_eq!(
            extract_parameter_item(&[0x13, 0x01, 0xb2], 0).unwrap_err(),
            S7CommParseError::InvalidParameterSpecificationType { spec_type: 0x13 }
        );
        assert_eq!(
            extract_parameter_item(&[0x12, 0x00], 0).unwrap_err(),
            S7CommParseError::MissingParameterSyntaxId
        );
    }

    #[test]
    fn test_extract_parameter_item_rejects_non_exact_s7any_length() {
        let data = [
            0x12, 0x0b, 0x10, 0x02, 0x00, 0x01, 0x00, 0x00, 0x83, 0x00, 0x00, 0x00, 0xff,
        ];
        assert_eq!(
            extract_parameter_item(&data, 0).unwrap_err(),
            S7CommParseError::InvalidS7AnyLength { length: 11 }
        );
    }
}
