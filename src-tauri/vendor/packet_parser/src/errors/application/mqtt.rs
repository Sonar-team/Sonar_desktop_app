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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::application::protocols::mqtt::MqttPacketType;

    #[test]
    fn test_packet_too_short_display() {
        let err = MqttError::PacketTooShort { actual: 1, min: 2 };

        assert_eq!(err.to_string(), "MQTT packet too short: 1 bytes (min 2)");
    }

    #[test]
    fn test_invalid_packet_type_display() {
        let err = MqttError::InvalidPacketType { raw: 15 };

        assert_eq!(err.to_string(), "Invalid MQTT packet type nibble: 15");
    }

    #[test]
    fn test_malformed_remaining_length_display() {
        let err = MqttError::MalformedRemainingLength;

        assert_eq!(err.to_string(), "Malformed MQTT remaining length");
    }

    #[test]
    fn test_remaining_length_overflow_display() {
        let err = MqttError::RemainingLengthOverflow;

        assert_eq!(err.to_string(), "MQTT remaining length overflow (>4 bytes)");
    }

    #[test]
    fn test_remaining_length_exceeds_buffer_display() {
        let err = MqttError::RemainingLengthExceedsBuffer {
            remaining_length: 128,
            available: 42,
        };

        assert_eq!(
            err.to_string(),
            "MQTT remaining length 128 exceeds available 42"
        );
    }

    #[test]
    fn test_invalid_header_flags_display() {
        let err = MqttError::InvalidHeaderFlags {
            packet_type: 3,
            flags: 0x0F,
        };

        assert_eq!(
            err.to_string(),
            "Invalid MQTT fixed header flags: type=3, flags=F"
        );
    }

    #[test]
    fn test_variable_header_too_short_display() {
        let err = MqttError::VariableHeaderTooShort {
            packet_type: MqttPacketType::Connect,
            actual: 4,
            min: 10,
        };

        assert_eq!(
            err.to_string(),
            "MQTT Connect variable header too short: 4 (min 10)"
        );
    }

    #[test]
    fn test_invalid_topic_length_display() {
        let err = MqttError::InvalidTopicLength {
            declared: 20,
            available: 8,
        };

        assert_eq!(err.to_string(), "MQTT topic length 20 exceeds available 8");
    }

    #[test]
    fn test_unsupported_packet_type_display() {
        let err = MqttError::UnsupportedPacketType {
            packet_type: MqttPacketType::Publish,
        };

        assert_eq!(err.to_string(), "Unsupported MQTT packet type: Publish");
    }

    #[test]
    fn test_packet_too_short_equality() {
        let left = MqttError::PacketTooShort { actual: 1, min: 2 };
        let right = MqttError::PacketTooShort { actual: 1, min: 2 };

        assert_eq!(left, right);
    }

    #[test]
    fn test_invalid_header_flags_equality() {
        let left = MqttError::InvalidHeaderFlags {
            packet_type: 3,
            flags: 0x02,
        };
        let right = MqttError::InvalidHeaderFlags {
            packet_type: 3,
            flags: 0x02,
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_invalid_topic_length_equality() {
        let left = MqttError::InvalidTopicLength {
            declared: 12,
            available: 4,
        };
        let right = MqttError::InvalidTopicLength {
            declared: 12,
            available: 4,
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_debug_contains_variant_name() {
        let err = MqttError::MalformedRemainingLength;
        let dbg = format!("{err:?}");

        assert!(dbg.contains("MalformedRemainingLength"));
    }
}
