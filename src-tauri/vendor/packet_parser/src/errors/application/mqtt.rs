use core::fmt;

use crate::parse::application::protocols::mqtt::MqttPacketType;

#[derive(Debug, PartialEq, Eq)]
pub enum MqttError {
    PacketTooShort {
        actual: usize,
        min: usize,
    },

    InvalidPacketType {
        raw: u8,
    },

    MalformedRemainingLength, // encodage RL invalide
    RemainingLengthOverflow,  // > 4 bytes
    RemainingLengthExceedsBuffer {
        remaining_length: u32,
        available: usize,
    },

    InvalidHeaderFlags {
        packet_type: u8,
        flags: u8,
    }, // strict MQTT flags

    VariableHeaderTooShort {
        packet_type: MqttPacketType,
        actual: usize,
        min: usize,
    },
    InvalidTopicLength {
        declared: usize,
        available: usize,
    },

    UnsupportedPacketType {
        packet_type: MqttPacketType,
    }, // si tu veux être strict et limiter
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::PacketTooShort { actual, min } => {
                write!(f, "MQTT packet too short: {actual} bytes (min {min})")
            }
            MqttError::InvalidPacketType { raw } => {
                write!(f, "Invalid MQTT packet type nibble: {raw}")
            }
            MqttError::MalformedRemainingLength => write!(f, "Malformed MQTT remaining length"),
            MqttError::RemainingLengthOverflow => {
                write!(f, "MQTT remaining length overflow (>4 bytes)")
            }
            MqttError::RemainingLengthExceedsBuffer {
                remaining_length,
                available,
            } => {
                write!(
                    f,
                    "MQTT remaining length {remaining_length} exceeds available {available}"
                )
            }
            MqttError::InvalidHeaderFlags { packet_type, flags } => {
                write!(
                    f,
                    "Invalid MQTT fixed header flags: type={packet_type}, flags={flags:01X}"
                )
            }
            MqttError::VariableHeaderTooShort {
                packet_type,
                actual,
                min,
            } => {
                write!(
                    f,
                    "MQTT {:?} variable header too short: {actual} (min {min})",
                    packet_type
                )
            }
            MqttError::InvalidTopicLength {
                declared,
                available,
            } => {
                write!(
                    f,
                    "MQTT topic length {declared} exceeds available {available}"
                )
            }
            MqttError::UnsupportedPacketType { packet_type } => {
                write!(f, "Unsupported MQTT packet type: {:?}", packet_type)
            }
        }
    }
}

// Optionnel si tu utilises thiserror : derive(Error) + #[error(...)].
