// Copyright (c) 2025 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! S7Comm protocol parser implementation.
//!
//! This module provides functionality to parse and handle S7Comm protocol packets,
//! which is a communication protocol used by Siemens S7 PLCs. The implementation
//! supports parsing of TPKT, COTP, and S7 protocol layers.
//!
//! # Example
//! ```
//! use packet_parser::parse::application::protocols::s7comm::S7CommPacket;
//!
//! // Read Var request, frame 11 from
//! // pcaps_exemple/protocols/s7comm/s7comm_varservice_libnodavedemo.pcap.
//! let raw_packet = [
//!     0x03, 0x00, 0x00, 0x1f, 0x02, 0xf0, 0x80, 0x32,
//!     0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00,
//!     0x00, 0x04, 0x01, 0x12, 0x0a, 0x10, 0x02, 0x00,
//!     0x40, 0x00, 0x01, 0x84, 0x00, 0x00, 0x00,
//! ];
//!
//! let packet = S7CommPacket::try_from(&raw_packet[..]).expect("valid S7Comm frame");
//! assert_eq!(packet.parameter.function, 0x04);
//! assert_eq!(packet.parameter.items[0].count, 64);
//! ```

use std::fmt;

use serde::Serialize;

use crate::{
    checks::application::s7comm::{
        checked_offset, extract_cotp_header, extract_parameter_item, extract_parameter_item_count,
        extract_s7_header, extract_tpkt_header, s7_header_length, validate_data_length,
        validate_min_size, validate_parameter_data_not_empty, validate_parameter_item_padding,
        validate_parameter_items_consumed, validate_parameter_length, validate_section_lengths,
    },
    errors::application::s7comm::S7CommParseError,
    parse::application::protocols::bounded_capacity,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// S7Comm Protocol Packet
///
/// ```mermaid
/// ---
/// title: S7CommPacket
/// ---
/// packet-beta
/// %% TPKT Header
/// 0-7: "TPKT Version u8"
/// 8-15: "TPKT Reserved u8"
/// 16-31: "TPKT Length u16"
///
/// %% COTP DT Header (fixed for S7Comm data)
/// 32-39: "COTP Length u8"
/// 40-47: "COTP PDU Type 0xf0"
/// 48-55: "COTP TPDU number / EOT"
///
/// %% Common S7 Header
/// 56-63: "S7 Protocol ID 0x32"
/// 64-71: "S7 ROSCTR u8"
/// 72-87: "S7 Reserved u16"
/// 88-103: "S7 PDU Ref u16"
/// 104-119: "S7 Param Len u16"
/// 120-135: "S7 Data Len u16"
/// 136-151: "ROSCTR-dependent tail: ACK error bytes, parameters, data"
/// ```
///
/// Job (`0x01`) and Userdata (`0x07`) headers are 10 bytes. ACK (`0x02`)
/// and ACK-Data (`0x03`) append error class/code and are 12 bytes. Parameter
/// and data slices are bounded by the first TPKT length even when TCP has
/// coalesced another TPKT after it.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct S7CommPacket<'a> {
    /// TPKT Header (RFC 1006)
    pub tpkt: TpktHeader,

    /// COTP Header (ISO 8073 / X.224)
    pub cotp: CotpHeader,

    /// S7 Communication Header (S7Comm)
    pub s7_header: S7Header,

    /// S7 Parameter section containing function code and items
    pub parameter: S7Parameter<'a>,

    /// Optional payload data
    pub payload: Option<&'a [u8]>,
}

/// TPKT (Transport Protocol Data Unit) Header (4 bytes)
///
/// Defined in RFC 1006, this is the outermost protocol layer.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TpktHeader {
    /// Protocol version (should be 0x03)
    pub version: u8,

    /// Reserved field.
    ///
    /// RFC 2126 allows receivers to ignore it, but this parser requires
    /// `0x00` as part of its strict application-protocol fingerprint.
    pub reserved: u8,

    /// Total length of the TPKT packet (including header)
    pub length: u16,
}

