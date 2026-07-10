// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use crate::{
    checks::application::copt::{
        parse_cotp_parameter, validate_connection_header_len, validate_declared_len,
        validate_min_len, validate_parameter_header, validate_parameter_len,
    },
    errors::application::copt::CotpParseError,
};

/// COTP PDU Types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum CotpPduType {
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
        match value {
            0xF0 => CotpPduType::Data,
            0xE0 => CotpPduType::ConnectionRequest,
            0xD0 => CotpPduType::ConnectionConfirm,
            0x80 => CotpPduType::DisconnectRequest,
            0xC0 => CotpPduType::DisconnectConfirm,
            0x70 => CotpPduType::TpduError,
            x => CotpPduType::Other(x),
        }
    }
}

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// COTP Header
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
#[derive(Debug, Clone)]
pub struct CotpHeader<'a> {
    pub length: u8,
    pub pdu_type: CotpPduType,
    pub dst_ref: u16,
    pub src_ref: u16,
    pub class: u8,
    pub extended_formats: bool,
    pub no_explicit_flow_control: bool,
    pub parameters: Vec<CotpParameter<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CotpParameter<'a> {
    TpduSize(u8),        // 0xC0: TPDU size
    SrcTsap(u16),        // 0xC1: Source TSAP
    DstTsap(u16),        // 0xC2: Destination TSAP
    TpduNumber(u8),      // 0xC0 in DT TPDU
    Eot(bool),           // 0x80 in DT TPDU
    Other(u8, &'a [u8]), // Other parameters (raw bytes borrowed from the packet)
}

impl<'a> CotpHeader<'a> {
    /// Minimum size of a COTP header (3 bytes for basic header)
    pub const MIN_SIZE: usize = 7; // 1 + 1 + 2 + 2 + 1 (for CR/CC)

