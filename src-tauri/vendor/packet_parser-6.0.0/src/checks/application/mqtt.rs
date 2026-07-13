// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::mqtt::MqttError, parse::application::protocols::mqtt::MqttPacketType,
};

pub const MQTT_MIN_HEADER_LEN: usize = 2;
pub const MQTT_REMAINING_LENGTH_MAX_BYTES: usize = 4;

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
        MqttPacketType::Publish => {
            let qos = (flags >> 1) & 0b11;
            if qos == 3 {
                return Err(MqttError::InvalidQos { qos });
            }
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

pub fn decode_remaining_length(buf: &[u8]) -> Result<(u32, usize), MqttError> {
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;

    for (i, &byte) in buf.iter().take(MQTT_REMAINING_LENGTH_MAX_BYTES).enumerate() {
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

    if buf.len() < MQTT_REMAINING_LENGTH_MAX_BYTES {
        // Le buffer se termine sur un octet de continuation : varint tronqué.
        Err(MqttError::MalformedRemainingLength)
    } else {
        Err(MqttError::RemainingLengthOverflow)
    }
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

/// Reason codes v5 plausibles pour PUBACK/PUBREC/PUBREL/PUBCOMP (union des
/// quatre types, suffisant pour départager du bruit binaire).
const ACK_REASON_CODES: &[u8] = &[0x00, 0x10, 0x80, 0x83, 0x87, 0x90, 0x91, 0x92, 0x97, 0x99];

/// Codes de retour SUBACK : granted QoS 0-2 et échec (v3.1.1), plus les
/// reason codes v5.
const SUBACK_REASON_CODES: &[u8] = &[
    0x00, 0x01, 0x02, 0x80, 0x83, 0x87, 0x8F, 0x91, 0x97, 0x9E, 0xA1, 0xA2,
];

/// Reason codes UNSUBACK v5.
const UNSUBACK_REASON_CODES: &[u8] = &[0x00, 0x11, 0x80, 0x83, 0x87, 0x8F, 0x91];

fn is_valid_disconnect_reason(code: u8) -> bool {
    code == 0x00 || code == 0x04 || (0x80..=0xA2).contains(&code)
}

fn is_valid_connack_code(code: u8) -> bool {
    // v3.1.1 : 0 (accepté) à 5 (non autorisé) ; v5 : 0 ou reason code >= 0x80.
    code <= 5 || (0x80..=0xA2).contains(&code)
}

/// Vérifie qu'un bloc de propriétés MQTT v5 remplit exactement `buf` :
/// varint de longueur + contenu, sans reste.
fn properties_fill_exactly(buf: &[u8]) -> bool {
    match decode_remaining_length(buf) {
        Ok((props_len, varint_bytes)) => varint_bytes + props_len as usize == buf.len(),
        Err(_) => false,
    }
}

/// Topic name d'un PUBLISH : UTF-8 valide, non vide, sans caractère de
/// contrôle ni wildcard ('#' et '+' sont interdits à la publication).
fn validate_publish_topic(topic: &[u8]) -> Result<(), MqttError> {
    if topic.is_empty() {
        return Err(MqttError::InvalidTopic);
    }
    let s = std::str::from_utf8(topic).map_err(|_| MqttError::InvalidTopic)?;
    if s.chars()
        .any(|c| c.is_control() || c == '#' || c == '+' || c == '\u{0}')
    {
        return Err(MqttError::InvalidTopic);
    }
    Ok(())
}

/// Topic filter d'un SUBSCRIBE/UNSUBSCRIBE : UTF-8 valide, non vide, sans
/// caractère de contrôle (les wildcards sont autorisés ici).
fn validate_topic_filter(topic: &[u8]) -> bool {
    if topic.is_empty() {
        return false;
    }
    match std::str::from_utf8(topic) {
        Ok(s) => !s.chars().any(|c| c.is_control()),
        Err(_) => false,
    }
}

fn read_packet_id(packet_type: MqttPacketType, body: &[u8]) -> Result<u16, MqttError> {
    if body.len() < 2 {
        return Err(MqttError::VariableHeaderTooShort {
            packet_type,
            actual: body.len(),
            min: 2,
        });
    }
    let pid = u16::from_be_bytes([body[0], body[1]]);
    if pid == 0 {
        return Err(MqttError::ZeroPacketId);
    }
    Ok(pid)
}

/// Itère les entrées d'un SUBSCRIBE (`with_qos`) ou UNSUBSCRIBE à partir de
/// `start` : chaque entrée est `len u16 + topic filter [+ QoS <= 2]`, et la
/// suite d'entrées doit consommer `body` exactement.
fn subscription_entries_fill_exactly(body: &[u8], start: usize, with_qos: bool) -> bool {
    let mut off = start;
    let mut entries = 0usize;
    while off < body.len() {
        if body.len() - off < 2 {
            return false;
        }
        let topic_len = u16::from_be_bytes([body[off], body[off + 1]]) as usize;
        off += 2;
        if body.len() - off < topic_len || !validate_topic_filter(&body[off..off + topic_len]) {
            return false;
        }
        off += topic_len;
        if with_qos {
            // Options de souscription : QoS 0-2 (v3.1.1) ou byte d'options v5
            // dont les bits 6-7 sont réservés à 0.
            if off >= body.len() || body[off] & 0xC0 != 0 || body[off] & 0b11 == 3 {
                return false;
            }
            off += 1;
        }
        entries += 1;
    }
    entries > 0 && off == body.len()
}

/// Valide le corps (`remaining length` octets) selon le type de paquet et
/// retourne la longueur du variable header.
///
/// Les règles sont celles de MQTT 3.1/3.1.1, avec une tolérance pour les
/// formes v5 (reason codes + bloc de propriétés) — l'objectif est de
/// discriminer du vrai MQTT face à du bruit binaire, pas de valider
/// exhaustivement la spec.
pub fn variable_header_len(
    packet_type: MqttPacketType,
    first_byte: u8,
    body: &[u8],
) -> Result<usize, MqttError> {
    let remaining_length = body.len() as u32;
    let invalid_len = || MqttError::InvalidRemainingLength {
        packet_type,
        remaining_length,
    };

    match packet_type {
        MqttPacketType::Connect => {
            if body.len() < 2 {
                return Err(MqttError::VariableHeaderTooShort {
                    packet_type,
                    actual: body.len(),
                    min: 2,
                });
            }
            let name_len = u16::from_be_bytes([body[0], body[1]]) as usize;
            // name + level (1) + connect flags (1) + keep alive (2)
            let vh_len = 2 + name_len + 4;
            if body.len() < vh_len {
                return Err(MqttError::VariableHeaderTooShort {
                    packet_type,
                    actual: body.len(),
                    min: vh_len,
                });
            }
            let name = &body[2..2 + name_len];
            let level = body[2 + name_len];
            match (name, level) {
                (b"MQTT", 4 | 5) | (b"MQIsdp", 3) => {}
                (b"MQTT", _) | (b"MQIsdp", _) => {
                    return Err(MqttError::InvalidProtocolLevel { level });
                }
                _ => return Err(MqttError::InvalidProtocolName),
            }
            let connect_flags = body[2 + name_len + 1];
            if connect_flags & 0x01 != 0 {
                return Err(MqttError::InvalidReservedConnectFlag);
            }
            Ok(vh_len)
        }
        MqttPacketType::Connack => {
            if body.len() < 2 {
                return Err(invalid_len());
            }
            if body[0] > 1 {
                // Connect acknowledge flags : seuls bits 0 (session present).
                return Err(MqttError::InvalidHeaderFlags {
                    packet_type: first_byte >> 4,
                    flags: body[0],
                });
            }
            if !is_valid_connack_code(body[1]) {
                return Err(MqttError::InvalidReasonCode {
                    packet_type,
                    code: body[1],
                });
            }
            // v3 : exactement 2 octets ; v5 : + bloc de propriétés exact.
            if body.len() > 2 && !properties_fill_exactly(&body[2..]) {
                return Err(invalid_len());
            }
            Ok(body.len())
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
            let mut vh_len = 2 + topic_len;
            if body.len() < vh_len {
                return Err(MqttError::InvalidTopicLength {
                    declared: topic_len,
                    available: body.len().saturating_sub(2),
                });
            }
            validate_publish_topic(&body[2..2 + topic_len])?;
            let qos = (first_byte >> 1) & 0b11;
            if qos > 0 {
                read_packet_id(packet_type, &body[vh_len..])?;
                vh_len += 2;
            }
            Ok(vh_len)
        }
        MqttPacketType::Puback
        | MqttPacketType::Pubrec
        | MqttPacketType::Pubrel
        | MqttPacketType::Pubcomp => {
            read_packet_id(packet_type, body)?;
            match body.len() {
                // v3.1.1 : packet id seul.
                2 => {}
                // v5 : + reason code, puis éventuel bloc de propriétés exact.
                3.. => {
                    if !ACK_REASON_CODES.contains(&body[2]) {
                        return Err(MqttError::InvalidReasonCode {
                            packet_type,
                            code: body[2],
                        });
                    }
                    if body.len() > 3 && !properties_fill_exactly(&body[3..]) {
                        return Err(invalid_len());
                    }
                }
                _ => return Err(invalid_len()),
            }
            Ok(body.len())
        }
        MqttPacketType::Subscribe | MqttPacketType::Unsubscribe => {
            read_packet_id(packet_type, body)?;
            let with_qos = packet_type == MqttPacketType::Subscribe;
            // v3 : les entrées commencent après le packet id ; v5 : après un
            // bloc de propriétés (toléré uniquement vide : 0x00).
            let v3 = subscription_entries_fill_exactly(body, 2, with_qos);
            let v5_empty_props = body.len() > 2
                && body[2] == 0x00
                && subscription_entries_fill_exactly(body, 3, with_qos);
            if !v3 && !v5_empty_props {
                return Err(MqttError::MalformedSubscriptionPayload { packet_type });
            }
            Ok(2)
        }
        MqttPacketType::Suback => {
            read_packet_id(packet_type, body)?;
            if body.len() < 3 {
                return Err(invalid_len());
            }
            // v3 : chaque octet après le packet id est un code de retour.
            // (Un SUBACK v5 à propriétés vides passe aussi : 0x00 est un code
            // valide.)
            if let Some(&bad) = body[2..]
                .iter()
                .find(|code| !SUBACK_REASON_CODES.contains(code))
            {
                return Err(MqttError::InvalidReasonCode {
                    packet_type,
                    code: bad,
                });
            }
            Ok(2)
        }
        MqttPacketType::Unsuback => {
            read_packet_id(packet_type, body)?;
            match body.len() {
                // v3.1.1 : packet id seul.
                2 => {}
                // v5 : propriétés puis au moins un reason code.
                _ => {
                    let props = &body[2..];
                    let Ok((props_len, varint_bytes)) = decode_remaining_length(props) else {
                        return Err(invalid_len());
                    };
                    let codes_start = 2 + varint_bytes + props_len as usize;
                    if codes_start >= body.len() {
                        return Err(invalid_len());
                    }
                    if let Some(&bad) = body[codes_start..]
                        .iter()
                        .find(|code| !UNSUBACK_REASON_CODES.contains(code))
                    {
                        return Err(MqttError::InvalidReasonCode {
                            packet_type,
                            code: bad,
                        });
                    }
                }
            }
            Ok(2)
        }
        MqttPacketType::Pingreq | MqttPacketType::Pingresp => {
            if !body.is_empty() {
                return Err(invalid_len());
            }
            Ok(0)
        }
        MqttPacketType::Disconnect => {
            // v3.1.1 : corps vide ; v5 : reason code + éventuelles propriétés.
            match body.len() {
                0 => Ok(0),
                _ => {
                    if !is_valid_disconnect_reason(body[0]) {
                        return Err(MqttError::InvalidReasonCode {
                            packet_type,
                            code: body[0],
                        });
                    }
                    if body.len() > 1 && !properties_fill_exactly(&body[1..]) {
                        return Err(invalid_len());
                    }
                    Ok(body.len())
                }
            }
        }
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
        // PUBLISH accepte dup/retain et QoS 0-2…
        assert!(validate_fixed_header_flags(MqttPacketType::Publish, 0x3D).is_ok());
        // …mais pas QoS 3 (les deux bits QoS à 1).
        assert!(matches!(
            validate_fixed_header_flags(MqttPacketType::Publish, 0x3F),
            Err(MqttError::InvalidQos { qos: 3 })
        ));

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
        // CONNACK : session present + return code valides
        assert_eq!(
            variable_header_len(MqttPacketType::Connack, 0x20, &[0, 0]).unwrap(),
            2
        );
        assert!(matches!(
            variable_header_len(MqttPacketType::Connack, 0x20, &[0]),
            Err(MqttError::InvalidRemainingLength { .. })
        ));
        // Return code hors plage (v3 : 0-5)
        assert!(matches!(
            variable_header_len(MqttPacketType::Connack, 0x20, &[0, 0x42]),
            Err(MqttError::InvalidReasonCode { .. })
        ));

        // PUBLISH : trop court
        assert!(matches!(
            variable_header_len(MqttPacketType::Publish, 0x30, &[0]),
            Err(MqttError::VariableHeaderTooShort { .. })
        ));
        // PUBLISH : wildcard interdit dans un topic name
        assert!(matches!(
            variable_header_len(MqttPacketType::Publish, 0x30, &[0, 3, b'a', b'/', b'#']),
            Err(MqttError::InvalidTopic)
        ));

        // Types sans variable header
        assert_eq!(
            variable_header_len(MqttPacketType::Disconnect, 0xE0, &[]).unwrap(),
            0
        );
        assert_eq!(
            variable_header_len(MqttPacketType::Pingresp, 0xD0, &[]).unwrap(),
            0
        );
        // PINGRESP avec un corps : rejeté
        assert!(matches!(
            variable_header_len(MqttPacketType::Pingresp, 0xD0, &[0]),
            Err(MqttError::InvalidRemainingLength { .. })
        ));

        // PUBACK v3 : packet id (non nul) seul
        assert_eq!(
            variable_header_len(MqttPacketType::Puback, 0x40, &[0, 1]).unwrap(),
            2
        );
        assert!(matches!(
            variable_header_len(MqttPacketType::Puback, 0x40, &[0, 0]),
            Err(MqttError::ZeroPacketId)
        ));
        // PUBACK avec remaining length fantaisiste (source de faux positifs)
        assert!(matches!(
            variable_header_len(MqttPacketType::Puback, 0x40, &[0, 1, 0x27, 0xB1, 0xDB]),
            Err(MqttError::InvalidReasonCode { .. })
        ));
    }

    #[test]
    fn test_connect_protocol_name_rules() {
        // "MQTT" niveau 4 (v3.1.1)
        let mqtt_v4 = [0, 4, b'M', b'Q', b'T', b'T', 4, 0x02, 0, 60];
        assert_eq!(
            variable_header_len(MqttPacketType::Connect, 0x10, &mqtt_v4).unwrap(),
            10
        );

        // "MQIsdp" niveau 3 (v3.1) : variable header de 12 octets
        let mqisdp = [0, 6, b'M', b'Q', b'I', b's', b'd', b'p', 3, 0x02, 0, 5];
        assert_eq!(
            variable_header_len(MqttPacketType::Connect, 0x10, &mqisdp).unwrap(),
            12
        );

        // Nom inconnu
        let bad_name = [0, 4, b'A', b'B', b'C', b'D', 4, 0x02, 0, 60];
        assert!(matches!(
            variable_header_len(MqttPacketType::Connect, 0x10, &bad_name),
            Err(MqttError::InvalidProtocolName)
        ));

        // Niveau incohérent avec le nom
        let bad_level = [0, 4, b'M', b'Q', b'T', b'T', 3, 0x02, 0, 60];
        assert!(matches!(
            variable_header_len(MqttPacketType::Connect, 0x10, &bad_level),
            Err(MqttError::InvalidProtocolLevel { level: 3 })
        ));

        // Bit réservé des connect flags à 1
        let reserved = [0, 4, b'M', b'Q', b'T', b'T', 4, 0x03, 0, 60];
        assert!(matches!(
            variable_header_len(MqttPacketType::Connect, 0x10, &reserved),
            Err(MqttError::InvalidReservedConnectFlag)
        ));
    }

    #[test]
    fn test_subscription_payload_rules() {
        // SUBSCRIBE v3 : pid + (topic, qos) exactement
        let sub = [0, 1, 0, 1, b'a', 0];
        assert_eq!(
            variable_header_len(MqttPacketType::Subscribe, 0x82, &sub).unwrap(),
            2
        );

        // Longueur de topic qui déborde : rejeté
        let bad = [0, 1, 0xA0, 0x34, b'a', 0];
        assert!(matches!(
            variable_header_len(MqttPacketType::Subscribe, 0x82, &bad),
            Err(MqttError::MalformedSubscriptionPayload { .. })
        ));

        // UNSUBSCRIBE v3 : pid + topic exactement
        let unsub = [0, 1, 0, 1, b'a'];
        assert_eq!(
            variable_header_len(MqttPacketType::Unsubscribe, 0xA2, &unsub).unwrap(),
            2
        );

        // SUBACK : codes de retour valides uniquement
        let suback = [0, 1, 0, 1, 0x80];
        assert_eq!(
            variable_header_len(MqttPacketType::Suback, 0x90, &suback).unwrap(),
            2
        );
        let bad_suback = [0, 1, 0x37, 0x05];
        assert!(matches!(
            variable_header_len(MqttPacketType::Suback, 0x90, &bad_suback),
            Err(MqttError::InvalidReasonCode { code: 0x37, .. })
        ));
    }

    #[test]
    fn test_header_available() {
        assert!(validate_mqtt_header_available(10, 5).is_ok());
        assert!(matches!(
            validate_mqtt_header_available(3, 5),
            Err(MqttError::PacketTooShort { actual: 3, min: 5 })
        ));
    }

    #[test]
    fn test_validate_mqtt_min_length() {
        assert!(validate_mqtt_min_length(&[0xC0, 0x00]).is_ok());
        assert!(matches!(
            validate_mqtt_min_length(&[0xC0]),
            Err(MqttError::PacketTooShort { actual: 1, min: 2 })
        ));
        assert!(matches!(
            validate_mqtt_min_length(&[]),
            Err(MqttError::PacketTooShort { actual: 0, min: 2 })
        ));
    }

    #[test]
    fn test_decode_remaining_length_truncated_varint() {
        // Buffer épuisé alors que l'octet de continuation annonce une suite.
        assert!(matches!(
            decode_remaining_length(&[0x80]),
            Err(MqttError::MalformedRemainingLength)
        ));
        assert!(matches!(
            decode_remaining_length(&[0xFF, 0xFF, 0x80]),
            Err(MqttError::MalformedRemainingLength)
        ));
        assert!(matches!(
            decode_remaining_length(&[]),
            Err(MqttError::MalformedRemainingLength)
        ));
    }

    #[test]
    fn test_decode_remaining_length_max_value() {
        // Valeur maximale encodable : 268 435 455 sur 4 octets.
        assert_eq!(
            decode_remaining_length(&[0xFF, 0xFF, 0xFF, 0x7F]).unwrap(),
            (268_435_455, 4)
        );
    }

    #[test]
    fn test_validate_remaining_length_available() {
        assert!(validate_remaining_length_available(4, 4).is_ok());
        assert!(validate_remaining_length_available(0, 0).is_ok());
        assert!(matches!(
            validate_remaining_length_available(5, 4),
            Err(MqttError::RemainingLengthExceedsBuffer {
                remaining_length: 5,
                available: 4
            })
        ));
    }
}
