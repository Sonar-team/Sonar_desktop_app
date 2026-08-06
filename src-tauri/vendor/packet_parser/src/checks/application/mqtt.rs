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

/// Vérifie qu'un reason code DISCONNECT est plausible : 0x00 (normal),
/// 0x04 (disconnect with will) ou reason code v5 (0x80-0xA2). Produit la
/// même erreur `InvalidReasonCode` que le point d'appel historique.
fn validate_disconnect_reason(code: u8) -> Result<(), MqttError> {
    if code == 0x00 || code == 0x04 || (0x80..=0xA2).contains(&code) {
        Ok(())
    } else {
        Err(MqttError::InvalidReasonCode {
            packet_type: MqttPacketType::Disconnect,
            code,
        })
    }
}

/// Vérifie un code de retour CONNACK : v3.1.1 : 0 (accepté) à 5 (non
/// autorisé) ; v5 : 0 ou reason code >= 0x80. Produit la même erreur
/// `InvalidReasonCode` que le point d'appel historique.
fn validate_connack_code(code: u8) -> Result<(), MqttError> {
    if code <= 5 || (0x80..=0xA2).contains(&code) {
        Ok(())
    } else {
        Err(MqttError::InvalidReasonCode {
            packet_type: MqttPacketType::Connack,
            code,
        })
    }
}

/// Vérifie qu'un bloc de propriétés MQTT v5 remplit exactement `buf` :
/// varint de longueur + contenu, sans reste. `remaining_length` est la
/// longueur du corps complet, reprise telle quelle dans l'erreur
/// `InvalidRemainingLength` (même mapping que l'inline d'origine).
fn validate_properties_fill_exactly(
    packet_type: MqttPacketType,
    remaining_length: u32,
    buf: &[u8],
) -> Result<(), MqttError> {
    let filled = match decode_remaining_length(buf) {
        Ok((props_len, varint_bytes)) => varint_bytes + props_len as usize == buf.len(),
        Err(_) => false,
    };
    if filled {
        Ok(())
    } else {
        Err(MqttError::InvalidRemainingLength {
            packet_type,
            remaining_length,
        })
    }
}

/// Topic name d'un PUBLISH : UTF-8 valide, non vide, sans caractère de
/// contrôle ni wildcard ('#' et '+' sont interdits à la publication).
/// Chaque mode d'échec sort avec son variant dédié.
fn validate_publish_topic(topic: &[u8]) -> Result<(), MqttError> {
    if topic.is_empty() {
        return Err(MqttError::EmptyTopic);
    }
    let s = std::str::from_utf8(topic).map_err(|_| MqttError::TopicNotUtf8)?;
    if s.chars().any(|c| c == '#' || c == '+') {
        return Err(MqttError::WildcardInPublishTopic);
    }
    if s.chars().any(char::is_control) {
        return Err(MqttError::ControlCharacterInTopic);
    }
    Ok(())
}

