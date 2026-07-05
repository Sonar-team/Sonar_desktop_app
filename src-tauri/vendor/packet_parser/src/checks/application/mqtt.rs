// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::mqtt::MqttError, parse::application::protocols::mqtt::MqttPacketType,
};

pub const MQTT_MIN_HEADER_LEN: usize = 2;

pub fn validate_mqtt_min_length(packet: &[u8]) -> Result<(), MqttError> {
    if packet.len() < MQTT_MIN_HEADER_LEN {
        return Err(MqttError::PacketTooShort {
            actual: packet.len(),
            min: MQTT_MIN_HEADER_LEN,
        });
    }

    Ok(())
}

pub fn parse_packet_type(first_byte: u8) -> Result<MqttPacketType, MqttError> {
    let nibble = first_byte >> 4;
    match nibble {
        1 => Ok(MqttPacketType::Connect),
        2 => Ok(MqttPacketType::Connack),
        3 => Ok(MqttPacketType::Publish),
        4 => Ok(MqttPacketType::Puback),
        5 => Ok(MqttPacketType::Pubrec),
        6 => Ok(MqttPacketType::Pubrel),
        7 => Ok(MqttPacketType::Pubcomp),
        8 => Ok(MqttPacketType::Subscribe),
        9 => Ok(MqttPacketType::Suback),
        10 => Ok(MqttPacketType::Unsubscribe),
        11 => Ok(MqttPacketType::Unsuback),
        12 => Ok(MqttPacketType::Pingreq),
        13 => Ok(MqttPacketType::Pingresp),
        14 => Ok(MqttPacketType::Disconnect),
        _ => Err(MqttError::InvalidPacketType { raw: nibble }),
    }
}

pub fn validate_fixed_header_flags(
    packet_type: MqttPacketType,
    first_byte: u8,
) -> Result<(), MqttError> {
    let flags = first_byte & 0x0F;
    let type_nibble = first_byte >> 4;

    match packet_type {
        MqttPacketType::Publish => Ok(()),
        MqttPacketType::Pubrel | MqttPacketType::Subscribe | MqttPacketType::Unsubscribe => {
            if flags == 0b0010 {
                Ok(())
            } else {
                Err(MqttError::InvalidHeaderFlags {
                    packet_type: type_nibble,
                    flags,
                })
            }
        }
        _ => {
            if flags == 0 {
                Ok(())
            } else {
                Err(MqttError::InvalidHeaderFlags {
                    packet_type: type_nibble,
                    flags,
                })
            }
        }
    }
}

pub fn decode_remaining_length(buf: &[u8]) -> Result<(u32, usize), MqttError> {
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;

    for (i, &byte) in buf.iter().take(4).enumerate() {
        value = value
            .checked_add(((byte & 127) as u32).saturating_mul(multiplier))
            .ok_or(MqttError::MalformedRemainingLength)?;

        if (byte & 128) == 0 {
            return Ok((value, i + 1));
        }

        multiplier = multiplier
            .checked_mul(128)
            .ok_or(MqttError::MalformedRemainingLength)?;
    }

    Err(MqttError::RemainingLengthOverflow)
}

pub fn validate_mqtt_header_available(
    packet_len: usize,
    header_len: usize,
) -> Result<(), MqttError> {
    if packet_len < header_len {
        return Err(MqttError::PacketTooShort {
            actual: packet_len,
            min: header_len,
        });
    }

    Ok(())
}

pub fn validate_remaining_length_available(
    remaining_length: u32,
    available: usize,
) -> Result<(), MqttError> {
    if available < remaining_length as usize {
        return Err(MqttError::RemainingLengthExceedsBuffer {
            remaining_length,
            available,
        });
    }

    Ok(())
}

