// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::fmt;
use std::convert::TryFrom;

use crate::{
    checks::application::mqtt::{
        decode_remaining_length, parse_packet_type, validate_fixed_header_flags,
        validate_mqtt_header_available, validate_mqtt_min_length,
        validate_remaining_length_available, variable_header_len,
    },
    errors::application::mqtt::MqttError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// MQTT Control Packet
///
/// ```mermaid
/// ---
/// title: MqttPacket
/// ---
/// packet-beta
/// 0-3: "Packet Type u4"
/// 4-7: "Fixed Header Flags u4"
/// 8-39: "Remaining Length varint"
/// 40-103: "Variable Header variable"
/// 104-167: "Payload variable"
/// ```
#[derive(Debug)]
pub struct MqttPacket<'a> {
    pub fixed_header: MqttFixedHeader,
    pub variable_header: &'a [u8],
    pub payload: &'a [u8],
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

impl fmt::Display for MqttPacket<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MQTT Packet: fixed_header={}, variable_header={:02X?}, payload={:02X?}",
            self.fixed_header, self.variable_header, self.payload
        )
    }
}

impl<'a> TryFrom<&'a [u8]> for MqttPacket<'a> {
    type Error = MqttError;

    /// Parse le premier paquet MQTT du buffer, en exigeant que le buffer
    /// entier soit une suite de paquets MQTT valides : un segment TCP peut
    /// coalescer plusieurs messages, mais des octets résiduels qui ne se
    /// parsent pas signifient que ce n'était pas du MQTT (garde anti-faux
    /// positifs du probing à l'aveugle).
    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        let (first, mut consumed) = parse_one(packet)?;
        while consumed < packet.len() {
            let (_, len) = parse_one(&packet[consumed..])?;
            consumed += len;
        }
        Ok(first)
    }
}

