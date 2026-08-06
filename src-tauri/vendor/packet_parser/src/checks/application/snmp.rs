// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::mem::size_of;

use crate::{
    errors::application::snmp::SnmpError,
    parse::application::protocols::snmp::{SnmpPduType, SnmpValue, SnmpVersion},
};

pub const ASN1_SEQUENCE_TAG: u8 = 0x30;
pub const ASN1_INTEGER_TAG: u8 = 0x02;
pub const ASN1_OCTET_STRING_TAG: u8 = 0x04;
pub const ASN1_NULL_TAG: u8 = 0x05;
pub const ASN1_OBJECT_IDENTIFIER_TAG: u8 = 0x06;

pub const SNMP_IP_ADDRESS_TAG: u8 = 0x40;
pub const SNMP_COUNTER32_TAG: u8 = 0x41;
pub const SNMP_GAUGE32_TAG: u8 = 0x42;
pub const SNMP_TIMETICKS_TAG: u8 = 0x43;
pub const SNMP_OPAQUE_TAG: u8 = 0x44;
pub const SNMP_COUNTER64_TAG: u8 = 0x46;
pub const SNMP_NO_SUCH_OBJECT_TAG: u8 = 0x80;
pub const SNMP_NO_SUCH_INSTANCE_TAG: u8 = 0x81;
pub const SNMP_END_OF_MIB_VIEW_TAG: u8 = 0x82;

pub const SNMP_MIN_PACKET_LEN: usize = 2;

pub fn validate_snmp_min_length(packet: &[u8]) -> Result<(), SnmpError> {
    if packet.len() < SNMP_MIN_PACKET_LEN {
        return Err(SnmpError::PacketTooShort {
            min: SNMP_MIN_PACKET_LEN,
            actual: packet.len(),
        });
    }

    Ok(())
}

pub fn ensure_available(
    field: &'static str,
    actual: usize,
    needed: usize,
) -> Result<(), SnmpError> {
    if actual < needed {
        return Err(SnmpError::Truncated {
            field,
            needed,
            actual,
        });
    }

    Ok(())
}

pub fn validate_tag(field: &'static str, actual: u8, expected: u8) -> Result<(), SnmpError> {
    if actual != expected {
        return Err(SnmpError::InvalidTag {
            field,
            expected,
            actual,
        });
    }

    Ok(())
}

pub fn validate_no_trailing(
    field: &'static str,
    consumed: usize,
    packet_len: usize,
) -> Result<(), SnmpError> {
    if consumed != packet_len {
        return Err(SnmpError::TrailingData {
            consumed,
            packet_len,
        });
    }

    let _ = field;
    Ok(())
}

pub fn validate_integer_length(field: &'static str, actual: usize) -> Result<(), SnmpError> {
    if actual == 0 || actual > 8 {
        return Err(SnmpError::InvalidIntegerLength { field, actual });
    }

    Ok(())
}

pub fn validate_unsigned_length(field: &'static str, actual: usize) -> Result<(), SnmpError> {
    if actual == 0 || actual > 8 {
        return Err(SnmpError::InvalidUnsignedLength { field, actual });
    }

    Ok(())
}

pub fn validate_version(version: i64) -> Result<SnmpVersion, SnmpError> {
    match version {
        0 => Ok(SnmpVersion::V1),
        1 => Ok(SnmpVersion::V2c),
        3 => Ok(SnmpVersion::V3),
        _ => Err(SnmpError::UnsupportedVersion { version }),
    }
}