/// COTP (Connection-Oriented Transport Protocol) Header
///
/// Defined in ISO 8073/X.224, this layer provides connection-oriented services.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CotpHeader {
    /// Length of the COTP header
    pub length: u8,

    /// PDU type (0xF0 = Data TPDU)
    pub pdu_type: u8,

    /// Destination reference number (always zero for a DT TPDU)
    pub destination_reference: u16,

    /// Source reference number (always zero for a DT TPDU)
    pub source_reference: u16,

    /// Indicates if this is the last data unit. The class-0 TPDU number in
    /// the same wire byte is validated as zero.
    pub last_data_unit: bool,
}

/// S7 Communication Protocol Header
///
/// This is the S7-specific protocol header that follows the COTP header.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct S7Header {
    /// Protocol ID (should be 0x32 for S7Comm)
    pub protocol_id: u8,

    /// Message type (0x01 = Job, 0x02 = Ack, 0x03 = Ack-Data, 0x07 = Userdata)
    pub rosctr: u8,

    /// Reserved field (should be 0x0000)
    pub reserved: u16,

    /// PDU reference number
    pub pduref: u16,

    /// Length of the parameter section
    pub parameter_length: u16,

    /// Length of the data section
    pub data_length: u16,

    /// Error class (present for ACK and ACK-Data messages)
    pub error_class: Option<u8>,

    /// Error code (present for ACK and ACK-Data messages)
    pub error_code: Option<u8>,
}

/// S7 Parameter section containing function code and items
///
/// This structure represents the parameter section of an S7Comm packet,
/// which contains the function code and associated parameter items.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct S7Parameter<'a> {
    /// Function code (e.g., 0x04 = Read Var, 0x05 = Write Var)
    pub function: u8,

    /// List of parameter items
    pub items: Vec<S7ParameterItem<'a>>,

    /// Declared item count for Read/Write Var requests and responses.
    pub item_count: Option<u8>,

    /// Entire zero-copy parameter section, including unsupported formats and
    /// any fill bytes between variable specifications.
    pub raw: &'a [u8],
}

/// Represents a single item in the S7 parameter section
///
/// This structure contains the addressing information for a single data item
/// being read from or written to the PLC.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct S7ParameterItem<'a> {
    /// Specification type (0x12 = Variable Specification)
    pub spec_type: u8,

    /// Length of the specification
    pub length: u8,

    /// Syntax ID (0x10 = S7ANY)
    pub syntax_id: u8,

    /// Transport size (0x02 = BYTE, 0x04 = WORD, etc.)
    pub transport_size: u8,

    /// Number of elements to read/write (in transport-size units)
    pub count: u16,

    /// DB number (0 for non-DB areas)
    pub db_number: u16,

    /// Memory area (0x81 = Input, 0x82 = Output, 0x83 = M/flags, 0x84 = DB)
    pub area: u8,

    /// Memory address (3-byte address in big-endian format)
    pub address: u32,

    /// Raw bytes of the parameter item (if needed for debugging)
    pub raw: Option<&'a [u8]>,
}

impl<'a> S7CommPacket<'a> {
    /// Minimum required size for an S7Comm packet (TPKT + COTP + S7 Header)
    pub const MIN_SIZE: usize = 4 + 3 + 10;

    /// Deprecated misspelled alias for [`S7CommPacket::MIN_SIZE`].
    #[deprecated(note = "use S7CommPacket::MIN_SIZE")]
    pub const MIN_SIZES: usize = Self::MIN_SIZE;

