use core::fmt;
use std::convert::TryFrom;

use crate::errors::application::mqtt::MqttError;

#[derive(Debug)]
pub struct MqttPacket {
    pub fixed_header: MqttFixedHeader,
    pub variable_header: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MqttFixedHeader {
    pub packet_type: MqttPacketType,
    pub remaining_length: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MqttPacketType {
    Connect = 1,
    Connack,
    Publish,
    Puback,
    Pubrec,
    Pubrel,
    Pubcomp,
    Subscribe,
    Suback,
    Unsubscribe,
    Unsuback,
    Pingreq,
    Pingresp,
    Disconnect,
}

impl fmt::Display for MqttPacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MqttPacketType::Connect => "CONNECT",
            MqttPacketType::Connack => "CONNACK",
            MqttPacketType::Publish => "PUBLISH",
            MqttPacketType::Puback => "PUBACK",
            MqttPacketType::Pubrec => "PUBREC",
            MqttPacketType::Pubrel => "PUBREL",
            MqttPacketType::Pubcomp => "PUBCOMP",
            MqttPacketType::Subscribe => "SUBSCRIBE",
            MqttPacketType::Suback => "SUBACK",
            MqttPacketType::Unsubscribe => "UNSUBSCRIBE",
            MqttPacketType::Unsuback => "UNSUBACK",
            MqttPacketType::Pingreq => "PINGREQ",
            MqttPacketType::Pingresp => "PINGRESP",
            MqttPacketType::Disconnect => "DISCONNECT",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for MqttFixedHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "packet_type={}, remaining_length={}",
            self.packet_type, self.remaining_length
        )
    }
}

impl fmt::Display for MqttPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MQTT Packet: fixed_header={}, variable_header={:02X?}, payload={:02X?}",
            self.fixed_header, self.variable_header, self.payload
        )
    }
}

fn parse_packet_type(first_byte: u8) -> Result<MqttPacketType, MqttError> {
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

fn validate_fixed_header_flags(
    packet_type: MqttPacketType,
    first_byte: u8,
) -> Result<(), MqttError> {
    let flags = first_byte & 0x0F;
    let type_nibble = first_byte >> 4;

    match packet_type {
        MqttPacketType::Publish => {
            // PUBLISH flags are used (DUP/QoS/RETAIN). Here we accept any 4-bit value.
            Ok(())
        }
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

fn decode_remaining_length(buf: &[u8]) -> Result<(u32, usize), MqttError> {
    // buf starts at byte 1 (after first fixed header byte)
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;

    for (i, &byte) in buf.iter().take(4).enumerate() {
        value = value
            .checked_add(((byte & 127) as u32).saturating_mul(multiplier))
            .ok_or(MqttError::MalformedRemainingLength)?;

        if (byte & 128) == 0 {
            // bytes consumed = i + 1
            return Ok((value, i + 1));
        }

        multiplier = multiplier
            .checked_mul(128)
            .ok_or(MqttError::MalformedRemainingLength)?;
    }

    Err(MqttError::RemainingLengthOverflow)
}

fn variable_header_len(packet_type: MqttPacketType, body: &[u8]) -> Result<usize, MqttError> {
    match packet_type {
        MqttPacketType::Connect => {
            // Protocol Name + Level + Flags + KeepAlive = 10 bytes MIN for your current model.
            // Strictly, CONNECT variable header is:
            // 2 + "MQTT"(4) + 1 + 1 + 2 = 10
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
            // topic length (2) + topic + (optional packet identifier if QoS>0)
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
            // NOTE: strict QoS handling (Packet Identifier) can be added later.
            Ok(needed)
        }
        MqttPacketType::Disconnect | MqttPacketType::Pingreq | MqttPacketType::Pingresp => Ok(0),
        // Pour le reste, tu peux soit retourner 0 (parser minimal), soit "unsupported" en strict.
        _ => Ok(0),
    }
}

impl TryFrom<&[u8]> for MqttPacket {
    type Error = MqttError;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
        if packet.len() < 2 {
            return Err(MqttError::PacketTooShort {
                actual: packet.len(),
                min: 2,
            });
        }

        let first = packet[0];
        let packet_type = parse_packet_type(first)?;
        validate_fixed_header_flags(packet_type, first)?;

        let (remaining_length, rl_bytes) = decode_remaining_length(&packet[1..])?;
        let header_len = 1 + rl_bytes;

        if packet.len() < header_len {
            return Err(MqttError::PacketTooShort {
                actual: packet.len(),
                min: header_len,
            });
        }

        let available = packet.len() - header_len;
        if available < remaining_length as usize {
            return Err(MqttError::RemainingLengthExceedsBuffer {
                remaining_length,
                available,
            });
        }

        let body = &packet[header_len..header_len + remaining_length as usize];
        let vh_len = variable_header_len(packet_type, body)?;
        let (vh, pl) = body.split_at(vh_len);

        Ok(MqttPacket {
            fixed_header: MqttFixedHeader {
                packet_type,
                remaining_length,
            },
            variable_header: vh.to_vec(),
            payload: pl.to_vec(),
        })
    }
}