pub fn variable_header_len(packet_type: MqttPacketType, body: &[u8]) -> Result<usize, MqttError> {
    match packet_type {
        MqttPacketType::Connect => {
            if body.len() < 10 {
                return Err(MqttError::VariableHeaderTooShort {
                    packet_type,
                    actual: body.len(),
                    min: 10,
                });
            }
            Ok(10)
        }
        MqttPacketType::Connack => {
            if body.len() < 2 {
                return Err(MqttError::VariableHeaderTooShort {
                    packet_type,
                    actual: body.len(),
                    min: 2,
                });
            }
            Ok(2)
        }
        MqttPacketType::Publish => {
            if body.len() < 2 {
                return Err(MqttError::VariableHeaderTooShort {
                    packet_type,
                    actual: body.len(),
                    min: 2,
                });
            }
            let topic_len = u16::from_be_bytes([body[0], body[1]]) as usize;
            let needed = 2 + topic_len;
            if body.len() < needed {
                return Err(MqttError::InvalidTopicLength {
                    declared: topic_len,
                    available: body.len().saturating_sub(2),
                });
            }
            Ok(needed)
        }
        MqttPacketType::Disconnect | MqttPacketType::Pingreq | MqttPacketType::Pingresp => Ok(0),
        _ => Ok(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_packet_type_all_nibbles() {
        let cases: &[(u8, MqttPacketType)] = &[
            (1, MqttPacketType::Connect),
            (2, MqttPacketType::Connack),
            (3, MqttPacketType::Publish),
            (4, MqttPacketType::Puback),
            (5, MqttPacketType::Pubrec),
            (6, MqttPacketType::Pubrel),
            (7, MqttPacketType::Pubcomp),
            (8, MqttPacketType::Subscribe),
            (9, MqttPacketType::Suback),
            (10, MqttPacketType::Unsubscribe),
            (11, MqttPacketType::Unsuback),
            (12, MqttPacketType::Pingreq),
            (13, MqttPacketType::Pingresp),
            (14, MqttPacketType::Disconnect),
        ];

        for (nibble, expected) in cases {
            assert_eq!(parse_packet_type(nibble << 4).unwrap(), *expected);
        }

        assert!(matches!(
            parse_packet_type(0x00),
            Err(MqttError::InvalidPacketType { raw: 0 })
        ));
        assert!(matches!(
            parse_packet_type(0xF0),
            Err(MqttError::InvalidPacketType { raw: 15 })
        ));
    }

    #[test]
    fn test_fixed_header_flags_rules() {
        // PUBLISH accepte tous les flags
        assert!(validate_fixed_header_flags(MqttPacketType::Publish, 0x3F).is_ok());

        // PUBREL / SUBSCRIBE / UNSUBSCRIBE exigent 0b0010
        for packet_type in [
            MqttPacketType::Pubrel,
            MqttPacketType::Subscribe,
            MqttPacketType::Unsubscribe,
        ] {
            assert!(validate_fixed_header_flags(packet_type, 0x62).is_ok());
            assert!(matches!(
                validate_fixed_header_flags(packet_type, 0x60),
                Err(MqttError::InvalidHeaderFlags { .. })
            ));
        }

        // Les autres exigent 0
        assert!(validate_fixed_header_flags(MqttPacketType::Connect, 0x10).is_ok());
        assert!(matches!(
            validate_fixed_header_flags(MqttPacketType::Connect, 0x11),
            Err(MqttError::InvalidHeaderFlags { .. })
        ));
    }

    #[test]
    fn test_decode_remaining_length_multi_byte() {
        // 321 = 0xC1 0x02 en varint MQTT
        assert_eq!(decode_remaining_length(&[0xC1, 0x02]).unwrap(), (321, 2));
        // continuation sans fin sur 4 octets
        assert!(matches!(
            decode_remaining_length(&[0x80, 0x80, 0x80, 0x80]),
            Err(MqttError::RemainingLengthOverflow)
        ));
    }

    #[test]
    fn test_variable_header_len_rules() {
        // CONNACK : 2 octets requis
        assert_eq!(
            variable_header_len(MqttPacketType::Connack, &[0, 0]).unwrap(),
            2
        );
        assert!(matches!(
            variable_header_len(MqttPacketType::Connack, &[0]),
            Err(MqttError::VariableHeaderTooShort { .. })
        ));

        // PUBLISH : trop court
        assert!(matches!(
            variable_header_len(MqttPacketType::Publish, &[0]),
            Err(MqttError::VariableHeaderTooShort { .. })
        ));

        // Types sans variable header
        assert_eq!(
            variable_header_len(MqttPacketType::Disconnect, &[]).unwrap(),
            0
        );
        assert_eq!(
            variable_header_len(MqttPacketType::Pingresp, &[]).unwrap(),
            0
        );
        assert_eq!(variable_header_len(MqttPacketType::Puback, &[0, 0]).unwrap(), 0);
    }

    #[test]
    fn test_header_available() {
        assert!(validate_mqtt_header_available(10, 5).is_ok());
        assert!(matches!(
            validate_mqtt_header_available(3, 5),
            Err(MqttError::PacketTooShort { actual: 3, min: 5 })
        ));
    }
}
