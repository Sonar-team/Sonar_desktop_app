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