/// Vérifie un topic filter de SUBSCRIBE/UNSUBSCRIBE : UTF-8 valide, non
/// vide, sans caractère de contrôle (les wildcards sont autorisés ici).
/// L'erreur est `MalformedSubscriptionPayload` : celle qui sortait déjà de
/// la branche d'appel quand le filtre était invalide.
fn validate_topic_filter(packet_type: MqttPacketType, topic: &[u8]) -> Result<(), MqttError> {
    let malformed = || MqttError::MalformedSubscriptionPayload { packet_type };
    if topic.is_empty() {
        return Err(malformed());
    }
    let s = std::str::from_utf8(topic).map_err(|_| malformed())?;
    if s.chars().any(|c| c.is_control()) {
        return Err(malformed());
    }
    Ok(())
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
/// suite d'entrées doit consommer `body` exactement. Reste booléen : la
/// branche d'appel tente les formes v3 puis v5 et ne produit une erreur que
/// si les deux échouent (structure d'essai double à préserver).
fn subscription_entries_fill_exactly(
    packet_type: MqttPacketType,
    body: &[u8],
    start: usize,
    with_qos: bool,
) -> bool {
    let mut off = start;
    let mut entries = 0usize;
    while off < body.len() {
        if body.len() - off < 2 {
            return false;
        }
        let topic_len = u16::from_be_bytes([body[off], body[off + 1]]) as usize;
        off += 2;
        if body.len() - off < topic_len
            || validate_topic_filter(packet_type, &body[off..off + topic_len]).is_err()
        {
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

/// Vérifie le couple (nom de protocole, protocol level) d'un CONNECT et
/// retourne les valeurs typées : "MQTT" niveau 4/5 ou "MQIsdp" niveau 3.
/// Même ordre de test que l'inline d'origine : nom connu d'abord
/// (`InvalidProtocolName`), niveau cohérent ensuite (`InvalidProtocolLevel`).
fn extract_protocol_name_level(name: &[u8], level: u8) -> Result<(&[u8], u8), MqttError> {
    match (name, level) {
        (b"MQTT", 4 | 5) | (b"MQIsdp", 3) => Ok((name, level)),
        (b"MQTT", _) | (b"MQIsdp", _) => Err(MqttError::InvalidProtocolLevel { level }),
        _ => Err(MqttError::InvalidProtocolName),
    }
}

/// Vérifie que le bit réservé (bit 0) des connect flags d'un CONNECT est à
/// zéro, comme l'exige la spec (`InvalidReservedConnectFlag` sinon).
fn validate_connect_flags(connect_flags: u8) -> Result<(), MqttError> {
    if connect_flags & 0x01 != 0 {
        return Err(MqttError::InvalidReservedConnectFlag);
    }
    Ok(())
}

/// Variable header CONNECT : longueur du nom, nom + niveau de protocole,
/// connect flags, keep alive — vérifiés en séquence wire-order.
fn validate_connect_vh(body: &[u8]) -> Result<usize, MqttError> {
    if body.len() < 2 {
        return Err(MqttError::VariableHeaderTooShort {
            packet_type: MqttPacketType::Connect,
            actual: body.len(),
            min: 2,
        });
    }
    let name_len = u16::from_be_bytes([body[0], body[1]]) as usize;
    // name + level (1) + connect flags (1) + keep alive (2)
    let vh_len = 2 + name_len + 4;
    if body.len() < vh_len {
        return Err(MqttError::VariableHeaderTooShort {
            packet_type: MqttPacketType::Connect,
            actual: body.len(),
            min: vh_len,
        });
    }
    // Les valeurs typées (nom, niveau) ne peuvent pas être placées dans
    // MqttPacket (champs publics = slices bruts) : écart assumé, cf. parse.
    extract_protocol_name_level(&body[2..2 + name_len], body[2 + name_len])?;
    validate_connect_flags(body[2 + name_len + 1])?;
    Ok(vh_len)
}

/// Variable header CONNACK : acknowledge flags (bit 0 seul), code de
/// retour, puis éventuel bloc de propriétés v5 exact.
fn validate_connack_vh(first_byte: u8, body: &[u8]) -> Result<usize, MqttError> {
    if body.len() < 2 {
        return Err(MqttError::InvalidRemainingLength {
            packet_type: MqttPacketType::Connack,
            remaining_length: body.len() as u32,
        });
    }
    if body[0] > 1 {
        // Connect acknowledge flags : seuls bits 0 (session present).
        return Err(MqttError::InvalidHeaderFlags {
            packet_type: first_byte >> 4,
            flags: body[0],
        });
    }
    validate_connack_code(body[1])?;
    // v3 : exactement 2 octets ; v5 : + bloc de propriétés exact.
    if body.len() > 2 {
        validate_properties_fill_exactly(MqttPacketType::Connack, body.len() as u32, &body[2..])?;
    }
    Ok(body.len())
}

/// Variable header PUBLISH : longueur de topic, topic name valide, puis
/// packet id si QoS > 0.
fn validate_publish_vh(first_byte: u8, body: &[u8]) -> Result<usize, MqttError> {
    if body.len() < 2 {
        return Err(MqttError::VariableHeaderTooShort {
            packet_type: MqttPacketType::Publish,
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
        read_packet_id(MqttPacketType::Publish, &body[vh_len..])?;
        vh_len += 2;
    }
    Ok(vh_len)
}

/// Variable header PUBACK/PUBREC/PUBREL/PUBCOMP : packet id non nul, puis
/// reason code + éventuel bloc de propriétés en forme v5.
fn validate_ack_vh(packet_type: MqttPacketType, body: &[u8]) -> Result<usize, MqttError> {
    let invalid_len = || MqttError::InvalidRemainingLength {
        packet_type,
        remaining_length: body.len() as u32,
    };
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
            if body.len() > 3 {
                validate_properties_fill_exactly(packet_type, body.len() as u32, &body[3..])?;
            }
        }
        _ => return Err(invalid_len()),
    }
    Ok(body.len())
}

/// Corps SUBSCRIBE/UNSUBSCRIBE : packet id non nul, puis une suite
/// d'entrées de souscription qui remplit exactement le corps. Essai double
/// v3 puis v5 à propriétés vides ; erreur seulement si les deux échouent,
/// pour ne pas changer quelle erreur sort sur quel input.
fn validate_subscription_vh(packet_type: MqttPacketType, body: &[u8]) -> Result<usize, MqttError> {
    read_packet_id(packet_type, body)?;
    let with_qos = packet_type == MqttPacketType::Subscribe;
    // v3 : les entrées commencent après le packet id ; v5 : après un
    // bloc de propriétés (toléré uniquement vide : 0x00).
    let v3 = subscription_entries_fill_exactly(packet_type, body, 2, with_qos);
    let v5_empty_props = body.len() > 2
        && body[2] == 0x00
        && subscription_entries_fill_exactly(packet_type, body, 3, with_qos);
    if !v3 && !v5_empty_props {
        return Err(MqttError::MalformedSubscriptionPayload { packet_type });
    }
    Ok(2)
}

/// Corps SUBACK : packet id non nul, puis au moins un code de retour, tous
/// dans la liste des codes SUBACK plausibles.
fn validate_suback_vh(body: &[u8]) -> Result<usize, MqttError> {
    read_packet_id(MqttPacketType::Suback, body)?;
    if body.len() < 3 {
        return Err(MqttError::InvalidRemainingLength {
            packet_type: MqttPacketType::Suback,
            remaining_length: body.len() as u32,
        });
    }
    // v3 : chaque octet après le packet id est un code de retour.
    // (Un SUBACK v5 à propriétés vides passe aussi : 0x00 est un code
    // valide.)
    if let Some(&bad) = body[2..]
        .iter()
        .find(|code| !SUBACK_REASON_CODES.contains(code))
    {
        return Err(MqttError::InvalidReasonCode {
            packet_type: MqttPacketType::Suback,
            code: bad,
        });
    }
    Ok(2)
}

/// Corps UNSUBACK : packet id non nul seul (v3.1.1), ou propriétés puis au
/// moins un reason code plausible (v5).
fn validate_unsuback_vh(body: &[u8]) -> Result<usize, MqttError> {
    let invalid_len = || MqttError::InvalidRemainingLength {
        packet_type: MqttPacketType::Unsuback,
        remaining_length: body.len() as u32,
    };
    read_packet_id(MqttPacketType::Unsuback, body)?;
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
                    packet_type: MqttPacketType::Unsuback,
                    code: bad,
                });
            }
        }
    }
    Ok(2)
}

