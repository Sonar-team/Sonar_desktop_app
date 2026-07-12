// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

use crate::parse::application::protocols::mqtt::MqttPacketType;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MqttError {
    #[error("MQTT packet too short: {actual} bytes (min {min})")]
    PacketTooShort { actual: usize, min: usize },

    #[error("Invalid MQTT packet type nibble: {raw}")]
    InvalidPacketType { raw: u8 },

    /// Encodage varint invalide ou tronqué.
    #[error("Malformed MQTT remaining length")]
    MalformedRemainingLength,

    /// Varint de plus de 4 octets.
    #[error("MQTT remaining length overflow (>4 bytes)")]
    RemainingLengthOverflow,

    #[error("MQTT remaining length {remaining_length} exceeds available {available}")]
    RemainingLengthExceedsBuffer {
        remaining_length: u32,
        available: usize,
    },

    /// Flags stricts du fixed header MQTT.
    #[error("Invalid MQTT fixed header flags: type={packet_type}, flags={flags:01X}")]
    InvalidHeaderFlags { packet_type: u8, flags: u8 },

    #[error("MQTT {packet_type:?} variable header too short: {actual} (min {min})")]
    VariableHeaderTooShort {
        packet_type: MqttPacketType,
        actual: usize,
        min: usize,
    },

    #[error("MQTT topic length {declared} exceeds available {available}")]
    InvalidTopicLength { declared: usize, available: usize },

    #[error("Unsupported MQTT packet type: {packet_type:?}")]
    UnsupportedPacketType { packet_type: MqttPacketType },

    /// CONNECT dont le nom de protocole n'est ni "MQTT" ni "MQIsdp".
    #[error("Invalid MQTT protocol name in CONNECT")]
    InvalidProtocolName,

    /// Niveau de protocole incohérent avec le nom (MQIsdp=3, MQTT=4/5).
    #[error("Invalid MQTT protocol level: {level}")]
    InvalidProtocolLevel { level: u8 },

    /// Bit réservé des connect flags à 1 (doit être 0).
    #[error("Reserved MQTT CONNECT flag bit set")]
    InvalidReservedConnectFlag,

    /// QoS 3 (les deux bits QoS à 1) : interdit par la spec.
    #[error("Invalid MQTT QoS: {qos}")]
    InvalidQos { qos: u8 },

    /// Packet identifier nul là où la spec impose une valeur non nulle.
    #[error("MQTT packet identifier must be non-zero")]
    ZeroPacketId,

    /// Remaining length impossible pour ce type (ex. PINGREQ != 0, PUBACK < 2).
    #[error("Invalid MQTT remaining length {remaining_length} for {packet_type:?}")]
    InvalidRemainingLength {
        packet_type: MqttPacketType,
        remaining_length: u32,
    },

    /// Code retour / reason code inconnu pour ce type de paquet.
    #[error("Invalid MQTT reason code {code:#04X} for {packet_type:?}")]
    InvalidReasonCode {
        packet_type: MqttPacketType,
        code: u8,
    },

    /// Topic non UTF-8, vide, avec caractère de contrôle, ou wildcard dans un
    /// PUBLISH.
    #[error("Invalid MQTT topic")]
    InvalidTopic,

    /// Payload de SUBSCRIBE/UNSUBSCRIBE qui ne se découpe pas en une suite
    /// exacte d'entrées (topic filter [+ QoS]).
    #[error("Malformed MQTT {packet_type:?} payload")]
    MalformedSubscriptionPayload { packet_type: MqttPacketType },
}
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