/// Parse un paquet MQTT au début de `packet` et retourne sa longueur totale
/// (fixed header compris). Les octets au-delà ne sont pas examinés.
fn parse_one(packet: &[u8]) -> Result<(MqttPacket<'_>, usize), MqttError> {
    validate_mqtt_min_length(packet)?;

    let first = packet[0];
    let packet_type = parse_packet_type(first)?;
    validate_fixed_header_flags(packet_type, first)?;

    let (remaining_length, rl_bytes) = decode_remaining_length(&packet[1..])?;
    let header_len = 1 + rl_bytes;

    validate_mqtt_header_available(packet.len(), header_len)?;

    let available = packet.len() - header_len;
    validate_remaining_length_available(remaining_length, available)?;

    let body = &packet[header_len..header_len + remaining_length as usize];
    let vh_len = variable_header_len(packet_type, first, body)?;
    let (vh, pl) = body.split_at(vh_len);

    Ok((
        MqttPacket {
            fixed_header: MqttFixedHeader {
                packet_type,
                remaining_length,
            },
            variable_header: vh,
            payload: pl,
        },
        header_len + remaining_length as usize,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_connect_packet() {
        // CONNECT, remaining length 14 : vh de 10 octets ("MQTT" v4) + client id
        let packet: &[u8] = &[
            0x10, 0x0E, // fixed header
            0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x00, 0x3C, // variable header
            0x00, 0x02, b'i', b'd', // payload : client id
        ];

        let mqtt = MqttPacket::try_from(packet).expect("CONNECT valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Connect);
        assert_eq!(mqtt.fixed_header.remaining_length, 14);
        assert_eq!(mqtt.variable_header.len(), 10);
        // Zero-copy : les slices doivent pointer dans le paquet original.
        assert_eq!(mqtt.variable_header, &packet[2..12]);
        assert_eq!(mqtt.payload, &[0x00, 0x02, b'i', b'd'][..]);
    }

    #[test]
    fn test_parse_connack_packet() {
        let packet: &[u8] = &[0x20, 0x02, 0x00, 0x00];
        let mqtt = MqttPacket::try_from(packet).expect("CONNACK valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Connack);
        assert_eq!(mqtt.variable_header, &[0x00, 0x00][..]);
        assert!(mqtt.payload.is_empty());
    }

    #[test]
    fn test_parse_publish_packet_with_flags() {
        // PUBLISH dup=1 qos=1 retain=1, topic "a/b", packet id + payload
        let packet: &[u8] = &[
            0x3B, 0x09, // fixed header
            0x00, 0x03, b'a', b'/', b'b', // topic
            0x00, 0x01, // packet id (QoS 1 : fait partie du variable header)
            0xDE, 0xAD, // payload applicatif
        ];

        let mqtt = MqttPacket::try_from(packet).expect("PUBLISH valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Publish);
        assert_eq!(
            mqtt.variable_header,
            &[0x00, 0x03, b'a', b'/', b'b', 0x00, 0x01][..]
        );
        assert_eq!(mqtt.payload, &[0xDE, 0xAD][..]);
    }

    #[test]
    fn test_publish_qos3_rejected() {
        // QoS 3 (bits 1-2 à 11) : interdit par la spec.
        let packet: &[u8] = &[0x36, 0x05, 0x00, 0x03, b'a', b'/', b'b'];
        assert!(matches!(
            MqttPacket::try_from(packet),
            Err(MqttError::InvalidQos { qos: 3 })
        ));
    }

    #[test]
    fn test_trailing_garbage_rejected() {
        // Un CONNACK valide suivi d'octets qui ne sont pas du MQTT : le
        // buffer entier doit être une suite de paquets MQTT.
        let packet: &[u8] = &[0x20, 0x02, 0x00, 0x00, 0xFF, 0xFE];
        assert!(MqttPacket::try_from(packet).is_err());
    }

    #[test]
    fn test_coalesced_mqtt_packets_accepted() {
        // Deux messages MQTT dans le même buffer (segments TCP coalescés) :
        // le premier est retourné, le second est validé.
        let packet: &[u8] = &[
            0x20, 0x02, 0x00, 0x00, // CONNACK
            0xC0, 0x00, // PINGREQ
        ];
        let mqtt = MqttPacket::try_from(packet).expect("suite MQTT valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Connack);
    }

    #[test]
    fn test_connect_bad_protocol_name_rejected() {
        // Même forme qu'un CONNECT mais nom de protocole "ABCD".
        let packet: &[u8] = &[
            0x10, 0x0E, 0x00, 0x04, b'A', b'B', b'C', b'D', 0x04, 0x02, 0x00, 0x3C, 0x00, 0x02,
            b'i', b'd',
        ];
        assert!(matches!(
            MqttPacket::try_from(packet),
            Err(MqttError::InvalidProtocolName)
        ));
    }

    #[test]
    fn test_puback_with_bogus_remaining_length_rejected() {
        // PUBACK dont le remaining length n'est ni 2 (v3) ni une forme v5
        // valide : c'était la première source de faux positifs.
        let packet: &[u8] = &[0x40, 0x08, 0x4C, 0xA3, 0x03, 0x97, 0x98, 0xA0, 0x38, 0xB0];
        assert!(matches!(
            MqttPacket::try_from(packet),
            Err(MqttError::InvalidReasonCode { .. })
        ));
    }

    #[test]
    fn test_nonempty_disconnect_v3_rejected() {
        // DISCONNECT v3 doit avoir un corps vide ; 0xF1 n'est pas un reason
        // code v5 plausible.
        let packet: &[u8] = &[0xE0, 0x02, 0xF1, 0x91];
        assert!(matches!(
            MqttPacket::try_from(packet),
            Err(MqttError::InvalidReasonCode { .. })
        ));
    }

    #[test]
    fn test_parse_pingreq_packet() {
        let packet: &[u8] = &[0xC0, 0x00];
        let mqtt = MqttPacket::try_from(packet).expect("PINGREQ valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Pingreq);
        assert_eq!(mqtt.fixed_header.remaining_length, 0);
    }

    #[test]
    fn test_parse_subscribe_requires_flags_0010() {
        // SUBSCRIBE avec flags 0b0000 : invalide
        let bad: &[u8] = &[0x80, 0x05, 0x00, 0x01, 0x00, 0x01, b'a'];
        assert!(matches!(
            MqttPacket::try_from(bad),
            Err(MqttError::InvalidHeaderFlags { .. })
        ));

        // SUBSCRIBE avec flags 0b0010 : valide
        let good: &[u8] = &[0x82, 0x06, 0x00, 0x01, 0x00, 0x01, b'a', 0x00];
        assert!(MqttPacket::try_from(good).is_ok());
    }

    #[test]
    fn test_packet_too_short() {
        assert!(matches!(
            MqttPacket::try_from(&[0x10][..]),
            Err(MqttError::PacketTooShort { .. })
        ));
    }

    #[test]
    fn test_invalid_packet_type() {
        assert!(matches!(
            MqttPacket::try_from(&[0x00, 0x00][..]),
            Err(MqttError::InvalidPacketType { raw: 0 })
        ));
        assert!(matches!(
            MqttPacket::try_from(&[0xF0, 0x00][..]),
            Err(MqttError::InvalidPacketType { raw: 15 })
        ));
    }

    #[test]
    fn test_truncated_remaining_length_varint() {
        // Octet de continuation sans octet suivant : varint tronqué.
        assert!(matches!(
            MqttPacket::try_from(&[0x10, 0x80][..]),
            Err(MqttError::MalformedRemainingLength)
        ));
        assert!(matches!(
            MqttPacket::try_from(&[0x10, 0xFF, 0xFF][..]),
            Err(MqttError::MalformedRemainingLength)
        ));
    }

    #[test]
    fn test_remaining_length_overflow() {
        assert!(matches!(
            MqttPacket::try_from(&[0xC0, 0xFF, 0xFF, 0xFF, 0xFF][..]),
            Err(MqttError::RemainingLengthOverflow)
        ));
    }

    #[test]
    fn test_remaining_length_exceeds_buffer() {
        assert!(matches!(
            MqttPacket::try_from(&[0xC0, 0x05][..]),
            Err(MqttError::RemainingLengthExceedsBuffer { .. })
        ));
    }

    #[test]
    fn test_connect_variable_header_too_short() {
        assert!(matches!(
            MqttPacket::try_from(&[0x10, 0x02, 0x00, 0x04][..]),
            Err(MqttError::VariableHeaderTooShort { .. })
        ));
    }

    #[test]
    fn test_publish_invalid_topic_length() {
        // topic_len déclaré 10, seulement 1 octet dispo
        assert!(matches!(
            MqttPacket::try_from(&[0x30, 0x03, 0x00, 0x0A, b'a'][..]),
            Err(MqttError::InvalidTopicLength { .. })
        ));
    }

    #[test]
    fn test_display_packet_types() {
        let cases: &[(MqttPacketType, &str)] = &[
            (MqttPacketType::Connect, "CONNECT"),
            (MqttPacketType::Connack, "CONNACK"),
            (MqttPacketType::Publish, "PUBLISH"),
            (MqttPacketType::Puback, "PUBACK"),
            (MqttPacketType::Pubrec, "PUBREC"),
            (MqttPacketType::Pubrel, "PUBREL"),
            (MqttPacketType::Pubcomp, "PUBCOMP"),
            (MqttPacketType::Subscribe, "SUBSCRIBE"),
            (MqttPacketType::Suback, "SUBACK"),
            (MqttPacketType::Unsubscribe, "UNSUBSCRIBE"),
            (MqttPacketType::Unsuback, "UNSUBACK"),
            (MqttPacketType::Pingreq, "PINGREQ"),
            (MqttPacketType::Pingresp, "PINGRESP"),
            (MqttPacketType::Disconnect, "DISCONNECT"),
        ];
        for (packet_type, expected) in cases {
            assert_eq!(packet_type.to_string(), *expected);
        }
    }

    #[test]
    fn test_display_packet_and_header() {
        let packet: &[u8] = &[0x20, 0x02, 0x00, 0x00];
        let mqtt = MqttPacket::try_from(packet).unwrap();

        assert_eq!(
            mqtt.fixed_header.to_string(),
            "packet_type=CONNACK, remaining_length=2"
        );
        let rendered = mqtt.to_string();
        assert!(rendered.contains("MQTT Packet:"));
        assert!(rendered.contains("CONNACK"));
    }
}
