// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Parsing des noms DNS (RFC 1035 §3.1) avec support de la compression
//! (§4.1.4), et parsing générique d'un resource record.

use crate::{
    checks::application::dns::{
        check_dns_label_bounds, check_dns_name_offset, check_dns_query_size,
    },
    errors::application::dns::DnsQueryParseError,
};

/// Les deux bits hauts d'un octet de longueur signalent un pointeur de
/// compression (`11`) ; `01` et `10` sont réservés.
const POINTER_MASK: u8 = 0xC0;

/// Nombre maximal de pointeurs suivis pour un même nom. Chaque pointeur
/// devant strictement pointer en arrière, cette borne n'est jamais atteinte
/// sur du trafic légitime ; elle protège contre les messages hostiles.
const MAX_POINTER_JUMPS: usize = 8;

/// Parse un nom DNS à `start` dans `message` (le message DNS complet,
/// en-tête inclus, car les pointeurs de compression sont des offsets depuis
/// le début du message).
///
/// Retourne le nom reconstruit et l'offset du premier octet suivant le nom
/// dans le flux d'origine (c.-à-d. après le premier pointeur rencontré).
pub fn parse_dns_name(message: &[u8], start: usize) -> Result<(String, usize), DnsQueryParseError> {
    // Allocation justifiée (METHODE_AJOUT_PROTOCOLE.md) : un nom DNS est
    // découpé en labels potentiellement non contigus (compression) — il doit
    // être reconstruit, pas emprunté.
    let mut labels: Vec<String> = Vec::new();
    let mut offset = start;
    // Offset de reprise dans le flux : figé au premier pointeur rencontré.
    let mut resume_offset: Option<usize> = None;
    let mut jumps = 0usize;

    loop {
        check_dns_name_offset(message, offset)?;
        let len_byte = message[offset];

        if len_byte & POINTER_MASK == POINTER_MASK {
            check_dns_name_offset(message, offset + 1)?;
            let target = (((len_byte & 0x3F) as usize) << 8) | message[offset + 1] as usize;

            if resume_offset.is_none() {
                resume_offset = Some(offset + 2);
            }
            jumps += 1;
            // Un pointeur doit référencer une position strictement antérieure,
            // sinon un message hostile peut créer une boucle infinie.
            if jumps > MAX_POINTER_JUMPS || target >= offset {
                return Err(DnsQueryParseError::InvalidCompressionPointer(offset));
            }
            offset = target;
            continue;
        }

        if len_byte & POINTER_MASK != 0 {
            return Err(DnsQueryParseError::ReservedLabelType(offset));
        }

        let len = len_byte as usize;
        if len == 0 {
            offset += 1;
            break;
        }
        offset += 1;

        check_dns_label_bounds(message, offset, len)?;
        let label = String::from_utf8(message[offset..offset + len].to_vec())?;
        labels.push(label);
        offset += len;
    }

    Ok((labels.join("."), resume_offset.unwrap_or(offset)))
}

/// Resource record brut (RFC 1035 §4.1.3), partagé par les sections answer,
/// authority et additional.
#[derive(Debug)]
pub struct RawRecord {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub data_length: u16,
    pub data: Vec<u8>,
}

