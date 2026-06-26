// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::convert::TryFrom;

use crate::{
    checks::application::modbus_tcp::{
        validate_consumed_length, validate_declared_total_length, validate_length_field,
        validate_mbap_min_size, validate_pdu_not_empty, validate_protocol_identifier,
    },
    errors::application::modbus_tcp::ModbusTcpError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// Modbus/TCP Protocol Packet
///
/// ```mermaid
/// ---
/// title: ModbusTcpPacket
/// ---
/// packet-beta
/// 0-15: "Transaction Identifier u16"
/// 16-31: "Protocol Identifier u16"
/// 32-47: "Length u16"
/// 48-55: "Unit Identifier u8"
/// 56-63: "Function Code u8"
/// 64-127: "PDU Data variable"
/// ```
#[derive(Debug)]
pub struct ModbusTcpPacket<'a> {
    pub mbaps: Vec<MBAP<'a>>, // plusieurs MBAP dans un paquet Modbus/TCP
}

impl<'a> TryFrom<&'a [u8]> for ModbusTcpPacket<'a> {
    type Error = ModbusTcpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        validate_mbap_min_size(value)?;

        let mut mbaps = Vec::new();
        let mut offset = 0usize;

        while offset < value.len() {
            let slice = &value[offset..];

            // Si on a des octets résiduels trop petits pour un MBAP, c'est une incohérence
            validate_mbap_min_size(slice)?;

            let mbap = MBAP::try_from(slice)?;
            let consumed = 6usize + mbap.length as usize;
            validate_consumed_length(consumed, mbap.length)?;

            mbaps.push(mbap);
            offset += consumed;
        }

        Ok(ModbusTcpPacket { mbaps })
    }
}

#[derive(Debug)]
pub struct MBAP<'a> {
    pub transaction_identifier: u16,
    pub protocol_identifier: u16,
    pub length: u16,
    pub unit_identifier: u8,
    pub pdu: Modbus<'a>,
}

#[derive(Debug)]
pub struct Modbus<'a> {
    pub function_code: u8,
    pub pdu_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for MBAP<'a> {
    type Error = ModbusTcpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        validate_mbap_min_size(value)?;

        let transaction_identifier = u16::from_be_bytes([value[0], value[1]]);
        let protocol_identifier = u16::from_be_bytes([value[2], value[3]]);
        let length = u16::from_be_bytes([value[4], value[5]]);
        let unit_identifier = value[6];

        validate_protocol_identifier(protocol_identifier)?;
        validate_length_field(length)?;

        // Taille totale attendue = 6 + length
        let expected_total = validate_declared_total_length(value, length)?;

        // PDU = après unit id
        let pdu = &value[7..expected_total];
        validate_pdu_not_empty(pdu)?;

        let function_code = pdu[0];
        let pdu_data = &pdu[1..];

        let modbus_data = Modbus {
            function_code,
            pdu_data,
        };

        Ok(MBAP {
            transaction_identifier,
            protocol_identifier,
            length,
            unit_identifier,
            pdu: modbus_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frame_2_request_read_input_registers() {
        // Wireshark Frame 2:
        // TI=0, PI=0, Length=6, Unit=255
        // Function=4, Reference=2258 (0x08D2), WordCount=2
        let bytes: [u8; 12] = [
            0x00, 0x00, // TI : 0
            0x00, 0x00, // PI : 0
            0x00, 0x06, // Length : 6
            0xFF, // Unit : 255
            0x04, // Function : 4
            0x08, 0xD2, // Reference : 2258
            0x00, 0x02, // Word count : 2
        ];

        let pkt = MBAP::try_from(bytes.as_slice()).unwrap();
        assert_eq!(pkt.transaction_identifier, 0);
        assert_eq!(pkt.protocol_identifier, 0);
        assert_eq!(pkt.length, 6);
        assert_eq!(pkt.unit_identifier, 0xFF);
        assert_eq!(pkt.pdu.function_code, 0x04);

        // let pdu = pkt.modbus().unwrap();
        // assert_eq!(pdu.function_code, 0x04);
        // assert_eq!(pdu.pdu_data, &[0x08, 0xD2, 0x00, 0x02]);
    }

    #[test]
    fn reject_non_zero_protocol_id() {
        let bytes: [u8; 7] = [
            0x00, 0x01, // TI
            0x00, 0x02, // PI != 0
            0x00, 0x01, // Length
            0x01, // Unit
        ];
        let err = MBAP::try_from(bytes.as_slice()).unwrap_err();
        assert_eq!(err, ModbusTcpError::InvalidProtocolIdentifier { got: 2 });
    }

    #[test]
    fn reject_truncated_buffer() {
        // length=6 => total attendu = 12, mais on donne moins
        let bytes: [u8; 10] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0xFF, 0x04, 0x00, 0x00];
        let err = MBAP::try_from(bytes.as_slice()).unwrap_err();
        assert_eq!(
            err,
            ModbusTcpError::LengthMismatch {
                expected: 12,
                actual: 10
            }
        );
    }

    #[test]
    fn parse_large_response_header_only_shape() {
        // Frame 3 first ADU: TI=31998 (0x7CFE), PI=0, Length=201 (0x00C9), Unit=255
        // PDU: function=4, byte_count=198, +198 bytes data => PDU len=200 => length=201 OK.
        let mut bytes = vec![
            0x7C, 0xFE, // TI
            0x00, 0x00, // PI
            0x00, 0xC9, // length=201
            0xFF, // unit
            0x04, // function
            0xC6, // byte_count=198
        ];
        bytes.extend(core::iter::repeat_n(0u8, 198));

        let pkt = MBAP::try_from(bytes.as_slice()).unwrap();
        assert_eq!(pkt.transaction_identifier, 0x7CFE);
        assert_eq!(pkt.length, 201);

        // assert_eq!(pkt.modbus_data.pdu_data.len(), 200); // PDU len = length-1
        // assert_eq!(pkt.modbus_data.pdu_data[0], 0x04);
        // assert_eq!(pkt.modbus_data.pdu_data[1], 0xC6);
    }

    #[test]
    fn parse_request() {
        // A MODBUS Request is the message sent on the network by the Client to initiate a transaction,
    }

    #[test]
    fn parse_response() {
        // A MODBUS Response is the Response message sent by the Server,
    }

    #[test]
    fn parse_configuration() {
        // A MODBUS Confirmation is the Response Message received on the Client side
    }

    #[test]
    fn parse_indication() {
        // is the Request message received on the Server side,
    }

    #[test]
    fn parse_multi_adu_is_ok() {
        let hex_str = "02b4000000c9ff04c600000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000004000000000000000000000000000001db000001d600004a380000000000000000000000000000000000000000000000000000000000006461696d006e000000000000000000000000000030310030000000000000000000000000000000000000000000000000000000000000000000000000000002b500000007ff04040004000002b60000002fff042c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002b700000007ff0404a00045a3";

        // Convert hex string to bytes
        let bytes = hex::decode(hex_str).expect("Failed to decode hex string");

        // Try to parse as S7Comm packet
        let result = ModbusTcpPacket::try_from(&bytes[..]);

        // Check if parsing succeeded
        assert!(result.is_ok());
        // Check if the result contain 4 MBAPs
        let pkt = result.unwrap();
        assert_eq!(pkt.mbaps.len(), 4);
    }
}