    /// Parse a COTP header from a byte slice
    pub fn from_bytes(data: &'a [u8]) -> Result<(Self, usize), CotpParseError> {
        validate_min_len(data)?;

        let length = data[0];
        let pdu_type = CotpPduType::from(data[1]);
        let declared_end = length as usize + 1;

        validate_declared_len(data.len(), declared_end)?;

        let mut offset = 2; // Skip length and PDU type
        let (dst_ref, src_ref, class, extended_formats, no_explicit_flow_control) = match pdu_type {
            CotpPduType::ConnectionRequest
            | CotpPduType::ConnectionConfirm
            | CotpPduType::DisconnectRequest
            | CotpPduType::DisconnectConfirm
            | CotpPduType::TpduError => {
                let expected = offset + 5;
                validate_connection_header_len(declared_end, expected)?;
                let dst_ref = u16::from_be_bytes([data[offset], data[offset + 1]]);
                let src_ref = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                let class = data[offset + 4] >> 4;
                let extended_formats = (data[offset + 4] & 0x04) != 0;
                let no_explicit_flow_control = (data[offset + 4] & 0x01) != 0;
                offset += 5;
                (
                    dst_ref,
                    src_ref,
                    class,
                    extended_formats,
                    no_explicit_flow_control,
                )
            }
            _ => (0, 0, 0, false, false),
        };

        // Parse parameters
        let mut parameters = Vec::new();
        while offset < declared_end {
            validate_parameter_header(declared_end, offset)?;

            let param_type = data[offset];
            let param_len = data[offset + 1] as usize;

            validate_parameter_len(declared_end, offset, param_len)?;

            let param_data = &data[offset + 2..offset + 2 + param_len];

            let param = parse_cotp_parameter(pdu_type, param_type, offset, param_data)?;

            parameters.push(param);
            offset += 2 + param_len;
        }

        Ok((
            Self {
                length,
                pdu_type,
                dst_ref,
                src_ref,
                class,
                extended_formats,
                no_explicit_flow_control,
                parameters,
            },
            offset,
        ))
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

        for param in &self.parameters {
            match param {
                CotpParameter::TpduSize(size) => {
                    let tpdu_size = match size {
                        0x09 => 512,
                        0x0A => 1024,
                        0x0B => 2048,
                        0x0C => 4096,
                        0x0D => 8192,
                        _ => 1 << (*size as u16 + 6),
                    };
                    writeln!(f, "  TPDU size: {tpdu_size} bytes")?;
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
                CotpParameter::Eot(_) => {
                    writeln!(f, "  End of TSDU: Yes")?;
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
        assert!(matches!(CotpPduType::from(0x42), CotpPduType::Other(0x42)));
    }

    #[test]
    fn test_parse_data_packet_with_eot_and_tpdu_number() {
        let data = [
            0x06, // Length : 6 octets après ce champ
            0xF0, // DT (Data)
            0xC0, 0x01, 0x05, // TPDU number = 5
            0x80, 0x00, // EOT
        ];

        let (header, bytes_parsed) = CotpHeader::from_bytes(&data).unwrap();
        assert_eq!(bytes_parsed, data.len());
        assert!(matches!(header.pdu_type, CotpPduType::Data));
        assert_eq!(header.dst_ref, 0);
        assert_eq!(header.src_ref, 0);
        assert!(matches!(header.parameters[0], CotpParameter::TpduNumber(5)));
        assert!(matches!(header.parameters[1], CotpParameter::Eot(true)));
    }

    #[test]
    fn test_parse_connection_request_with_class_and_flags() {
        let data = [
            0x06, // Length
            0xE0, // CR
            0x00, 0x01, // dst_ref
            0x00, 0x02, // src_ref
            0x45, // class 4, extended formats, no explicit flow control
        ];

        let header = CotpHeader::try_from(&data[..]).unwrap();
        assert!(matches!(header.pdu_type, CotpPduType::ConnectionRequest));
        assert_eq!(header.class, 4);
        assert!(header.extended_formats);
        assert!(header.no_explicit_flow_control);
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
    fn test_parse_c0_with_unusual_length_maps_to_other() {
        // 0xC0 avec param_len 2 sur un CC : ni TpduNumber ni TpduSize
        let data = [
            0x0A, // Length
            0xD0, // CC
            0x00, 0x01, 0x00, 0x03, 0x00, // refs + class
            0xC0, 0x02, 0x09, 0x0A,
        ];

        let (header, _) = CotpHeader::from_bytes(&data).unwrap();
        assert!(matches!(
            header.parameters[0],
            CotpParameter::Other(0xC0, _)
        ));
    }

    #[test]
    fn test_packet_too_short() {
        assert!(matches!(
            CotpHeader::from_bytes(&[0x02, 0xF0]),
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
            Err(CotpParseError::ConnectionHeaderTooShort { .. })
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
    fn test_data_tpdu_number_empty_parameter() {
        // 0xC0 avec longueur 0 dans un DT : TPDU number vide.
        let data = [
            0x03, // Length : declared_end = 4
            0xF0, // DT (Data)
            0xC0, 0x00, // TPDU number sans donnée
        ];
        assert!(matches!(
            CotpHeader::from_bytes(&data),
            Err(CotpParseError::ParameterLengthExceedsPacket { .. })
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
    fn test_display_data_with_parameters_and_other() {
        let data = [
            0x09, 0xF0, // DT
            0xC0, 0x01, 0x07, // TPDU number
            0x80, 0x00, // EOT
            0xC5, 0x01, 0xFF, // paramètre inconnu
        ];
        let header = CotpHeader::try_from(&data[..]).unwrap();
        let rendered = header.to_string();

        assert!(rendered.contains("COTP: Data (DT)"));
        assert!(rendered.contains("TPDU Number: 7"));
        assert!(rendered.contains("End of TSDU: Yes"));
        assert!(rendered.contains("Parameter 0xC5: FF"));
    }

    #[test]
    fn test_display_unknown_pdu_type() {
        let data = [0x01, 0x42, 0x00];
        let (header, _) = CotpHeader::from_bytes(&data).unwrap();
        assert_eq!(header.to_string(), "Unknown PDU Type: 0x42");
    }

    #[test]
    fn test_display_class_and_flags() {
        let data = [0x06, 0xE0, 0x00, 0x01, 0x00, 0x02, 0x45];
        let header = CotpHeader::try_from(&data[..]).unwrap();
        let rendered = header.to_string();

        assert!(rendered.contains("Class: 4"));
        assert!(rendered.contains("Extended formats: True"));
        assert!(rendered.contains("No explicit flow control: True"));
    }
}
