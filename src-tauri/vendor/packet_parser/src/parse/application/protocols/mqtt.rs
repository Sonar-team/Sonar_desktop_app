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