/// Parse un resource record à `*offset` dans `message` et avance l'offset
/// après le record.
pub(crate) fn parse_resource_record(
    message: &[u8],
    offset: &mut usize,
) -> Result<RawRecord, DnsQueryParseError> {
    let (name, after_name) = parse_dns_name(message, *offset)?;

    // Type (2) + classe (2) + TTL (4) + longueur des données (2).
    check_dns_query_size(message, after_name, 10)?;
    let rtype = u16::from_be_bytes([message[after_name], message[after_name + 1]]);
    let rclass = u16::from_be_bytes([message[after_name + 2], message[after_name + 3]]);
    let ttl = u32::from_be_bytes([
        message[after_name + 4],
        message[after_name + 5],
        message[after_name + 6],
        message[after_name + 7],
    ]);
    let data_length = u16::from_be_bytes([message[after_name + 8], message[after_name + 9]]);

    let data_start = after_name + 10;
    check_dns_query_size(message, data_start, data_length as usize)?;
    let data = message[data_start..data_start + data_length as usize].to_vec();

    *offset = data_start + data_length as usize;

    Ok(RawRecord {
        name,
        rtype,
        rclass,
        ttl,
        data_length,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dns_name_without_compression() {
        let data = [
            3, b'w', b'w', b'w', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0,
        ];

        let (name, offset) = parse_dns_name(&data, 0).unwrap();

        assert_eq!(name, "www.google.com");
        assert_eq!(offset, 16);
    }

    #[test]
    fn test_parse_dns_name_with_compression_pointer() {
        // "example.com" à l'offset 0, puis à l'offset 13 : "www" + pointeur
        // vers l'offset 0.
        let mut data = vec![7u8];
        data.extend_from_slice(b"example");
        data.push(3);
        data.extend_from_slice(b"com");
        data.push(0); // offset 12 : fin du premier nom
        data.push(3);
        data.extend_from_slice(b"www");
        data.extend_from_slice(&[0xC0, 0x00]); // pointeur vers l'offset 0

        let (name, offset) = parse_dns_name(&data, 13).unwrap();

        assert_eq!(name, "www.example.com");
        assert_eq!(offset, 19); // juste après le pointeur de 2 octets
    }

    #[test]
    fn test_parse_dns_name_rejects_pointer_loop() {
        // Deux pointeurs qui se référencent mutuellement.
        let data = [0xC0u8, 0x02, 0xC0, 0x00];

        let result = parse_dns_name(&data, 2);

        assert!(matches!(
            result,
            Err(DnsQueryParseError::InvalidCompressionPointer(_))
        ));
    }

    #[test]
    fn test_parse_dns_name_rejects_self_pointer() {
        let data = [0xC0u8, 0x00];

        let result = parse_dns_name(&data, 0);

        assert!(matches!(
            result,
            Err(DnsQueryParseError::InvalidCompressionPointer(_))
        ));
    }

    #[test]
    fn test_parse_dns_name_rejects_reserved_label_type() {
        let data = [0x40u8, 0x00];

        let result = parse_dns_name(&data, 0);

        assert!(matches!(
            result,
            Err(DnsQueryParseError::ReservedLabelType(_))
        ));
    }

    #[test]
    fn test_parse_dns_name_truncated_pointer() {
        let data = [0xC0u8];

        let result = parse_dns_name(&data, 0);

        assert!(matches!(result, Err(DnsQueryParseError::OutOfBoundParse)));
    }

    #[test]
    fn test_parse_resource_record() {
        // Nom "a" + type A + classe IN + TTL 300 + 4 octets de données.
        let mut data = vec![1u8, b'a', 0];
        data.extend_from_slice(&[0x00, 0x01]); // type A
        data.extend_from_slice(&[0x00, 0x01]); // classe IN
        data.extend_from_slice(&300u32.to_be_bytes());
        data.extend_from_slice(&[0x00, 0x04]);
        data.extend_from_slice(&[93, 184, 216, 34]);

        let mut offset = 0;
        let record = parse_resource_record(&data, &mut offset).unwrap();

        assert_eq!(record.name, "a");
        assert_eq!(record.rtype, 1);
        assert_eq!(record.rclass, 1);
        assert_eq!(record.ttl, 300);
        assert_eq!(record.data_length, 4);
        assert_eq!(record.data, vec![93, 184, 216, 34]);
        assert_eq!(offset, data.len());
    }

    #[test]
    fn test_parse_resource_record_truncated_data() {
        let mut data = vec![0u8]; // nom racine
        data.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);
        data.extend_from_slice(&300u32.to_be_bytes());
        data.extend_from_slice(&[0x00, 0x10]); // annonce 16 octets, absents

        let mut offset = 0;
        let result = parse_resource_record(&data, &mut offset);

        assert!(matches!(
            result,
            Err(DnsQueryParseError::InsufficientData { .. })
        ));
    }
}