pub fn validate_pdu_type(tag: u8, version: SnmpVersion) -> Result<SnmpPduType, SnmpError> {
    let pdu_type = match tag {
        0xA0 => SnmpPduType::GetRequest,
        0xA1 => SnmpPduType::GetNextRequest,
        0xA2 => SnmpPduType::Response,
        0xA3 => SnmpPduType::SetRequest,
        0xA4 => SnmpPduType::TrapV1,
        0xA5 => SnmpPduType::GetBulkRequest,
        0xA6 => SnmpPduType::InformRequest,
        0xA7 => SnmpPduType::TrapV2,
        0xA8 => SnmpPduType::Report,
        _ => return Err(SnmpError::UnsupportedPduType { tag, version }),
    };

    let supported = match version {
        SnmpVersion::V1 => matches!(
            pdu_type,
            SnmpPduType::GetRequest
                | SnmpPduType::GetNextRequest
                | SnmpPduType::Response
                | SnmpPduType::SetRequest
                | SnmpPduType::TrapV1
        ),
        SnmpVersion::V2c => matches!(
            pdu_type,
            SnmpPduType::GetRequest
                | SnmpPduType::GetNextRequest
                | SnmpPduType::Response
                | SnmpPduType::SetRequest
                | SnmpPduType::GetBulkRequest
                | SnmpPduType::InformRequest
                | SnmpPduType::TrapV2
        ),
        SnmpVersion::V3 => matches!(
            pdu_type,
            SnmpPduType::GetRequest
                | SnmpPduType::GetNextRequest
                | SnmpPduType::Response
                | SnmpPduType::SetRequest
                | SnmpPduType::GetBulkRequest
                | SnmpPduType::InformRequest
                | SnmpPduType::TrapV2
                | SnmpPduType::Report
        ),
    };

    if supported {
        Ok(pdu_type)
    } else {
        Err(SnmpError::UnsupportedPduType { tag, version })
    }
}

/// TLV BER lu par [`read_tlv`] : tag, valeur et encodage complet, empruntés au paquet.
pub(crate) struct BerTlv<'a> {
    pub(crate) tag: u8,
    pub(crate) value: &'a [u8],
    pub(crate) encoded: &'a [u8],
}

/// Lit et vérifie un TLV BER à `offset` : présence de l'en-tête, rejet des longueurs
/// indéfinies ou trop larges, valeur bornée au buffer ; avance `offset` après le TLV.
pub(crate) fn read_tlv<'a>(
    data: &'a [u8],
    offset: &mut usize,
    field: &'static str,
) -> Result<BerTlv<'a>, SnmpError> {
    let start = *offset;
    let header_needed = start
        .checked_add(2)
        .ok_or(SnmpError::LengthOverflow { field })?;
    ensure_available(field, data.len(), header_needed)?;

    let tag = data[start];
    let first_len = data[start + 1];
    *offset = header_needed;

    let length = if first_len & 0x80 == 0 {
        usize::from(first_len)
    } else {
        let len_len = usize::from(first_len & 0x7F);
        if len_len == 0 {
            return Err(SnmpError::UnsupportedIndefiniteLength { field });
        }
        if len_len > size_of::<usize>() {
            return Err(SnmpError::UnsupportedLengthSize {
                field,
                actual: len_len,
            });
        }

        let length_end = (*offset)
            .checked_add(len_len)
            .ok_or(SnmpError::LengthOverflow { field })?;
        ensure_available(field, data.len(), length_end)?;

        let mut length = 0usize;
        for &byte in &data[*offset..length_end] {
            length = length
                .checked_mul(256)
                .and_then(|value| value.checked_add(usize::from(byte)))
                .ok_or(SnmpError::LengthOverflow { field })?;
        }
        *offset = length_end;
        length
    };

    let value_start = *offset;
    let value_end = value_start
        .checked_add(length)
        .ok_or(SnmpError::LengthOverflow { field })?;
    ensure_available(field, data.len(), value_end)?;
    *offset = value_end;

    Ok(BerTlv {
        tag,
        value: &data[value_start..value_end],
        encoded: &data[start..value_end],
    })
}

/// Vérifie la longueur d'un INTEGER BER (1 à 8 octets) puis décode sa valeur signée
/// en complément à deux.
pub(crate) fn parse_integer_value(value: &[u8], field: &'static str) -> Result<i64, SnmpError> {
    validate_integer_length(field, value.len())?;

    let negative = value[0] & 0x80 != 0;
    let mut parsed = if negative { -1i64 } else { 0i64 };
    for &byte in value {
        parsed = (parsed << 8) | i64::from(byte);
    }

    Ok(parsed)
}

/// Décode un entier non signé BER et vérifie qu'il tient dans un `u32`.
pub(crate) fn parse_u32_value(value: &[u8], field: &'static str) -> Result<u32, SnmpError> {
    let parsed = parse_unsigned_value(value, field)?;
    u32::try_from(parsed).map_err(|_| SnmpError::UnsignedOverflow { field })
}

/// Vérifie la longueur d'un entier non signé BER (1 à 8 octets) puis décode sa valeur.
pub(crate) fn parse_unsigned_value(value: &[u8], field: &'static str) -> Result<u64, SnmpError> {
    validate_unsigned_length(field, value.len())?;

    let mut parsed = 0u64;
    for &byte in value {
        parsed = (parsed << 8) | u64::from(byte);
    }

    Ok(parsed)
}

