// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use crate::{
    checks::application::copt::{
        checked_offset, extract_connection_fields, extract_length_and_pdu_type, extract_parameter,
        validate_connection_references, validate_cr_cc_class_options, validate_declared_len,
        validate_disconnect_reason, validate_extended_credit_code, validate_fixed_header_len,
        validate_normal_sequence_number_high_bit, validate_reject_cause,
        validate_sequence_number_high_bit, validate_user_data,
    },
    errors::application::copt::CotpParseError,
};

/// COTP PDU Types
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CotpPduType {
    ExpeditedData = 0x10,
    ExpeditedDataAcknowledgement = 0x20,
    Reject = 0x50,
    DataAcknowledgement = 0x60,
    Data = 0xF0,
    ConnectionRequest = 0xE0,
    ConnectionConfirm = 0xD0,
    DisconnectRequest = 0x80,
    DisconnectConfirm = 0xC0,
    TpduError = 0x70,
    Other(u8),
}

impl From<u8> for CotpPduType {
    fn from(value: u8) -> Self {
        // CR/CC and normal-format AK/RJ use their low nibble for credit. The
        // remaining TPDU codes supported here are fixed full-octet values.
        match value {
            0x10 => CotpPduType::ExpeditedData,
            0x20 => CotpPduType::ExpeditedDataAcknowledgement,
            0xF0 => CotpPduType::Data,
            0x80 => CotpPduType::DisconnectRequest,
            0xC0 => CotpPduType::DisconnectConfirm,
            0x70 => CotpPduType::TpduError,
            _ if value & 0xF0 == 0xE0 => CotpPduType::ConnectionRequest,
            _ if value & 0xF0 == 0xD0 => CotpPduType::ConnectionConfirm,
            _ if value & 0xF0 == 0x50 => CotpPduType::Reject,
            _ if value & 0xF0 == 0x60 => CotpPduType::DataAcknowledgement,
            _ => CotpPduType::Other(value),
        }
    }
}

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// COTP header.
///
/// The diagram shows a connection/control TPDU. A normal-format Data TPDU is
/// only three bytes (`LI=2`, PDU type `0xf0`, then EOT + TPDU number); its user
/// data begins immediately afterward.
///
/// Unknown TPDU codes are deliberately preserved as [`CotpPduType::Other`]
/// for lossless decoding. Consequently, `CotpHeader::try_from(data).is_ok()`
/// alone is not a safe protocol fingerprint. Use `PacketFlow` when detecting
/// traffic: it additionally requires RFC 1006 TPKT framing, TCP port 102 and a
/// fully determined COTP layout.
///
/// ```mermaid
/// ---
/// title: CotpHeader
/// ---
/// packet-beta
/// 0-7: "Header Length u8"
/// 8-15: "PDU Type u8"
/// 16-31: "Destination Reference u16"
/// 32-47: "Source Reference u16"
/// 48-55: "Class / Options u8"
/// 56-127: "Parameters variable"
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CotpHeader<'a> {
    pub length: u8,
    pub pdu_type: CotpPduType,
    pub dst_ref: u16,
    pub src_ref: u16,
    pub class: u8,
    pub extended_formats: bool,
    pub no_explicit_flow_control: bool,
    /// Initial credit on CR/CC/normal AK/RJ, or the 16-bit extended credit.
    pub credit: Option<u16>,
    /// DR reason code (RFC 905 section 13.5), absent on every other TPDU.
    pub disconnect_reason: Option<u8>,
    /// ER reject cause (RFC 905 section 13.12), absent on every other TPDU.
    pub reject_cause: Option<u8>,
    /// Numbering format for DT/ED/EA/AK/RJ, when that can be inferred safely.
    pub number_format: Option<CotpNumberFormat>,
    /// Fixed bytes after the TPDU code, preserved for unsupported/ambiguous
    /// layouts instead of being interpreted with guessed offsets.
    pub raw_fixed_part: &'a [u8],
    pub parameters: Vec<CotpParameter<'a>>,
    /// User data after the LI-delimited header for TPDU types that allow it.
    pub user_data: &'a [u8],
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CotpNumberFormat {
    /// Class 0/1 DT: no destination reference, 7-bit TPDU number.
    Class01Normal,
    /// Classes 1-4 normal 7-bit TPDU numbering.
    Normal,
    /// Classes 2-4 extended 31-bit TPDU numbering.
    Extended,
    /// LI is bounded but the format cannot be inferred without connection state.
    Undetermined,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CotpParameter<'a> {
    TpduSize(u8),   // 0xC0: TPDU size
    SrcTsap(u16),   // 0xC1: Source TSAP
    DstTsap(u16),   // 0xC2: Destination TSAP
    TpduNumber(u8), // Low 7 bits of the DT header's third octet
    TpduNumberExtended(u32),
    Eot(bool), // High bit of the DT header's third octet
    Checksum(u16),
    AtnExtendedChecksum32(u32),
    AtnExtendedChecksum16(u16),
    DisconnectAdditionalInfo(&'a [u8]),
    InvalidTpdu(&'a [u8]),
    Other(u8, &'a [u8]), // Other parameters (raw bytes borrowed from the packet)
}

impl<'a> CotpHeader<'a> {
    fn parse_parameters(
        data: &'a [u8],
        mut offset: usize,
        declared_end: usize,
        pdu_type: CotpPduType,
    ) -> Result<Vec<CotpParameter<'a>>, CotpParseError> {
        let mut parameters = Vec::new();
        while offset < declared_end {
            let (parameter, next_offset) = extract_parameter(data, offset, declared_end, pdu_type)?;
            parameters.push(parameter);
            offset = next_offset;
        }
        Ok(parameters)
    }

    /// Parse one COTP header from a byte slice.
    ///
    /// Canonical linear sequence: length pre-check, then one `extract_*` per
    /// wire field (each check lives in `checks::application::copt`), then
    /// construction from the already-validated values.
    ///
    /// Unknown TPDU codes are returned as [`CotpPduType::Other`] with their
    /// LI-delimited bytes in [`CotpHeader::raw_fixed_part`]. This tolerant API
    /// is a decoder, not a protocol detector; use `PacketFlow` for strict
    /// TPKT/TCP/102 fingerprinting.
    pub fn from_bytes(data: &'a [u8]) -> Result<(Self, usize), CotpParseError> {
        let (length, pdu_type) = extract_length_and_pdu_type(data)?;
        let declared_end = checked_offset(usize::from(length), 1, "length indicator")?;

        validate_declared_len(data.len(), declared_end)?;
        let empty = &data[declared_end..declared_end];
        let mut header = Self {
            length,
            pdu_type,
            dst_ref: 0,
            src_ref: 0,
            class: 0,
            extended_formats: false,
            no_explicit_flow_control: false,
            credit: None,
            disconnect_reason: None,
            reject_cause: None,
            number_format: None,
            raw_fixed_part: &data[2..declared_end],
            parameters: Vec::new(),
            user_data: empty,
        };

        match pdu_type {
            CotpPduType::ConnectionRequest | CotpPduType::ConnectionConfirm => {
                const FIXED_END: usize = 7;
                validate_fixed_header_len(
                    declared_end,
                    FIXED_END,
                    if pdu_type == CotpPduType::ConnectionRequest {
                        "CR"
                    } else {
                        "CC"
                    },
                )?;
                let credit = data[1] & 0x0F;
                let (dst_ref, src_ref, _, _, _) = extract_connection_fields(data, 2, declared_end)?;
                let (class, extended_formats, no_explicit_flow_control) =
                    validate_cr_cc_class_options(data[6], credit)?;
                validate_connection_references(pdu_type, dst_ref, src_ref)?;
                if pdu_type == CotpPduType::ConnectionRequest && data.len() > 128 {
                    return Err(CotpParseError::PduTooLong {
                        pdu: "CR",
                        maximum: 128,
                        actual: data.len(),
                    });
                }

                header.dst_ref = dst_ref;
                header.src_ref = src_ref;
                header.class = class;
                header.extended_formats = extended_formats;
                header.no_explicit_flow_control = no_explicit_flow_control;
                header.credit = Some(u16::from(credit));
                header.raw_fixed_part = &data[2..FIXED_END];
                header.parameters =
                    Self::parse_parameters(data, FIXED_END, declared_end, pdu_type)?;
                header.user_data = &data[declared_end..];
                validate_user_data(
                    if pdu_type == CotpPduType::ConnectionRequest {
                        "CR"
                    } else {
                        "CC"
                    },
                    Some(class),
                    header.user_data.len(),
                    32,
                )?;
            }
            CotpPduType::DisconnectRequest => {
                const FIXED_END: usize = 7;
                validate_fixed_header_len(declared_end, FIXED_END, "DR")?;
                let fixed = &data[2..FIXED_END];
                header.dst_ref = u16::from_be_bytes([fixed[0], fixed[1]]);
                header.src_ref = u16::from_be_bytes([fixed[2], fixed[3]]);
                validate_disconnect_reason(fixed[4])?;
                header.disconnect_reason = Some(fixed[4]);
                header.raw_fixed_part = fixed;
                header.parameters =
                    Self::parse_parameters(data, FIXED_END, declared_end, pdu_type)?;
                header.user_data = &data[declared_end..];
                validate_user_data("DR", None, header.user_data.len(), 64)?;
            }
            CotpPduType::DisconnectConfirm => {
                const FIXED_END: usize = 6;
                validate_fixed_header_len(declared_end, FIXED_END, "DC")?;
                let fixed = &data[2..FIXED_END];
                header.dst_ref = u16::from_be_bytes([fixed[0], fixed[1]]);
                header.src_ref = u16::from_be_bytes([fixed[2], fixed[3]]);
                header.raw_fixed_part = fixed;
                header.parameters =
                    Self::parse_parameters(data, FIXED_END, declared_end, pdu_type)?;
            }
            CotpPduType::TpduError => {
                const FIXED_END: usize = 5;
                validate_fixed_header_len(declared_end, FIXED_END, "ER")?;
                let fixed = &data[2..FIXED_END];
                header.dst_ref = u16::from_be_bytes([fixed[0], fixed[1]]);
                validate_reject_cause(fixed[2])?;
                header.reject_cause = Some(fixed[2]);
                header.raw_fixed_part = fixed;
                header.parameters =
                    Self::parse_parameters(data, FIXED_END, declared_end, pdu_type)?;
            }
            CotpPduType::Data
            | CotpPduType::ExpeditedData
            | CotpPduType::ExpeditedDataAcknowledgement => {
                let has_eot = matches!(pdu_type, CotpPduType::Data | CotpPduType::ExpeditedData);
                let has_user_data =
                    matches!(pdu_type, CotpPduType::Data | CotpPduType::ExpeditedData);

                if length == 2 && pdu_type == CotpPduType::Data {
                    let number_and_eot = data[2];
                    header.number_format = Some(CotpNumberFormat::Class01Normal);
                    header.raw_fixed_part = &data[2..3];
                    header
                        .parameters
                        .push(CotpParameter::TpduNumber(number_and_eot & 0x7F));
                    header
                        .parameters
                        .push(CotpParameter::Eot(number_and_eot & 0x80 != 0));
                } else if length >= 4 && length % 2 == 0 {
                    let fixed_end = 5;
                    let number_and_eot = data[4];
                    if !has_eot {
                        validate_normal_sequence_number_high_bit("EA", number_and_eot)?;
                    }
                    header.dst_ref = u16::from_be_bytes([data[2], data[3]]);
                    header.number_format = Some(CotpNumberFormat::Normal);
                    header.raw_fixed_part = &data[2..fixed_end];
                    header
                        .parameters
                        .push(CotpParameter::TpduNumber(number_and_eot & 0x7F));
                    if has_eot {
                        let eot = number_and_eot & 0x80 != 0;
                        if pdu_type == CotpPduType::ExpeditedData && !eot {
                            return Err(CotpParseError::MissingEot { pdu: "ED" });
                        }
                        header.parameters.push(CotpParameter::Eot(eot));
                    }
                    header.parameters.extend(Self::parse_parameters(
                        data,
                        fixed_end,
                        declared_end,
                        pdu_type,
                    )?);
                } else if length >= 7 && length % 2 == 1 {
                    let fixed_end = 8;
                    let number_and_eot = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                    if !has_eot {
                        validate_sequence_number_high_bit("EA", number_and_eot)?;
                    }
                    header.dst_ref = u16::from_be_bytes([data[2], data[3]]);
                    header.number_format = Some(CotpNumberFormat::Extended);
                    header.raw_fixed_part = &data[2..fixed_end];
                    header.parameters.push(CotpParameter::TpduNumberExtended(
                        number_and_eot & 0x7FFF_FFFF,
                    ));
                    if has_eot {
                        let eot = number_and_eot & 0x8000_0000 != 0;
                        if pdu_type == CotpPduType::ExpeditedData && !eot {
                            return Err(CotpParseError::MissingEot { pdu: "ED" });
                        }
                        header.parameters.push(CotpParameter::Eot(eot));
                    }
                    header.parameters.extend(Self::parse_parameters(
                        data,
                        fixed_end,
                        declared_end,
                        pdu_type,
                    )?);
                } else {
                    return Err(CotpParseError::InvalidTpduHeaderLength {
                        pdu: if pdu_type == CotpPduType::Data {
                            "DT"
                        } else if pdu_type == CotpPduType::ExpeditedData {
                            "ED"
                        } else {
                            "EA"
                        },
                        length,
                    });
                }

                if has_user_data {
                    header.user_data = &data[declared_end..];
                }
                if pdu_type == CotpPduType::ExpeditedData {
                    if header.user_data.is_empty() {
                        return Err(CotpParseError::UserDataTooShort {
                            pdu: "ED",
                            minimum: 1,
                            actual: 0,
                        });
                    }
                    validate_user_data("ED", None, header.user_data.len(), 16)?;
                }
            }
            CotpPduType::DataAcknowledgement => {
                if length >= 4 && length % 2 == 0 {
                    let fixed_end = 5;
                    validate_normal_sequence_number_high_bit("AK", data[4])?;
                    header.dst_ref = u16::from_be_bytes([data[2], data[3]]);
                    header.credit = Some(u16::from(data[1] & 0x0F));
                    header.number_format = Some(CotpNumberFormat::Normal);
                    header.raw_fixed_part = &data[2..fixed_end];
                    header
                        .parameters
                        .push(CotpParameter::TpduNumber(data[4] & 0x7F));
                    header.parameters.extend(Self::parse_parameters(
                        data,
                        fixed_end,
                        declared_end,
                        pdu_type,
                    )?);
                } else if length >= 9 && length % 2 == 1 {
                    let fixed_end = 10;
                    validate_extended_credit_code("AK", data[1])?;
                    let number = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                    validate_sequence_number_high_bit("AK", number)?;
                    header.dst_ref = u16::from_be_bytes([data[2], data[3]]);
                    header.credit = Some(u16::from_be_bytes([data[8], data[9]]));
                    header.number_format = Some(CotpNumberFormat::Extended);
                    header.raw_fixed_part = &data[2..fixed_end];
                    header
                        .parameters
                        .push(CotpParameter::TpduNumberExtended(number & 0x7FFF_FFFF));
                    header.parameters.extend(Self::parse_parameters(
                        data,
                        fixed_end,
                        declared_end,
                        pdu_type,
                    )?);
                } else {
                    return Err(CotpParseError::InvalidTpduHeaderLength { pdu: "AK", length });
                }
            }
            CotpPduType::Reject => match length {
                4 => {
                    let fixed_end = 5;
                    validate_normal_sequence_number_high_bit("RJ", data[4])?;
                    header.dst_ref = u16::from_be_bytes([data[2], data[3]]);
                    header.credit = Some(u16::from(data[1] & 0x0F));
                    header.number_format = Some(CotpNumberFormat::Normal);
                    header.raw_fixed_part = &data[2..fixed_end];
                    header
                        .parameters
                        .push(CotpParameter::TpduNumber(data[4] & 0x7F));
                }
                9 => {
                    let fixed_end = 10;
                    validate_extended_credit_code("RJ", data[1])?;
                    let number = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
                    validate_sequence_number_high_bit("RJ", number)?;
                    header.dst_ref = u16::from_be_bytes([data[2], data[3]]);
                    header.credit = Some(u16::from_be_bytes([data[8], data[9]]));
                    header.number_format = Some(CotpNumberFormat::Extended);
                    header.raw_fixed_part = &data[2..fixed_end];
                    header
                        .parameters
                        .push(CotpParameter::TpduNumberExtended(number & 0x7FFF_FFFF));
                }
                _ => {
                    return Err(CotpParseError::InvalidTpduHeaderLength { pdu: "RJ", length });
                }
            },
            CotpPduType::Other(_) => {}
        }

        Ok((header, declared_end))
    }
}

impl<'a> TryFrom<&'a [u8]> for CotpHeader<'a> {
    type Error = CotpParseError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let (header, _) = Self::from_bytes(data)?;
        Ok(header)
    }
}

