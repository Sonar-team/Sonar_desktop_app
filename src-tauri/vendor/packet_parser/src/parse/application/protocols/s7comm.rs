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

#[cfg_attr(doc, aquamarine::aquamarine)]
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
    /// * `Err(&'static str)` if the packet is malformed or incomplete
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
    pub fn try_from(packet: &'a [u8]) -> Result<Self, &'static str> {
        // println!("S7CommPacket::try_from: packet len: {:?}", packet);
        if packet.len() < Self::MIN_SIZES {
            return Err("Packet too short for S7Comm header");
        }

        // Parse TPKT Header (4 bytes)
        if packet[0] != 0x03 {
            return Err("Invalid TPKT version, expected 0x03");
        }

        let tpkt = TpktHeader {
            version: packet[0],
            reserved: packet[1],
            length: u16::from_be_bytes([packet[2], packet[3]]),
        };

        // Parse COTP Header (starts at offset 4)
        let cotp_len = packet[4] as usize;
        if 4 + cotp_len + 1 > packet.len() {
            return Err("Invalid COTP header length");
        }

        let cotp = CotpHeader {
            length: packet[4],
            pdu_type: packet[5],
            destination_reference: u16::from_be_bytes([packet[6], packet[7]]),
            source_reference: u16::from_be_bytes([packet[8], packet[9]]),
            last_data_unit: (packet[10] & 0x80) != 0,
        };

        // S7 Header starts after TPKT + COTP
        let s7_start = 4 + cotp.length as usize + 1; // +1 for the length byte itself
        // println!("S7 header start: {}", s7_start);

        if s7_start + 10 > packet.len() {
            // println!(
            //     "Packet too short for S7 header: need {} bytes, have {}",
            //     s7_start + 10,
            //     packet.len()
            // );
            return Err("Packet too short for S7 header");
        }

        // First create the header without error fields
        let mut s7_header = S7Header {
            protocol_id: packet[s7_start],
            rosctr: packet[s7_start + 1],
            reserved: u16::from_be_bytes([packet[s7_start + 2], packet[s7_start + 3]]),
            pduref: u16::from_be_bytes([packet[s7_start + 4], packet[s7_start + 5]]),
            parameter_length: u16::from_be_bytes([packet[s7_start + 6], packet[s7_start + 7]]),
            data_length: u16::from_be_bytes([packet[s7_start + 8], packet[s7_start + 9]]),
            error_class: None,
            error_code: None,
        };

        // Update error fields if this is an ACK/Error message
        if s7_header.rosctr == 0x03 && s7_start + 11 < packet.len() {
            s7_header.error_class = Some(packet[s7_start + 10]);
            s7_header.error_code = Some(packet[s7_start + 11]);
        }

        // println!("S7 Header: {:?}", s7_header);

        // Print packet structure for debugging - only print up to the packet length
        // println!("Packet structure:");
        // println!(
        //     "  TPKT: {:02x} {:02x} {:02x}{:02x}",
        //     packet[0], packet[1], packet[2], packet[3]
        // );
        // println!(
        //     "  COTP: {:02x} {:02x} {:02x}",
        //     packet[4], packet[5], packet[6]
        // );

        // Only print S7 header bytes that exist in the packet
        let s7_header_end = std::cmp::min(s7_start + 12, packet.len());
        // print!("  S7 Header: ");
        for _byte in packet.iter().take(s7_header_end).skip(s7_start) {
            // print!("{:02x} ", byte);
        }
        // println!();

        // The parameter section starts right after the S7 header (10 bytes for header + 2 for error class/code if present)
        let s7_header_length = if s7_header.rosctr == 0x03 { 12 } else { 10 };
        let param_start = s7_start + s7_header_length;

        // println!("Parameter section start: {}", param_start);

        // If there's no parameter data, return an empty parameter section
        let parameter = if s7_header.parameter_length > 0 {
            let param_end = param_start + s7_header.parameter_length as usize;

            if param_end > packet.len() {
                // println!(
                //     "Invalid parameter length: param_end={}, packet_len={}",
                //     param_end,
                //     packet.len()
                // );
                // println!("  TPKT length: {}", tpkt.length);
                // println!("  COTP length: {}", cotp.length);
                // println!("  S7 parameter_length: {}", s7_header.parameter_length);
                // println!("  S7 data_length: {}", s7_header.data_length);
                return Err("Invalid parameter length");
            }

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
            if data_end > packet.len() {
                return Err("Invalid data length");
            }
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
    /// * `Err(&'static str)` if the parameter data is invalid
    fn parse_parameter(data: &'a [u8]) -> Result<S7Parameter<'a>, &'static str> {
        if data.is_empty() {
            return Err("Empty parameter data");
        }

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
            if offset + 2 > data.len() {
                return Err("Invalid parameter item header");
            }

            let spec_type = data[offset];
            let length = data[offset + 1] as usize;

            if offset + 2 + length > data.len() {
                return Err("Invalid parameter item length");
            }

            if spec_type == 0x12 && length >= 0x0A {
                if offset + 12 > data.len() {
                    return Err("S7ANY parameter too short");
                }

                let syntax_id = data[offset + 2];
                let transport_size = data[offset + 3];
                let db_number = u16::from_be_bytes([data[offset + 5], data[offset + 6]]);
                let area = data[offset + 7];
                let address = ((data[offset + 8] as u32) << 16)
                    | ((data[offset + 9] as u32) << 8)
                    | (data[offset + 10] as u32);

                items.push(S7ParameterItem {
                    spec_type,
                    length: length as u8,
                    syntax_id,
                    transport_size,
                    db_number,
                    area,
                    address,
                    raw: Some(&data[offset..offset + 2 + length]),
                });
            } else {
                items.push(S7ParameterItem {
                    spec_type,
                    length: length as u8,
                    syntax_id: 0,
                    transport_size: 0,
                    db_number: 0,
                    area: 0,
                    address: 0,
                    raw: Some(&data[offset..offset + 2 + length]),
                });
            }

            offset += 2 + length;
        }

        // Important : certains paquets ont item_count=0 => OK.
        Ok(S7Parameter { function, items })
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
}