/// Corps PINGREQ/PINGRESP : doit être vide.
fn validate_ping_vh(packet_type: MqttPacketType, body: &[u8]) -> Result<usize, MqttError> {
    if !body.is_empty() {
        return Err(MqttError::InvalidRemainingLength {
            packet_type,
            remaining_length: body.len() as u32,
        });
    }
    Ok(0)
}

/// Corps DISCONNECT : vide (v3.1.1), ou reason code plausible + éventuel
/// bloc de propriétés exact (v5).
fn validate_disconnect_vh(body: &[u8]) -> Result<usize, MqttError> {
    // v3.1.1 : corps vide ; v5 : reason code + éventuelles propriétés.
    match body.len() {
        0 => Ok(0),
        _ => {
            validate_disconnect_reason(body[0])?;
            if body.len() > 1 {
                validate_properties_fill_exactly(
                    MqttPacketType::Disconnect,
                    body.len() as u32,
                    &body[1..],
                )?;
            }
            Ok(body.len())
        }
    }
}

/// Valide le corps (`remaining length` octets) selon le type de paquet et
/// retourne la longueur du variable header.
///
/// Les règles sont celles de MQTT 3.1/3.1.1, avec une tolérance pour les
/// formes v5 (reason codes + bloc de propriétés) — l'objectif est de
/// discriminer du vrai MQTT face à du bruit binaire, pas de valider
/// exhaustivement la spec.
///
/// Simple dispatch vers une fonction de validation par type de paquet,
/// chacune enchaînant les checks champ par champ dans l'ordre du wire.
pub fn variable_header_len(
    packet_type: MqttPacketType,
    first_byte: u8,
    body: &[u8],
) -> Result<usize, MqttError> {
    match packet_type {
        MqttPacketType::Connect => validate_connect_vh(body),
        MqttPacketType::Connack => validate_connack_vh(first_byte, body),
        MqttPacketType::Publish => validate_publish_vh(first_byte, body),
        MqttPacketType::Puback
        | MqttPacketType::Pubrec
        | MqttPacketType::Pubrel
        | MqttPacketType::Pubcomp => validate_ack_vh(packet_type, body),
        MqttPacketType::Subscribe | MqttPacketType::Unsubscribe => {
            validate_subscription_vh(packet_type, body)
        }
        MqttPacketType::Suback => validate_suback_vh(body),
        MqttPacketType::Unsuback => validate_unsuback_vh(body),
        MqttPacketType::Pingreq | MqttPacketType::Pingresp => validate_ping_vh(packet_type, body),
        MqttPacketType::Disconnect => validate_disconnect_vh(body),
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
            Err(MqttError::WildcardInPublishTopic)
        ));
        // PUBLISH : topic vide
        assert!(matches!(
            variable_header_len(MqttPacketType::Publish, 0x30, &[0, 0]),
            Err(MqttError::EmptyTopic)
        ));
        // PUBLISH : topic non UTF-8
        assert!(matches!(
            variable_header_len(MqttPacketType::Publish, 0x30, &[0, 2, 0xFF, 0xFE]),
            Err(MqttError::TopicNotUtf8)
        ));
        // PUBLISH : caractère de contrôle dans le topic
        assert!(matches!(
            variable_header_len(MqttPacketType::Publish, 0x30, &[0, 2, b'a', 0x01]),
            Err(MqttError::ControlCharacterInTopic)
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

    #[test]
    fn test_extract_protocol_name_level_rules() {
        assert!(matches!(
            extract_protocol_name_level(b"MQTT", 4),
            Ok((_, 4))
        ));
        assert!(matches!(
            extract_protocol_name_level(b"MQTT", 5),
            Ok((_, 5))
        ));
        assert!(matches!(
            extract_protocol_name_level(b"MQIsdp", 3),
            Ok((_, 3))
        ));
        // Nom connu mais niveau incohérent : le niveau est fautif.
        assert!(matches!(
            extract_protocol_name_level(b"MQTT", 3),
            Err(MqttError::InvalidProtocolLevel { level: 3 })
        ));
        // Nom inconnu : le nom est fautif, quel que soit le niveau.
        assert!(matches!(
            extract_protocol_name_level(b"ABCD", 4),
            Err(MqttError::InvalidProtocolName)
        ));
    }

    #[test]
    fn test_validate_connect_flags_reserved_bit() {
        assert!(validate_connect_flags(0x02).is_ok());
        assert!(validate_connect_flags(0x00).is_ok());
        assert!(matches!(
            validate_connect_flags(0x03),
            Err(MqttError::InvalidReservedConnectFlag)
        ));
    }

    #[test]
    fn test_validate_connack_code_rules() {
        assert!(validate_connack_code(0x00).is_ok());
        assert!(validate_connack_code(0x05).is_ok());
        assert!(validate_connack_code(0x87).is_ok());
        assert!(matches!(
            validate_connack_code(0x42),
            Err(MqttError::InvalidReasonCode { code: 0x42, .. })
        ));
    }

    #[test]
    fn test_validate_disconnect_reason_rules() {
        assert!(validate_disconnect_reason(0x00).is_ok());
        assert!(validate_disconnect_reason(0x04).is_ok());
        assert!(validate_disconnect_reason(0x8E).is_ok());
        assert!(matches!(
            validate_disconnect_reason(0xF1),
            Err(MqttError::InvalidReasonCode { code: 0xF1, .. })
        ));
    }

    #[test]
    fn test_validate_properties_fill_exactly_rules() {
        // Bloc de propriétés vide : varint 0x00 sans contenu.
        assert!(validate_properties_fill_exactly(MqttPacketType::Connack, 3, &[0x00]).is_ok());
        // Varint 1 + un octet de propriété : rempli exactement.
        assert!(
            validate_properties_fill_exactly(MqttPacketType::Connack, 4, &[0x01, 0x22]).is_ok()
        );
        // Octet non couvert par le varint : rejeté avec la remaining length
        // du corps complet, comme au point d'appel.
        assert!(matches!(
            validate_properties_fill_exactly(MqttPacketType::Connack, 4, &[0x00, 0x22]),
            Err(MqttError::InvalidRemainingLength {
                remaining_length: 4,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_topic_filter_rules() {
        // Les wildcards sont autorisés dans un topic filter.
        assert!(validate_topic_filter(MqttPacketType::Subscribe, b"a/+/#").is_ok());
        assert!(matches!(
            validate_topic_filter(MqttPacketType::Subscribe, b""),
            Err(MqttError::MalformedSubscriptionPayload { .. })
        ));
        // Non-UTF8 : rejeté.
        assert!(matches!(
            validate_topic_filter(MqttPacketType::Unsubscribe, &[0xFF, 0xFE]),
            Err(MqttError::MalformedSubscriptionPayload { .. })
        ));
        // Caractère de contrôle : rejeté.
        assert!(matches!(
            validate_topic_filter(MqttPacketType::Subscribe, &[b'a', 0x01]),
            Err(MqttError::MalformedSubscriptionPayload { .. })
        ));
    }
}