impl fmt::Display for CotpHeader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self.pdu_type {
            CotpPduType::ExpeditedData => "Expedited Data (ED)",
            CotpPduType::ExpeditedDataAcknowledgement => "Expedited Data Acknowledgement (EA)",
            CotpPduType::DataAcknowledgement => "Data Acknowledgement (AK)",
            CotpPduType::Reject => "Reject (RJ)",
            CotpPduType::Data => "Data (DT)",
            CotpPduType::ConnectionRequest => "Connection Request (CR)",
            CotpPduType::ConnectionConfirm => "Connection Confirm (CC)",
            CotpPduType::DisconnectRequest => "Disconnect Request (DR)",
            CotpPduType::DisconnectConfirm => "Disconnect Confirm (DC)",
            CotpPduType::TpduError => "TPDU Error (ER)",
            CotpPduType::Other(code) => return write!(f, "Unknown PDU Type: 0x{code:02X}"),
        };

        writeln!(f, "COTP: {type_str}")?;
        writeln!(f, "  Length: {}", self.length)?;
        writeln!(f, "  Destination reference: 0x{:04X}", self.dst_ref)?;
        writeln!(f, "  Source reference: 0x{:04X}", self.src_ref)?;

        if self.class != 0 {
            writeln!(f, "  Class: {}", self.class)?;
        }
        if self.extended_formats {
            writeln!(f, "  Extended formats: True")?;
        }
        if self.no_explicit_flow_control {
            writeln!(f, "  No explicit flow control: True")?;
        }
        if let Some(credit) = self.credit {
            writeln!(f, "  Credit: {credit}")?;
        }
        if let Some(reason) = self.disconnect_reason {
            writeln!(f, "  Disconnect reason: 0x{reason:02X}")?;
        }
        if let Some(cause) = self.reject_cause {
            writeln!(f, "  Reject cause: 0x{cause:02X}")?;
        }

        for param in &self.parameters {
            match param {
                CotpParameter::TpduSize(size) => {
                    let tpdu_size = (0x07..=0x0D)
                        .contains(size)
                        .then(|| 1u32 << u32::from(*size));
                    match tpdu_size {
                        Some(tpdu_size) => writeln!(f, "  TPDU size: {tpdu_size} bytes")?,
                        None => writeln!(f, "  TPDU size: invalid (code 0x{size:02X})")?,
                    }
                }
                CotpParameter::SrcTsap(tsap) => {
                    writeln!(f, "  Source TSAP: 0x{tsap:04X}")?;
                }
                CotpParameter::DstTsap(tsap) => {
                    writeln!(f, "  Destination TSAP: 0x{tsap:04X}")?;
                }
                CotpParameter::TpduNumber(num) => {
                    writeln!(f, "  TPDU Number: {num}")?;
                }
                CotpParameter::TpduNumberExtended(num) => {
                    writeln!(f, "  TPDU Number: {num}")?;
                }
                CotpParameter::Eot(eot) => {
                    writeln!(f, "  End of TSDU: {}", if *eot { "Yes" } else { "No" })?;
                }
                CotpParameter::Checksum(checksum) => {
                    writeln!(f, "  Checksum: 0x{checksum:04X}")?;
                }
                CotpParameter::AtnExtendedChecksum32(checksum) => {
                    writeln!(f, "  ATN extended checksum: 0x{checksum:08X}")?;
                }
                CotpParameter::AtnExtendedChecksum16(checksum) => {
                    writeln!(f, "  ATN extended checksum: 0x{checksum:04X}")?;
                }
                CotpParameter::DisconnectAdditionalInfo(data) => {
                    writeln!(
                        f,
                        "  Disconnect additional information: {} byte(s)",
                        data.len()
                    )?;
                }
                CotpParameter::InvalidTpdu(data) => {
                    writeln!(f, "  Invalid TPDU: {} byte(s)", data.len())?;
                }
                CotpParameter::Other(code, data) => {
                    write!(f, "  Parameter 0x{code:02X}: ")?;
                    for byte in *data {
                        write!(f, "{byte:02X} ")?;
                    }
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cc_packet() {
        // Example COTP Connection Confirm (CC) packet
        let data = [
            0x11, // Length
            0xD0, // CC (Connection Confirm)
            0x00, 0x01, // Destination reference
            0x00, 0x03, // Source reference
            0x00, // Class and options
            0xC0, 0x01, 0x09, // TPDU size = 512
            0xC1, 0x02, 0x01, 0x00, // Source TSAP = 0x0100
            0xC2, 0x02, 0x01, 0x02, // Destination TSAP = 0x0102
        ];

        let (header, bytes_parsed) = CotpHeader::from_bytes(&data).unwrap();
        assert_eq!(bytes_parsed, data.len());
        assert!(matches!(header.pdu_type, CotpPduType::ConnectionConfirm));
        assert_eq!(header.dst_ref, 0x0001);
        assert_eq!(header.src_ref, 0x0003);

        // Check parameters
        let mut has_tpdu_size = false;
        let mut has_src_tsap = false;
        let mut has_dst_tsap = false;

        for param in &header.parameters {
            match param {
                CotpParameter::TpduSize(0x09) => has_tpdu_size = true,
                CotpParameter::SrcTsap(0x0100) => has_src_tsap = true,
                CotpParameter::DstTsap(0x0102) => has_dst_tsap = true,
                _ => {}
            }
        }

        assert!(has_tpdu_size);
        assert!(has_src_tsap);
        assert!(has_dst_tsap);
    }

    #[test]
    fn test_pdu_type_from_u8() {
        assert!(matches!(CotpPduType::from(0xF0), CotpPduType::Data));
        assert!(matches!(
            CotpPduType::from(0xE0),
            CotpPduType::ConnectionRequest
        ));
        assert!(matches!(
            CotpPduType::from(0xD0),
            CotpPduType::ConnectionConfirm
        ));
        assert!(matches!(
            CotpPduType::from(0x80),
            CotpPduType::DisconnectRequest
        ));
        assert!(matches!(
            CotpPduType::from(0xC0),
            CotpPduType::DisconnectConfirm
        ));
        assert!(matches!(CotpPduType::from(0x70), CotpPduType::TpduError));
        assert!(matches!(
            CotpPduType::from(0xE5),
            CotpPduType::ConnectionRequest
        ));
        assert!(matches!(
            CotpPduType::from(0xDF),
            CotpPduType::ConnectionConfirm
        ));
        assert!(matches!(CotpPduType::from(0xF1), CotpPduType::Other(0xF1)));
        assert!(matches!(CotpPduType::from(0x42), CotpPduType::Other(0x42)));
    }

    #[test]
    fn test_parse_data_packet_with_eot_and_tpdu_number() {
        let data = [0x02, 0xF0, 0x85]; // EOT + TPDU number 5

        let (header, bytes_parsed) = CotpHeader::from_bytes(&data).unwrap();
        assert_eq!(bytes_parsed, data.len());
        assert!(matches!(header.pdu_type, CotpPduType::Data));
        assert_eq!(header.dst_ref, 0);
        assert_eq!(header.src_ref, 0);
        assert!(matches!(header.parameters[0], CotpParameter::TpduNumber(5)));
        assert!(matches!(header.parameters[1], CotpParameter::Eot(true)));
    }

    #[test]
    fn test_parse_minimal_data_packet_stops_before_user_data() {
        let data = [0x02, 0xF0, 0x80, 0x32, 0x01, 0x00, 0x00];

        let (header, bytes_parsed) = CotpHeader::from_bytes(&data).unwrap();

        assert_eq!(bytes_parsed, 3);
        assert!(matches!(header.pdu_type, CotpPduType::Data));
        assert!(matches!(header.parameters[0], CotpParameter::TpduNumber(0)));
        assert!(matches!(header.parameters[1], CotpParameter::Eot(true)));
    }

    #[test]
    fn test_parse_connection_request_with_class_and_flags() {
        let data = [
            0x06, // Length
            0xE5, // CR + initial credit 5
            0x00, 0x00, // dst_ref (must be zero in CR)
            0x00, 0x02, // src_ref
            0x23, // class 2, extended formats + no explicit flow control
        ];

        let header = CotpHeader::try_from(&data[..]).unwrap();
        assert!(matches!(header.pdu_type, CotpPduType::ConnectionRequest));
        assert_eq!(header.class, 2);
        assert!(header.extended_formats);
        assert!(header.no_explicit_flow_control);
        assert_eq!(header.credit, Some(5));
    }

    #[test]
    fn test_parse_unknown_parameter() {
        let data = [
            0x0A, // Length
            0xD0, // CC
            0x00, 0x01, 0x00, 0x03, 0x00, // refs + class
            0xC5, 0x02, 0xAB, 0xCD, // paramètre inconnu 0xC5
        ];

        let (header, _) = CotpHeader::from_bytes(&data).unwrap();
        match &header.parameters[0] {
            CotpParameter::Other(0xC5, bytes) => {
                // Zero-copy : la slice pointe dans le paquet original.
                assert_eq!(*bytes, &data[9..11]);
                assert_eq!(*bytes, &[0xAB, 0xCD][..]);
            }
            other => panic!("attendu Other(0xC5, ..), obtenu {other:?}"),
        }
    }

    #[test]
    fn test_parse_c0_with_unusual_length_is_rejected() {
        let data = [
            0x0A, // Length
            0xD0, // CC
            0x00, 0x01, 0x00, 0x03, 0x00, // refs + class
            0xC0, 0x02, 0x09, 0x0A,
        ];

        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::InvalidParameterValueLength {
                code: 0xC0,
                expected: 1,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_packet_too_short() {
        assert!(matches!(
            CotpHeader::from_bytes(&[0x02]),
            Err(CotpParseError::PacketTooShort { .. })
        ));
    }

    #[test]
    fn test_declared_length_exceeds_packet() {
        // length déclare 0x20 octets, paquet bien plus court
        let data = [0x20, 0xF0, 0xC0, 0x01, 0x05, 0x80, 0x00];
        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::LengthExceedsPacket { .. })
        ));
    }

    #[test]
    fn test_connection_header_too_short() {
        // CR avec length trop courte pour contenir refs + class
        let data = [0x04, 0xE0, 0x00, 0x01, 0x00, 0x02, 0x00];
        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::FixedHeaderTooShort { pdu: "CR", .. })
        ));
    }

    #[test]
    fn test_parameter_length_exceeds_packet() {
        let data = [
            0x08, // Length
            0xD0, // CC
            0x00, 0x01, 0x00, 0x03, 0x00, // refs + class
            0xC1, 0x0A, // param annonce 10 octets absents
        ];
        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::ParameterLengthExceedsPacket { .. })
        ));
    }

    #[test]
    fn test_parameter_header_truncated() {
        // Un seul octet de paramètre restant : impossible de lire type + longueur.
        let data = [
            0x07, // Length : declared_end = 8
            0xD0, // CC
            0x00, 0x01, 0x00, 0x03, 0x00, // refs + class
            0xC1, // début de paramètre tronqué
        ];
        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::ParameterHeaderTruncated { offset: 7 })
        ));
    }

    #[test]
    fn test_rejects_non_standard_data_header_length() {
        let data = [0x03, 0xF0, 0x80, 0x00];
        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::InvalidTpduHeaderLength {
                pdu: "DT",
                length: 3
            })
        ));
    }

    #[test]
    fn test_display_connection_confirm() {
        let data = [
            0x11, 0xD0, 0x00, 0x01, 0x00, 0x03, 0x00, 0xC0, 0x01, 0x09, 0xC1, 0x02, 0x01, 0x00,
            0xC2, 0x02, 0x01, 0x02,
        ];
        let header = CotpHeader::try_from(&data[..]).unwrap();
        let rendered = header.to_string();

        assert!(rendered.contains("COTP: Connection Confirm (CC)"));
        assert!(rendered.contains("Destination reference: 0x0001"));
        assert!(rendered.contains("Source reference: 0x0003"));
        assert!(rendered.contains("TPDU size: 512 bytes"));
        assert!(rendered.contains("Source TSAP: 0x0100"));
        assert!(rendered.contains("Destination TSAP: 0x0102"));
    }

    #[test]
    fn test_display_data_header() {
        let data = [0x02, 0xF0, 0x87];
        let header = CotpHeader::try_from(&data[..]).unwrap();
        let rendered = header.to_string();

        assert!(rendered.contains("COTP: Data (DT)"));
        assert!(rendered.contains("TPDU Number: 7"));
        assert!(rendered.contains("End of TSDU: Yes"));
    }

    #[test]
    fn test_display_unknown_pdu_type() {
        let data = [0x01, 0x42, 0x00];
        let (header, _) = CotpHeader::from_bytes(&data).unwrap();
        assert_eq!(header.to_string(), "Unknown PDU Type: 0x42");
    }

    #[test]
    fn test_display_class_and_flags() {
        let data = [0x06, 0xE5, 0x00, 0x00, 0x00, 0x02, 0x23];
        let header = CotpHeader::try_from(&data[..]).unwrap();
        let rendered = header.to_string();

        assert!(rendered.contains("Class: 2"));
        assert!(rendered.contains("Extended formats: True"));
        assert!(rendered.contains("No explicit flow control: True"));
        assert!(rendered.contains("Credit: 5"));
    }

    /// Regression : un octet TPDU size hostile (>= 26) faisait paniquer le
    /// Display par overflow de shift (`1 << 32`) en build debug.
    #[test]
    fn test_display_hostile_tpdu_size_does_not_panic() {
        let data = [0x06, 0xE0, 0x00, 0x00, 0x00, 0x02, 0x00];
        let mut header = CotpHeader::try_from(&data[..]).unwrap();
        header.parameters = vec![CotpParameter::TpduSize(0xFF)];
        let rendered = header.to_string();

        assert!(rendered.contains("TPDU size: invalid (code 0xFF)"));

        header.parameters = vec![CotpParameter::TpduSize(0x0E)];
        assert!(
            header
                .to_string()
                .contains("TPDU size: invalid (code 0x0E)")
        );
    }

    #[test]
    fn fixed_control_tpdu_layouts_are_distinct() {
        let (dr, consumed) =
            CotpHeader::from_bytes(&[0x06, 0x80, 0x00, 0x01, 0x00, 0x02, 0x80]).unwrap();
        assert_eq!(consumed, 7);
        assert_eq!(dr.pdu_type, CotpPduType::DisconnectRequest);
        assert_eq!((dr.dst_ref, dr.src_ref), (1, 2));
        assert_eq!(dr.disconnect_reason, Some(0x80));

        let (dc, consumed) = CotpHeader::from_bytes(&[0x05, 0xC0, 0x00, 0x01, 0x00, 0x02]).unwrap();
        assert_eq!(consumed, 6);
        assert_eq!(dc.pdu_type, CotpPduType::DisconnectConfirm);
        assert_eq!((dc.dst_ref, dc.src_ref), (1, 2));

        let (er, consumed) = CotpHeader::from_bytes(&[0x04, 0x70, 0x00, 0x01, 0x03]).unwrap();
        assert_eq!(consumed, 5);
        assert_eq!(er.pdu_type, CotpPduType::TpduError);
        assert_eq!(er.dst_ref, 1);
        assert_eq!(er.reject_cause, Some(3));
    }

    #[test]
    fn fixed_control_tpdus_parse_only_their_variable_parts() {
        let dr_data = [
            0x0B, 0x80, 0x00, 0x01, 0x00, 0x02, 0x80, 0xE0, 0x03, 0x01, 0x02, 0x03, 0xAA,
        ];
        let (dr, consumed) = CotpHeader::from_bytes(&dr_data).unwrap();
        assert_eq!(consumed, 12);
        assert_eq!(dr.user_data, &[0xAA]);
        assert_eq!(
            dr.parameters,
            vec![CotpParameter::DisconnectAdditionalInfo(&[0x01, 0x02, 0x03])]
        );

        let dc_data = [0x09, 0xC0, 0x00, 0x01, 0x00, 0x02, 0xC3, 0x02, 0x12, 0x34];
        let (dc, consumed) = CotpHeader::from_bytes(&dc_data).unwrap();
        assert_eq!(consumed, dc_data.len());
        assert_eq!(dc.parameters, vec![CotpParameter::Checksum(0x1234)]);

        let er_data = [0x09, 0x70, 0x00, 0x01, 0x00, 0xC1, 0x03, 0xF0, 0x80, 0x00];
        let (er, consumed) = CotpHeader::from_bytes(&er_data).unwrap();
        assert_eq!(consumed, er_data.len());
        assert_eq!(
            er.parameters,
            vec![CotpParameter::InvalidTpdu(&[0xF0, 0x80, 0x00])]
        );

        let invalid_dc = [0x08, 0xC0, 0x00, 0x01, 0x00, 0x02, 0xC0, 0x01, 0x07];
        assert!(matches!(
            CotpHeader::from_bytes(&invalid_dc),
            Err(CotpParseError::UnexpectedParameterCode {
                pdu: "DC",
                code: 0xC0
            })
        ));
    }

    #[test]
    fn every_control_tpdu_enforces_its_fixed_part_size() {
        for (data, pdu) in [
            (&[0x05, 0xE0, 0, 0, 0, 1][..], "CR"),
            (&[0x05, 0xD0, 0, 1, 0, 2][..], "CC"),
            (&[0x05, 0x80, 0, 1, 0, 2][..], "DR"),
            (&[0x04, 0xC0, 0, 1, 0][..], "DC"),
            (&[0x03, 0x70, 0, 1][..], "ER"),
        ] {
            assert!(matches!(
                CotpHeader::from_bytes(data),
                Err(CotpParseError::FixedHeaderTooShort { pdu: actual, .. }) if actual == pdu
            ));
        }
    }

    #[test]
    fn connection_references_classes_and_options_are_strict() {
        for data in [
            [0x06, 0xE0, 0, 1, 0, 2, 0],
            [0x06, 0xE0, 0, 0, 0, 0, 0],
            [0x06, 0xD0, 0, 0, 0, 2, 0],
            [0x06, 0xD0, 0, 1, 0, 0, 0],
            [0x06, 0xE0, 0, 0, 0, 2, 0x50],
            [0x06, 0xE0, 0, 0, 0, 2, 0x24],
            [0x06, 0xE0, 0, 0, 0, 2, 0x12],
            [0x06, 0xE0, 0, 0, 0, 2, 0x31],
            [0x06, 0xE1, 0, 0, 0, 2, 0x00],
        ] {
            assert!(
                CotpHeader::from_bytes(&data).is_err(),
                "accepted {data:02x?}"
            );
        }
    }

    #[test]
    fn connection_user_data_and_cr_total_size_are_bounded() {
        let class_zero_with_data = [0x06, 0xE0, 0, 0, 0, 1, 0, 0xAA];
        assert!(matches!(
            CotpHeader::from_bytes(&class_zero_with_data),
            Err(CotpParseError::UserDataNotAllowed {
                pdu: "CR",
                class: 0
            })
        ));

        let mut class_one = vec![0x06, 0xE0, 0, 0, 0, 1, 0x10];
        class_one.resize(7 + 32, 0xAA);
        assert_eq!(
            CotpHeader::from_bytes(&class_one)
                .unwrap()
                .0
                .user_data
                .len(),
            32
        );
        class_one.push(0xAA);
        assert!(matches!(
            CotpHeader::from_bytes(&class_one),
            Err(CotpParseError::UserDataTooLong { pdu: "CR", .. })
        ));

        let mut oversized_cr = vec![0x06, 0xE0, 0, 0, 0, 1, 0x10];
        oversized_cr.resize(129, 0);
        assert!(matches!(
            CotpHeader::from_bytes(&oversized_cr),
            Err(CotpParseError::PduTooLong {
                pdu: "CR",
                maximum: 128,
                actual: 129
            })
        ));
    }

    #[test]
    fn tsap_lengths_remain_lossless_while_fixed_parameters_are_strict() {
        let data = [0x0B, 0xD0, 0, 1, 0, 2, 0, 0xC1, 0x03, 1, 2, 3];
        let header = CotpHeader::from_bytes(&data).unwrap().0;
        assert_eq!(
            header.parameters,
            vec![CotpParameter::Other(0xC1, &[1, 2, 3])]
        );

        let invalid_tpdu_size = [0x0A, 0xD0, 0, 1, 0, 2, 0, 0xC0, 0x02, 0x09, 0x0A];
        assert!(matches!(
            CotpHeader::from_bytes(&invalid_tpdu_size),
            Err(CotpParseError::InvalidParameterValueLength { code: 0xC0, .. })
        ));
    }

    #[test]
    fn dt_supports_all_number_formats_and_repeated_checksums() {
        let class_01 = [0x02, 0xF0, 0x85, 0xAA];
        let (header, consumed) = CotpHeader::from_bytes(&class_01).unwrap();
        assert_eq!(consumed, 3);
        assert_eq!(header.number_format, Some(CotpNumberFormat::Class01Normal));
        assert_eq!(header.user_data, &[0xAA]);

        let normal = [0x04, 0xF0, 0x12, 0x34, 0x85, 0xAA];
        let (header, consumed) = CotpHeader::from_bytes(&normal).unwrap();
        assert_eq!(consumed, 5);
        assert_eq!(header.number_format, Some(CotpNumberFormat::Normal));
        assert_eq!(header.dst_ref, 0x1234);
        assert_eq!(header.user_data, &[0xAA]);

        let extended = [0x07, 0xF0, 0x12, 0x34, 0x80, 0, 0, 5, 0xAA];
        let (header, consumed) = CotpHeader::from_bytes(&extended).unwrap();
        assert_eq!(consumed, 8);
        assert_eq!(header.number_format, Some(CotpNumberFormat::Extended));
        assert!(
            header
                .parameters
                .contains(&CotpParameter::TpduNumberExtended(5))
        );
        assert!(header.parameters.contains(&CotpParameter::Eot(true)));

        let repeated = [
            0x0C, 0xF0, 0x12, 0x34, 0x80, 0xC3, 0x02, 0x11, 0x11, 0xC3, 0x02, 0x22, 0x22,
        ];
        let header = CotpHeader::from_bytes(&repeated).unwrap().0;
        assert_eq!(
            header
                .parameters
                .iter()
                .filter(|parameter| matches!(parameter, CotpParameter::Checksum(_)))
                .count(),
            2
        );
    }

    #[test]
    fn dt_supports_strict_atn_checksums_and_rejects_unrelated_tlvs() {
        let normal_atn32 = [0x0A, 0xF0, 0, 1, 0x80, 0x08, 0x04, 0x01, 0x02, 0x03, 0x04];
        let header = CotpHeader::from_bytes(&normal_atn32).unwrap().0;
        assert!(
            header
                .parameters
                .contains(&CotpParameter::AtnExtendedChecksum32(0x0102_0304))
        );

        let extended_atn16 = [0x0B, 0xF0, 0, 1, 0x80, 0, 0, 1, 0x09, 0x02, 0x12, 0x34];
        let header = CotpHeader::from_bytes(&extended_atn16).unwrap().0;
        assert!(
            header
                .parameters
                .contains(&CotpParameter::AtnExtendedChecksum16(0x1234))
        );

        let unrelated = [0x08, 0xF0, 0, 1, 0x80, 0xAA, 0x02, 0x00, 0x00];
        assert!(matches!(
            CotpHeader::from_bytes(&unrelated),
            Err(CotpParseError::UnexpectedParameterCode {
                pdu: "DT",
                code: 0xAA
            })
        ));
    }

    #[test]
    fn ed_requires_eot_and_one_to_sixteen_user_octets() {
        let normal = [0x04, 0x10, 0, 1, 0x82, 0xAA];
        let header = CotpHeader::from_bytes(&normal).unwrap().0;
        assert_eq!(header.number_format, Some(CotpNumberFormat::Normal));
        assert_eq!(header.user_data, &[0xAA]);

        let extended = [0x07, 0x10, 0, 1, 0x80, 0, 0, 2, 0xAA];
        assert_eq!(
            CotpHeader::from_bytes(&extended).unwrap().0.number_format,
            Some(CotpNumberFormat::Extended)
        );

        assert!(matches!(
            CotpHeader::from_bytes(&[0x04, 0x10, 0, 1, 0x02, 0xAA]),
            Err(CotpParseError::MissingEot { pdu: "ED" })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x04, 0x10, 0, 1, 0x82]),
            Err(CotpParseError::UserDataTooShort { pdu: "ED", .. })
        ));

        let mut too_long = vec![0x04, 0x10, 0, 1, 0x82];
        too_long.resize(5 + 17, 0);
        assert!(matches!(
            CotpHeader::from_bytes(&too_long),
            Err(CotpParseError::UserDataTooLong { pdu: "ED", .. })
        ));
    }

    #[test]
    fn ea_rejects_the_reserved_sequence_number_bit() {
        let normal = CotpHeader::from_bytes(&[0x04, 0x20, 0, 1, 0x05]).unwrap().0;
        assert_eq!(normal.number_format, Some(CotpNumberFormat::Normal));
        assert!(normal.parameters.contains(&CotpParameter::TpduNumber(5)));

        let extended = CotpHeader::from_bytes(&[0x07, 0x20, 0, 1, 0x00, 0, 0, 5])
            .unwrap()
            .0;
        assert_eq!(extended.number_format, Some(CotpNumberFormat::Extended));

        assert!(matches!(
            CotpHeader::from_bytes(&[0x04, 0x20, 0, 1, 0x85]),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "EA" })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x07, 0x20, 0, 1, 0x80, 0, 0, 5]),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "EA" })
        ));
    }

    #[test]
    fn ak_supports_normal_extended_and_repeatable_variable_parameters() {
        let normal = CotpHeader::from_bytes(&[0x04, 0x65, 0, 1, 5]).unwrap().0;
        assert_eq!(normal.credit, Some(5));
        assert_eq!(normal.number_format, Some(CotpNumberFormat::Normal));

        let extended = CotpHeader::from_bytes(&[0x09, 0x60, 0, 1, 0, 0, 0, 5, 0, 16])
            .unwrap()
            .0;
        assert_eq!(extended.credit, Some(16));
        assert_eq!(extended.number_format, Some(CotpNumberFormat::Extended));

        let variable = [
            0x12, 0x65, 0, 1, 5, // normal fixed part
            0x8A, 0x02, 0, 1, // subsequence
            0x8C, 0x08, 0, 0, 0, 1, 0, 2, 0, 3, // flow-control confirmation
        ];
        let header = CotpHeader::from_bytes(&variable).unwrap().0;
        assert_eq!(header.parameters.len(), 3);
        assert!(matches!(
            header.parameters[1],
            CotpParameter::Other(0x8A, _)
        ));
        assert!(matches!(
            header.parameters[2],
            CotpParameter::Other(0x8C, _)
        ));

        let repeated_checksums = [0x0C, 0x61, 0, 1, 5, 0xC3, 0x02, 0, 1, 0xC3, 0x02, 0, 2];
        assert!(CotpHeader::from_bytes(&repeated_checksums).is_ok());
    }

    #[test]
    fn ak_rejects_reserved_bits_wrong_codes_and_wrong_lengths() {
        assert!(matches!(
            CotpHeader::from_bytes(&[0x04, 0x61, 0, 1, 0x85]),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "AK" })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x09, 0x61, 0, 1, 0, 0, 0, 5, 0, 1]),
            Err(CotpParseError::InvalidExtendedCreditCode { pdu: "AK", .. })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x09, 0x60, 0, 1, 0x80, 0, 0, 5, 0, 1]),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "AK" })
        ));

        let wrong_code = [0x08, 0x61, 0, 1, 5, 0x8B, 0x02, 0, 1];
        assert!(matches!(
            CotpHeader::from_bytes(&wrong_code),
            Err(CotpParseError::UnexpectedParameterCode {
                pdu: "AK",
                code: 0x8B
            })
        ));
        let wrong_length = [0x08, 0x61, 0, 1, 5, 0x8C, 0x02, 0, 1];
        assert!(matches!(
            CotpHeader::from_bytes(&wrong_length),
            Err(CotpParseError::InvalidParameterValueLength { code: 0x8C, .. })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x05, 0x60, 0, 1, 5, 0]),
            Err(CotpParseError::InvalidTpduHeaderLength {
                pdu: "AK",
                length: 5
            })
        ));
    }

    #[test]
    fn rj_has_exact_normal_or_extended_layouts() {
        let normal = CotpHeader::from_bytes(&[0x04, 0x5F, 0, 1, 5]).unwrap().0;
        assert_eq!(normal.credit, Some(15));
        assert_eq!(normal.number_format, Some(CotpNumberFormat::Normal));

        let extended = CotpHeader::from_bytes(&[0x09, 0x50, 0, 1, 0, 0, 0, 5, 0, 16])
            .unwrap()
            .0;
        assert_eq!(extended.credit, Some(16));
        assert_eq!(extended.number_format, Some(CotpNumberFormat::Extended));

        assert!(matches!(
            CotpHeader::from_bytes(&[0x08, 0x5F, 0, 1, 5, 0xC3, 2, 0, 0]),
            Err(CotpParseError::InvalidTpduHeaderLength {
                pdu: "RJ",
                length: 8
            })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x09, 0x51, 0, 1, 0, 0, 0, 5, 0, 16]),
            Err(CotpParseError::InvalidExtendedCreditCode { pdu: "RJ", .. })
        ));
        assert!(matches!(
            CotpHeader::from_bytes(&[0x04, 0x50, 0, 1, 0x80]),
            Err(CotpParseError::ReservedSequenceNumberBit { pdu: "RJ" })
        ));
    }

    #[test]
    fn li_zero_and_255_are_rejected_before_indexing() {
        for data in [[0x00, 0xF0], [0xFF, 0xF0]] {
            assert!(matches!(
                CotpHeader::from_bytes(&data),
                Err(CotpParseError::InvalidLengthIndicator { .. })
            ));
        }
    }

    #[test]
    fn non_user_tpdus_are_consumed_one_at_a_time() {
        let concatenated = [
            0x05, 0xC0, 0, 1, 0, 2, // DC
            0x04, 0x70, 0, 1, 0, // ER
        ];
        let (dc, first_len) = CotpHeader::from_bytes(&concatenated).unwrap();
        assert_eq!(dc.pdu_type, CotpPduType::DisconnectConfirm);
        assert_eq!(first_len, 6);
        assert!(dc.user_data.is_empty());

        let (er, second_len) = CotpHeader::from_bytes(&concatenated[first_len..]).unwrap();
        assert_eq!(er.pdu_type, CotpPduType::TpduError);
        assert_eq!(second_len, 5);
        assert!(er.user_data.is_empty());
    }
}
