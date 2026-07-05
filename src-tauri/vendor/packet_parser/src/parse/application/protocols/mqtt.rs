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

impl TryFrom<&[u8]> for MqttPacket {
    type Error = MqttError;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
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
        assert_eq!(mqtt.payload, vec![0x00, 0x02, b'i', b'd']);
    }

    #[test]
    fn test_parse_connack_packet() {
        let packet: &[u8] = &[0x20, 0x02, 0x00, 0x00];
        let mqtt = MqttPacket::try_from(packet).expect("CONNACK valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Connack);
        assert_eq!(mqtt.variable_header, vec![0x00, 0x00]);
        assert!(mqtt.payload.is_empty());
    }

    #[test]
    fn test_parse_publish_packet_with_flags() {
        // PUBLISH dup=1 qos=1 retain=1, topic "a/b", packet id + payload
        let packet: &[u8] = &[
            0x3B, 0x09, // fixed header
            0x00, 0x03, b'a', b'/', b'b', // topic
            0x00, 0x01, // packet id (compté dans le payload ici)
            0xDE, 0xAD, // payload applicatif
        ];

        let mqtt = MqttPacket::try_from(packet).expect("PUBLISH valide");
        assert_eq!(mqtt.fixed_header.packet_type, MqttPacketType::Publish);
        assert_eq!(mqtt.variable_header, vec![0x00, 0x03, b'a', b'/', b'b']);
        assert_eq!(mqtt.payload, vec![0x00, 0x01, 0xDE, 0xAD]);
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
