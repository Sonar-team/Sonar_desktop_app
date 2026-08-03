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
//! ```no_run
//! use packet_parser::parse::application::protocols::s7comm::S7CommPacket;
//!
//! // Example S7Comm packet (simplified for demonstration)
//! let raw_packet = [
//!     0x03, 0x00, 0x00, 0x16, 0x11, 0xE0, 0x00, 0x00,
//!     0x00, 0x01, 0x00, 0xC0, 0x01, 0x0A, 0xC1, 0x02,
//!     0x01, 0x00, 0xC2, 0x02, 0x01, 0x02
//! ];
//!
//! match S7CommPacket::try_from(&raw_packet[..]) {
//!     Ok(packet) => println!("Successfully parsed S7Comm packet: {:?}", packet),
//!     Err(e) => eprintln!("Failed to parse S7Comm packet: {}", e),
//! }
//! ```

use std::fmt;

use crate::{
    checks::application::s7comm::{
        extract_cotp_header, extract_parameter_item, extract_s7_header, extract_tpkt_header,
        validate_data_length, validate_min_size, validate_parameter_data_not_empty,
        validate_parameter_length,
    },
    errors::application::s7comm::S7CommParseError,
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
/// %% COTP Header
/// 32-39: "COTP Length u8"
/// 40-47: "COTP PDU Type u8"
/// 48-63: "COTP Dest Ref u16"
/// 64-79: "COTP Src Ref u16"
/// 80-87: "COTP Last Data Unit"
///
/// %% S7 Header
/// 88-95: "S7 Protocol ID u8"
/// 96-103: "S7 ROSCTR u8"
/// 104-119: "S7 Reserved u16"
/// 120-135: "S7 PDU Ref u16"
/// 136-151: "S7 Param Len u16"
/// 152-167: "S7 Data Len u16"
/// 168-175: "S7 Error Class"
/// 176-183: "S7 Error Code"
///
/// %% S7 Parameter
/// 184-191: "Param Function u8"
/// ```
#[derive(Debug)]
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
#[derive(Debug)]
pub struct TpktHeader {
    /// Protocol version (should be 0x03)
    pub version: u8,

    /// Reserved field (should be 0x00)
    pub reserved: u8,

    /// Total length of the TPKT packet (including header)
    pub length: u16,
}

/// COTP (Connection-Oriented Transport Protocol) Header
///
/// Defined in ISO 8073/X.224, this layer provides connection-oriented services.
#[derive(Debug)]
pub struct CotpHeader {
    /// Length of the COTP header
    pub length: u8,

    /// PDU type (0xF0 = Data TPDU)
    pub pdu_type: u8,

    /// Destination reference number
    pub destination_reference: u16,

    /// Source reference number
    pub source_reference: u16,

    /// Indicates if this is the last data unit
    pub last_data_unit: bool,
}

/// S7 Communication Protocol Header
///
/// This is the S7-specific protocol header that follows the COTP header.
#[derive(Debug)]
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

    /// Error class (only present in ACK/Error messages)
    pub error_class: Option<u8>,

    /// Error code (only present in ACK/Error messages)
    pub error_code: Option<u8>,
}

/// S7 Parameter section containing function code and items
///
/// This structure represents the parameter section of an S7Comm packet,
/// which contains the function code and associated parameter items.
#[derive(Debug)]
pub struct S7Parameter<'a> {
    /// Function code (e.g., 0x04 = Read Var, 0x05 = Write Var)
    pub function: u8,

    /// List of parameter items
    pub items: Vec<S7ParameterItem<'a>>,
}

/// Represents a single item in the S7 parameter section
///
/// This structure contains the addressing information for a single data item
/// being read from or written to the PLC.
#[derive(Debug)]
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

    /// Memory area (0x81 = Input, 0x82 = Output, 0x83 = DB, etc.)
    pub area: u8,

    /// Memory address (3-byte address in big-endian format)
    pub address: u32,

    /// Raw bytes of the parameter item (if needed for debugging)
    pub raw: Option<&'a [u8]>,
}

