use core::convert::TryFrom;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Modbus/TCP Protocol Packet
///
/// ```mermaid
/// ---
/// title: ModbusTcpPacket
/// ---
/// packet
/// %% MBAP Header
/// 0-15: "Transaction Identifier u16"
/// 16-31: "Protocol Identifier u16"
/// 32-47: "Length u16"
/// 48-63: "Unit Identifier u8"
///
/// %% PDU
/// 64-64: "Function Code u16"
/// 48-63: "COTP Dest Ref u16"
/// ```

#[derive(Debug)]
pub struct ModbusTcpPacket<'a> {
    pub mbaps: Vec<MBAP<'a>>, // plusieurs MBAP dans un paquet Modbus/TCP
}

impl<'a> TryFrom<&'a [u8]> for ModbusTcpPacket<'a> {
    type Error = ModbusTcpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() < MBAP_MIN_SIZE {
            return Err(ModbusTcpError::BufferTooSmall {
                needed: MBAP_MIN_SIZE,
                actual: value.len(),
            });
        }

        let mut mbaps = Vec::new();
        let mut offset = 0usize;

        while offset < value.len() {
            let slice = &value[offset..];

            // Si on a des octets résiduels trop petits pour un MBAP, c'est une incohérence
            if slice.len() < MBAP_MIN_SIZE {
                return Err(ModbusTcpError::BufferTooSmall {
                    needed: MBAP_MIN_SIZE,
                    actual: slice.len(),
                });
            }

            let mbap = MBAP::try_from(slice)?;
            let consumed = 6usize + mbap.length as usize;

            // Sécurité: ne jamais boucler à l'infini
            if consumed == 0 {
                return Err(ModbusTcpError::InvalidLengthField { got: mbap.length });
            }

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

pub const MBAP_MIN_SIZE: usize = 7;

#[derive(Debug)]
pub struct Modbus<'a> {
    pub function_code: u8,
    pub pdu_data: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModbusTcpError {
    BufferTooSmall { needed: usize, actual: usize },
    InvalidProtocolIdentifier { got: u16 }, // en Modbus/TCP standard, c'est 0
    InvalidLengthField { got: u16 },        // length doit être >= 1 (car inclut unit_id)
    LengthMismatch { expected: usize, actual: usize },
    PduTooSmall { needed: usize, actual: usize },
}

impl<'a> TryFrom<&'a [u8]> for MBAP<'a> {
    type Error = ModbusTcpError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // MBAP minimal = 7 bytes
        if value.len() < MBAP_MIN_SIZE {
            return Err(ModbusTcpError::BufferTooSmall {
                needed: MBAP_MIN_SIZE,
                actual: value.len(),
            });
        }

        let transaction_identifier = u16::from_be_bytes([value[0], value[1]]);
        let protocol_identifier = u16::from_be_bytes([value[2], value[3]]);
        let length = u16::from_be_bytes([value[4], value[5]]);
        let unit_identifier = value[6];

        if protocol_identifier != 0 {
            return Err(ModbusTcpError::InvalidProtocolIdentifier {
                got: protocol_identifier,
            });
        }

        if length < 1 {
            return Err(ModbusTcpError::InvalidLengthField { got: length });
        }

        // Taille totale attendue = 6 + length
        let expected_total = 6usize + length as usize;
        if value.len() < expected_total {
            return Err(ModbusTcpError::LengthMismatch {
                expected: expected_total,
                actual: value.len(),
            });
        }

        // PDU = après unit id
        let pdu = &value[7..expected_total];
        if pdu.is_empty() {
            return Err(ModbusTcpError::PduTooSmall {
                needed: 1,
                actual: pdu.len(),
            });
        }

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