/// Vérifie qu'une valeur NULL BER est vide et retourne `SnmpValue::Null`.
pub(crate) fn extract_null(value: &[u8]) -> Result<SnmpValue<'_>, SnmpError> {
    if !value.is_empty() {
        return Err(SnmpError::NonEmptyNull {
            length: value.len(),
        });
    }

    Ok(SnmpValue::Null)
}

/// Vérifie qu'une adresse IP SNMP fait exactement 4 octets et la retourne en `[u8; 4]`.
pub(crate) fn extract_ip_address(value: &[u8]) -> Result<[u8; 4], SnmpError> {
    if value.len() != 4 {
        return Err(SnmpError::InvalidIpAddressLength {
            actual: value.len(),
        });
    }

    // Indexation sûre : la longueur exacte de 4 octets vient d'être vérifiée.
    Ok([value[0], value[1], value[2], value[3]])
}

/// Vérifie qu'une valeur d'exception SNMPv2 (noSuchObject, noSuchInstance,
/// endOfMibView) est vide.
pub(crate) fn validate_exception_empty(value: &[u8], field: &'static str) -> Result<(), SnmpError> {
    if !value.is_empty() {
        return Err(SnmpError::NonEmptyException {
            field,
            length: value.len(),
        });
    }

    Ok(())
}

/// Vérifie que le tag du champ `v3_data` est SEQUENCE (ScopedPDU en clair) ou
/// OCTET STRING (ScopedPDU chiffrée).
///
/// Le champ `expected` de l'erreur annonce `ASN1_SEQUENCE_TAG` alors qu'OCTET STRING
/// est aussi accepté : comportement historique conservé volontairement pour garder
/// les mêmes erreurs pour les mêmes inputs.
pub(crate) fn validate_v3_data_tag(actual: u8) -> Result<(), SnmpError> {
    if actual != ASN1_SEQUENCE_TAG && actual != ASN1_OCTET_STRING_TAG {
        return Err(SnmpError::InvalidTag {
            field: "v3_data",
            expected: ASN1_SEQUENCE_TAG,
            actual,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_null_accepte_valeur_vide() {
        assert!(matches!(extract_null(&[]), Ok(SnmpValue::Null)));
    }

    #[test]
    fn extract_null_rejette_valeur_non_vide() {
        assert!(matches!(
            extract_null(&[0x01]),
            Err(SnmpError::NonEmptyNull { length: 1 })
        ));
    }

    #[test]
    fn extract_ip_address_accepte_quatre_octets() {
        assert_eq!(extract_ip_address(&[10, 0, 0, 1]), Ok([10, 0, 0, 1]));
    }

    #[test]
    fn extract_ip_address_rejette_mauvaise_longueur() {
        assert!(matches!(
            extract_ip_address(&[192, 168, 1]),
            Err(SnmpError::InvalidIpAddressLength { actual: 3 })
        ));
        assert!(matches!(
            extract_ip_address(&[1, 2, 3, 4, 5]),
            Err(SnmpError::InvalidIpAddressLength { actual: 5 })
        ));
    }

    #[test]
    fn validate_exception_empty_accepte_valeur_vide() {
        assert!(validate_exception_empty(&[], "no_such_object").is_ok());
    }

    #[test]
    fn validate_exception_empty_rejette_valeur_non_vide() {
        assert!(matches!(
            validate_exception_empty(&[0x01], "no_such_object"),
            Err(SnmpError::NonEmptyException {
                field: "no_such_object",
                length: 1,
            })
        ));
    }

    #[test]
    fn validate_v3_data_tag_accepte_sequence_et_octet_string() {
        assert!(validate_v3_data_tag(ASN1_SEQUENCE_TAG).is_ok());
        assert!(validate_v3_data_tag(ASN1_OCTET_STRING_TAG).is_ok());
    }

    #[test]
    fn validate_v3_data_tag_rejette_autre_tag() {
        assert!(matches!(
            validate_v3_data_tag(ASN1_INTEGER_TAG),
            Err(SnmpError::InvalidTag {
                field: "v3_data",
                expected: ASN1_SEQUENCE_TAG,
                actual: ASN1_INTEGER_TAG,
            })
        ));
    }
}