impl<'a> S7CommPacket<'a> {
    /// Minimum required size for an S7Comm packet (TPKT + COTP + S7 Header)
    pub const MIN_SIZES: usize = 4 + 3 + 10;

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
    /// ```no_run
    /// # use packet_parser::parse::application::protocols::s7comm::S7CommPacket;
    /// let packet_data = [/* raw packet data */];
    /// match S7CommPacket::try_from(&packet_data[..]) {
    ///     Ok(packet) => println!("Successfully parsed packet: {:?}", packet),
    ///     Err(e) => eprintln!("Failed to parse packet: {}", e),
    /// }
    /// ```
    fn parse(packet: &'a [u8]) -> Result<Self, S7CommParseError> {
        // Canonical linear sequence: length pre-check, then one extract_* per
        // header (each one checks the bytes of its own fields in checks),
        // then cross validations, then construction. The check order matches
        // the historical one so the same inputs yield the same errors.
        validate_min_size(packet.len(), Self::MIN_SIZES)?;

        // TPKT Header (4 bytes)
        let tpkt = extract_tpkt_header(packet)?;

        // COTP Header (starts at offset 4)
        let cotp = extract_cotp_header(packet)?;

        // S7 Header starts after TPKT + COTP (+1 for the length byte itself)
        let s7_start = 4 + cotp.length as usize + 1;
        let s7_header = extract_s7_header(packet, s7_start)?;

        // The parameter section starts right after the S7 header (10 bytes for header + 2 for error class/code if present)
        let s7_header_length = if s7_header.rosctr == 0x03 { 12 } else { 10 };
        let param_start = s7_start + s7_header_length;

        // If there's no parameter data, return an empty parameter section
        let parameter = if s7_header.parameter_length > 0 {
            let param_end = param_start + s7_header.parameter_length as usize;
            validate_parameter_length(param_end, packet.len())?;

            Self::parse_parameter(&packet[param_start..param_end])?
        } else {
            // Return empty parameter section
            S7Parameter {
                function: 0,
                items: Vec::new(),
            }
        };

        // Parse payload if present
        let payload = if s7_header.data_length > 0 {
            let data_start = param_start + s7_header.parameter_length as usize;
            let data_end = data_start + s7_header.data_length as usize;
            validate_data_length(data_end, packet.len())?;
            Some(&packet[data_start..data_end])
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
    fn parse_parameter(data: &'a [u8]) -> Result<S7Parameter<'a>, S7CommParseError> {
        validate_parameter_data_not_empty(data)?;

        // Cas "fonction seule" (ex: parameter_length = 1)
        if data.len() == 1 {
            return Ok(S7Parameter {
                function: data[0],
                items: Vec::new(),
            });
        }

        let function = data[0];
        let item_count = data[1] as usize;
        let mut items = Vec::with_capacity(item_count);
        let mut offset = 2;

        for _ in 0..item_count {
            let item = extract_parameter_item(data, offset)?;
            offset += 2 + item.length as usize;
            items.push(item);
        }

        // Important : certains paquets ont item_count=0 => OK.
        Ok(S7Parameter { function, items })
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
        write!(f, "S7Comm Protocol ")
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
        let result = S7CommPacket::try_from(&bytes[..]);
        assert!(
            result.is_ok(),
            "Failed to parse S7Comm packet: {:?}",
            result.err().unwrap()
        );
    }

    #[test]
    fn test_parameter_request_download() {
        let hex_str = "0300001402f080320300000e000001000000001a";
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let result = S7CommPacket::try_from(&bytes[..]);
        assert!(
            result.is_ok(),
            "Failed to parse S7Comm packet: {:?}",
            result.err().unwrap()
        );
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
        // Valid packet truncated after 25 bytes: declared parameter_length (14)
        // ends at offset 31, beyond the 25-byte packet -> InvalidParameterLength
        let hex_str = "0300001f02f080320100000013000e00000401120a10020001000083000000";
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");
        let err = S7CommPacket::try_from(&bytes[..25]).unwrap_err();
        assert_eq!(
            err,
            S7CommParseError::InvalidParameterLength {
                expected: 31,
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
}