    /// Attempts to parse a byte slice into an `S7CommPacket`.
    ///
    /// # Arguments
    /// * `packet` - A byte slice containing the raw S7Comm packet
    ///
    /// # Returns
    /// * `Ok(S7CommPacket)` if parsing was successful
    /// * `Err(S7CommParseError)` if the packet is malformed or incomplete
    ///
    /// # Example
    /// ```
    /// # use packet_parser::parse::application::protocols::s7comm::S7CommPacket;
    /// let packet_data = [
    ///     0x03, 0x00, 0x00, 0x13, 0x02, 0xf0, 0x80,
    ///     0x32, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x00,
    /// ];
    /// let packet = S7CommPacket::try_from(&packet_data[..]).expect("valid ACK-Data");
    /// assert_eq!(packet.s7_header.rosctr, 0x03);
    /// ```
    fn parse(packet: &'a [u8]) -> Result<Self, S7CommParseError> {
        // Canonical linear sequence: pre-check, checked extraction in wire
        // order, cross-field validation, then zero-copy construction.
        validate_min_size(packet.len(), Self::MIN_SIZE)?;

        let tpkt = extract_tpkt_header(packet)?;
        let tpkt_end = usize::from(tpkt.length);
        validate_min_size(tpkt_end, Self::MIN_SIZE)?;

        // Bound every subsequent read to the first TPKT. A TCP payload may
        // contain another complete TPKT after this one.
        let tpkt_packet = &packet[..tpkt_end];

        let cotp = extract_cotp_header(tpkt_packet)?;

        let cotp_with_indicator =
            checked_offset(usize::from(cotp.length), 1, "COTP length indicator")?;
        let s7_start = checked_offset(4, cotp_with_indicator, "S7 header start")?;
        let s7_header = extract_s7_header(tpkt_packet, s7_start)?;

        let header_length = s7_header_length(s7_header.rosctr)?;
        let param_start = checked_offset(s7_start, header_length, "parameter start")?;
        let param_end = checked_offset(
            param_start,
            usize::from(s7_header.parameter_length),
            "parameter section",
        )?;
        validate_parameter_length(param_end, tpkt_end)?;

        let data_end = checked_offset(
            param_end,
            usize::from(s7_header.data_length),
            "data section",
        )?;
        validate_data_length(data_end, tpkt_end)?;
        validate_section_lengths(data_end, tpkt_end)?;

        let parameter = if s7_header.parameter_length > 0 {
            Self::parse_parameter(&tpkt_packet[param_start..param_end], s7_header.rosctr)?
        } else {
            S7Parameter {
                function: 0,
                items: Vec::new(),
                item_count: None,
                raw: &tpkt_packet[param_start..param_start],
            }
        };

        let payload = if s7_header.data_length > 0 {
            Some(&tpkt_packet[param_end..data_end])
        } else {
            None
        };

        Ok(S7CommPacket {
            tpkt,
            cotp,
            s7_header,
            parameter,
            payload,
        })
    }

    /// Parses the S7 parameter section of the packet.
    ///
    /// This is a helper function used internally by `try_from` to parse
    /// the parameter section of an S7Comm packet.
    ///
    /// # Arguments
    /// * `data` - The parameter section bytes to parse
    ///
    /// # Returns
    /// * `Ok(S7Parameter)` if parsing was successful
    /// * `Err(S7CommParseError)` if the parameter data is invalid
    fn parse_parameter(data: &'a [u8], rosctr: u8) -> Result<S7Parameter<'a>, S7CommParseError> {
        validate_parameter_data_not_empty(data)?;

        let function = data[0];

        // Only Read Var and Write Var use the generic variable-specification
        // list, and only JOB messages carry those specifications. Their
        // ACK-Data parameters contain function + item count, while the
        // returned values stay in the data section.
        if !matches!(function, 0x04 | 0x05) || !matches!(rosctr, 0x01 | 0x03) {
            return Ok(S7Parameter {
                function,
                items: Vec::new(),
                item_count: None,
                raw: data,
            });
        }

        let item_count = extract_parameter_item_count(data)?;

        if rosctr == 0x03 {
            validate_parameter_items_consumed(2, data.len())?;
            return Ok(S7Parameter {
                function,
                items: Vec::new(),
                item_count: Some(item_count),
                raw: data,
            });
        }

        let count = usize::from(item_count);
        let remaining = data.len().saturating_sub(2);
        let mut items = Vec::with_capacity(bounded_capacity(count, remaining, 2));
        let mut offset = 2;

        for item_index in 0..count {
            let item = extract_parameter_item(data, offset)?;
            let item_wire_length = checked_offset(2, usize::from(item.length), "parameter item")?;
            offset = checked_offset(offset, item_wire_length, "parameter item")?;

            if item_wire_length % 2 != 0 && item_index + 1 < count {
                validate_parameter_item_padding(offset, data.len())?;
                offset = checked_offset(offset, 1, "parameter item padding")?;
            }

            items.push(item);
        }

        validate_parameter_items_consumed(offset, data.len())?;

        Ok(S7Parameter {
            function,
            items,
            item_count: Some(item_count),
            raw: data,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for S7CommPacket<'a> {
    type Error = S7CommParseError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        Self::parse(packet)
    }
}

impl<'a> fmt::Display for S7CommPacket<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "S7Comm ROSCTR=0x{:02x} PDU={} parameters={} data={}",
            self.s7_header.rosctr,
            self.s7_header.pduref,
            self.s7_header.parameter_length,
            self.s7_header.data_length
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_s7comm_try_from() {
        // The provided hex string
        let hex_str = "0300001f02f080320100000013000e00000401120a10020001000083000000";

        // Convert hex string to bytes
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");

        // Try to parse as S7Comm packet
        let result = S7CommPacket::try_from(&bytes[..]);

        // Check if parsing succeeded
        assert!(
            result.is_ok(),
            "Failed to parse S7Comm packet: {:?}",
            result.err().unwrap()
        );

        // Add more assertions based on the expected values from your packet
    }

    /// Golden test : trame 11 de
    /// pcaps_exemple/protocols/s7comm/s7comm_varservice_libnodavedemo.pcap,
    /// requete Read Var "DB 1.DBX 0.0 BYTE 64". Valeurs attendues verifiees
    /// avec Wireshark (tshark -O s7comm).
    #[test]
    fn test_s7comm_read_var_real_frame_decodes_s7any_item() {
        let hex_str = "0300001f02f080320100000000000e00000401120a10020040000184000000";
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");

        let packet = S7CommPacket::try_from(&bytes[..]).expect("valid Read Var frame");

        assert_eq!(packet.tpkt.length, 0x1f);
        assert_eq!(packet.cotp.pdu_type, 0xf0);
        // DT TPDU : pas de references, bit EOT positionne.
        assert_eq!(packet.cotp.destination_reference, 0);
        assert_eq!(packet.cotp.source_reference, 0);
        assert!(packet.cotp.last_data_unit);

        assert_eq!(packet.s7_header.protocol_id, 0x32);
        assert_eq!(packet.s7_header.rosctr, 0x01); // Job
        assert_eq!(packet.s7_header.pduref, 0);
        assert_eq!(packet.s7_header.parameter_length, 14);
        assert_eq!(packet.s7_header.data_length, 0);

        assert_eq!(packet.parameter.function, 0x04); // Read Var
        assert_eq!(packet.parameter.item_count, Some(1));
        assert_eq!(packet.parameter.raw, &bytes[17..31]);
        assert_eq!(packet.parameter.items.len(), 1);
        let item = &packet.parameter.items[0];
        assert_eq!(item.syntax_id, 0x10); // S7ANY
        assert_eq!(item.transport_size, 0x02); // BYTE
        assert_eq!(item.count, 64);
        assert_eq!(item.db_number, 1);
        assert_eq!(item.area, 0x84); // Data blocks (DB)
        assert_eq!(item.address, 0);
    }

    #[test]
    fn test_s7comm_parse() {
        let hex_str = "0300003102f080320100000e00002000001a00010000000000095f30413030303031500d31303030353030303030343030";
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let packet = S7CommPacket::try_from(&bytes[..]).expect("valid PLC control frame");
        assert_eq!(packet.parameter.function, 0x1a);
        assert_eq!(packet.parameter.item_count, None);
        assert_eq!(packet.parameter.raw, &bytes[17..49]);
        assert!(packet.parameter.items.is_empty());
    }

    #[test]
    fn test_parameter_request_download() {
        let hex_str = "0300001402f080320300000e000001000000001a";
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let packet = S7CommPacket::try_from(&bytes[..]).expect("valid Request Download response");
        assert_eq!(packet.parameter.function, 0x1a);
        assert_eq!(packet.parameter.item_count, None);
        assert_eq!(packet.parameter.raw, &[0x1a]);
    }

    #[test]
    fn test_truncated_parameter_item_length() {
        // parameter_length = 6, item declares length 0x0a but only 4 bytes
        // follow the item header -> InvalidParameterItemLength
        let hex_str = "0300001702f08032010000000100060000_0401120a1002".replace('_', "");
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let err = S7CommPacket::try_from(&bytes[..]).unwrap_err();
        assert_eq!(err, S7CommParseError::InvalidParameterItemLength);
    }

    #[test]
    fn test_truncated_parameter_item_header() {
        // parameter_length = 3, item_count = 1 but only 1 byte remains for the
        // 2-byte item header -> InvalidParameterItemHeader
        let hex_str = "0300001402f08032010000000100030000_040112".replace('_', "");
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let err = S7CommPacket::try_from(&bytes[..]).unwrap_err();
        assert_eq!(err, S7CommParseError::InvalidParameterItemHeader);
    }

    #[test]
    fn test_parameter_section_beyond_packet() {
        // The first TPKT declares 31 bytes but only 25 are available.
        let hex_str = "0300001f02f080320100000013000e00000401120a10020001000083000000";
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let err = S7CommPacket::try_from(&bytes[..25]).unwrap_err();
        assert_eq!(
            err,
            S7CommParseError::TruncatedTpkt {
                declared: 31,
                actual: 25,
            }
        );
    }

    #[test]
    fn test_data_section_beyond_packet() {
        // data_length = 4 declared but no data bytes present -> InvalidDataLength
        let hex_str = "0300001202f08032010000000100010004_04".replace('_', "");
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let err = S7CommPacket::try_from(&bytes[..]).unwrap_err();
        assert_eq!(
            err,
            S7CommParseError::InvalidDataLength {
                expected: 22,
                actual: 18,
            }
        );
    }

    #[test]
    fn test_parameter_items_skip_odd_length_fill_byte() {
        // Synthetic Read Var request with two short non-S7ANY descriptors.
        // Item 1 has a 3-byte wire length and is followed by one fill byte.
        let bytes = hex::decode("0300001a02f0803201000000010009000004021201b2001201b2")
            .expect("synthetic padded request");
        let packet = S7CommPacket::try_from(bytes.as_slice()).expect("valid item padding");

        assert_eq!(packet.parameter.item_count, Some(2));
        assert_eq!(packet.parameter.items.len(), 2);
        assert_eq!(packet.parameter.items[0].syntax_id, 0xb2);
        assert_eq!(packet.parameter.items[1].syntax_id, 0xb2);
        assert_eq!(packet.parameter.raw, &bytes[17..26]);
    }

    #[test]
    fn test_missing_odd_length_item_padding_is_rejected() {
        // Synthetic request: count=2, but the first odd-sized descriptor ends
        // exactly at the parameter boundary, so even its fill byte is absent.
        let bytes = hex::decode("0300001602f0803201000000010005000004021201b2")
            .expect("synthetic truncated padding");
        assert_eq!(
            S7CommPacket::try_from(bytes.as_slice()).unwrap_err(),
            S7CommParseError::MissingParameterItemPadding
        );
    }

    #[test]
    fn test_read_write_job_requires_variable_specification_and_syntax_id() {
        let mut invalid_type = hex::decode("0300001602f0803201000000010005000004011201b2")
            .expect("synthetic Read Var request");
        invalid_type[19] = 0x13;
        assert_eq!(
            S7CommPacket::try_from(invalid_type.as_slice()).unwrap_err(),
            S7CommParseError::InvalidParameterSpecificationType { spec_type: 0x13 }
        );

        let missing_syntax = hex::decode("0300001502f0803201000000010004000004011200")
            .expect("synthetic Read Var request without syntax id");
        assert_eq!(
            S7CommPacket::try_from(missing_syntax.as_slice()).unwrap_err(),
            S7CommParseError::MissingParameterSyntaxId
        );
    }

    #[test]
    fn test_ack_data_read_var_does_not_parse_response_count_as_an_item() {
        // Synthetic ACK-Data response with one unexpected parameter byte.
        // A valid response parameter is exactly `function + item_count`.
        let bytes = hex::decode("0300001602f0803203000000010003000000000401ff")
            .expect("synthetic ACK-Data response");
        assert_eq!(
            S7CommPacket::try_from(bytes.as_slice()).unwrap_err(),
            S7CommParseError::UnexpectedParameterBytes { remaining: 1 }
        );
    }

    #[test]
    fn test_s7_sections_must_fill_the_first_tpkt() {
        // Synthetic JOB whose lengths account for 17 bytes while TPKT claims
        // three additional bytes inside the first PDU.
        let bytes = hex::decode("0300001402f08032010000000100000000deadbe")
            .expect("synthetic inconsistent lengths");
        assert_eq!(
            S7CommPacket::try_from(bytes.as_slice()).unwrap_err(),
            S7CommParseError::InconsistentSectionLengths {
                sections_end: 17,
                tpkt_end: 20,
            }
        );
    }
}
