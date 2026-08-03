// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::quic::QuicError,
    parse::application::protocols::quic::{ConnectionId, QuicLongHeader, QuicPacketType},
};

/// QUIC v1 version number (RFC 9000).
pub const QUIC_V1: u32 = 1;

/// Taille minimale plausible d'un paquet Short Header (RFC 9001 §5.4.2) :
/// 1 octet de flags + DCID (peut être vide) + au moins 20 octets après le
/// DCID — la protection d'en-tête exige un échantillon de 16 octets pris
/// 4 octets après le début du Packet Number, donc l'émetteur padde toujours
/// à cette taille.
pub const QUIC_SHORT_HEADER_MIN_LEN: usize = 21;

/// Heuristique stateless pour un paquet 1-RTT à Short Header (RFC 9000 §17.3).
///
/// Le Short Header est volontairement opaque : pas de version, pas de
/// longueur de DCID encodée. Les seuls invariants observables sont
/// Header Form = 0 et Fixed Bit = 1 (`b0 & 0xC0 == 0x40`), plus la taille
/// minimale imposée par l'échantillonnage de la protection d'en-tête.
/// Signature faible : à n'utiliser que derrière un garde-fou de port.
pub fn is_plausible_short_header(payload: &[u8]) -> bool {
    payload.len() >= QUIC_SHORT_HEADER_MIN_LEN && payload[0] & 0b1100_0000 == 0b0100_0000
}

/// Bounds-checked cursor over a QUIC packet slice.
///
/// Controlled-extraction helper: every read validates the remaining length
/// before slicing, so the parser never indexes out of bounds and stays
/// zero-copy (all reads return sub-slices of the original packet).
#[derive(Debug, Clone, Copy)]
pub struct QuicCursor<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> QuicCursor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    /// Number of bytes left to read.
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.offset)
    }

    /// Take `n` bytes as a borrowed sub-slice, validating bounds first.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], QuicError> {
        if self.remaining() < n {
            return Err(QuicError::Truncated {
                needed: n,
                remaining: self.remaining(),
            });
        }
        let slice = &self.buf[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    /// Take a single byte, validating bounds first.
    pub fn take_u8(&mut self) -> Result<u8, QuicError> {
        let slice = self.take(1)?;
        Ok(slice[0])
    }

    /// Take all remaining bytes as a borrowed sub-slice.
    pub fn take_rest(&mut self) -> &'a [u8] {
        let slice = &self.buf[self.offset..];
        self.offset = self.buf.len();
        slice
    }

    /// Read a QUIC variable-length integer (RFC 9000 §16) with bounds checking.
    ///
    /// The two most significant bits of the first byte encode the total
    /// length (1, 2, 4 or 8 bytes); the remaining 6 bits plus the
    /// continuation bytes form the big-endian value.
    pub fn read_varint(&mut self) -> Result<u64, QuicError> {
        let first = self.take_u8()?;
        let total_len = 1usize << (first >> 6); // 1, 2, 4 or 8
        let mut value = (first & 0b0011_1111) as u64;

        let continuation = total_len - 1;
        if self.remaining() < continuation {
            return Err(QuicError::TruncatedVarint {
                needed: continuation,
                remaining: self.remaining(),
            });
        }
        for &byte in self.take(continuation)? {
            value = (value << 8) | (byte as u64);
        }
        Ok(value)
    }
}

/// The header form bit must be 1 for a Long Header packet (RFC 9000 §17.2).
pub fn validate_long_header(header_form_long: bool) -> Result<(), QuicError> {
    if !header_form_long {
        return Err(QuicError::NotLongHeader);
    }
    Ok(())
}

/// The fixed bit must be 1 (RFC 9000 §17.2).
pub fn validate_fixed_bit(fixed_bit: bool) -> Result<(), QuicError> {
    if !fixed_bit {
        return Err(QuicError::FixedBitNotSet);
    }
    Ok(())
}

/// Checks the Long Header bits of the first byte (RFC 9000 §17.2) and
/// returns the packet type together with the Packet Number length (1..=4).
///
/// `NotLongHeader` is checked before `FixedBitNotSet` to preserve the
/// historical error precedence on a byte failing both checks.
pub fn extract_first_byte(b0: u8) -> Result<(QuicPacketType, u8), QuicError> {
    let header_form_long = (b0 & 0b1000_0000) != 0;
    validate_long_header(header_form_long)?;

    let fixed_bit = (b0 & 0b0100_0000) != 0;
    validate_fixed_bit(fixed_bit)?;

    let lptype = (b0 >> 4) & 0b11; // Long Packet Type (2 bits)
    // Bits 2-3 are reserved (protected by header protection, not validated).
    let pn_len_code = b0 & 0b11; // PN length code
    let pn_length = pn_len_code + 1; // 1..=4

    let packet_type = match lptype {
        0 => QuicPacketType::Initial,
        1 => QuicPacketType::ZeroRtt,
        2 => QuicPacketType::Handshake,
        3 => QuicPacketType::Retry,
        x => QuicPacketType::Unknown(x),
    };

    Ok((packet_type, pn_length))
}

/// Only QUIC v1 is accepted.
pub fn validate_version(version: u32) -> Result<(), QuicError> {
    if version != QUIC_V1 {
        return Err(QuicError::UnsupportedVersion(version));
    }
    Ok(())
}

/// Reads the 4-byte Version field and checks that it is QUIC v1.
///
/// Error order is preserved from the historical inline extraction:
/// `Truncated` if fewer than 4 bytes remain, then `UnsupportedVersion`.
pub fn extract_version(cur: &mut QuicCursor<'_>) -> Result<u32, QuicError> {
    let ver_bytes = cur.take(4)?;
    let version = u32::from_be_bytes([ver_bytes[0], ver_bytes[1], ver_bytes[2], ver_bytes[3]]);
    validate_version(version)?;
    Ok(version)
}

/// The announced payload length must fit in the remaining bytes.
pub fn validate_payload_available(available: usize, expected: usize) -> Result<(), QuicError> {
    if available < expected {
        return Err(QuicError::PayloadTooShort {
            expected,
            available,
        });
    }
    Ok(())
}

/// The Length field covers Packet Number + payload, so it must be at least
/// `packet_number_length`. Returns the payload length (Length - PN length).
pub fn validate_length_field(
    length_field: u64,
    packet_number_length: u8,
) -> Result<usize, QuicError> {
    (length_field as usize)
        .checked_sub(packet_number_length as usize)
        .ok_or(QuicError::LengthFieldTooSmall {
            length_field,
            pn_length: packet_number_length,
        })
}

/// Lit un Connection ID (longueur u8 + octets) sans copie, en vérifiant
/// les bornes avant chaque lecture.
pub(crate) fn read_cid<'a>(cur: &mut QuicCursor<'a>) -> Result<ConnectionId<'a>, QuicError> {
    let len = cur.take_u8()?;
    let bytes = cur.take(len as usize)?;
    Ok(ConnectionId { len, bytes })
}

/// Lit Length (varint), Packet Number puis le payload chiffré.
///
/// Vérifie d'abord tous les champs, puis remplit `length_field` et
/// `packet_number` dans le header seulement après la dernière validation
/// (« check puis place »), et retourne le payload emprunté
/// (Length - PN length octets).
///
/// L'ordre des vérifications (varint Length, octets du PN, cohérence
/// Length/PN, disponibilité du payload) est identique à la séquence
/// historique pour que les mêmes inputs produisent les mêmes erreurs.
pub(crate) fn read_pn_and_payload<'a>(
    cur: &mut QuicCursor<'a>,
    header: &mut QuicLongHeader<'a>,
) -> Result<&'a [u8], QuicError> {
    let length_field = cur.read_varint()?;

    // PN (1..=4 octets big-endian)
    let pn_raw = cur.take(header.pn_length as usize)?;
    let mut pn: u64 = 0;
    for &b in pn_raw {
        pn = (pn << 8) | (b as u64);
    }

    // Payload (length_field inclut PN + payload)
    let payload_len = validate_length_field(length_field, header.pn_length)?;
    validate_payload_available(cur.remaining(), payload_len)?;
    let payload = cur.take(payload_len)?;

    // Toutes les validations ont réussi : on place les valeurs.
    header.length_field = length_field;
    header.packet_number = Some(pn);
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_take_within_bounds() {
        let data = [1u8, 2, 3, 4];
        let mut cur = QuicCursor::new(&data);
        assert_eq!(cur.take(2).unwrap(), &[1, 2]);
        assert_eq!(cur.remaining(), 2);
        assert_eq!(cur.take(2).unwrap(), &[3, 4]);
        assert_eq!(cur.remaining(), 0);
    }

    #[test]
    fn test_cursor_take_out_of_bounds() {
        let data = [1u8, 2];
        let mut cur = QuicCursor::new(&data);
        assert_eq!(
            cur.take(3),
            Err(QuicError::Truncated {
                needed: 3,
                remaining: 2
            })
        );
        // A failed take must not advance the cursor.
        assert_eq!(cur.remaining(), 2);
    }

    #[test]
    fn test_cursor_take_u8() {
        let data = [0xABu8];
        let mut cur = QuicCursor::new(&data);
        assert_eq!(cur.take_u8(), Ok(0xAB));
        assert_eq!(
            cur.take_u8(),
            Err(QuicError::Truncated {
                needed: 1,
                remaining: 0
            })
        );
    }

    #[test]
    fn test_cursor_take_rest() {
        let data = [1u8, 2, 3];
        let mut cur = QuicCursor::new(&data);
        cur.take_u8().unwrap();
        assert_eq!(cur.take_rest(), &[2, 3]);
        assert_eq!(cur.remaining(), 0);
        assert_eq!(cur.take_rest(), &[] as &[u8]);
    }

    #[test]
    fn test_read_varint_all_lengths() {
        // 1 byte: 0x25 = 37
        let mut cur = QuicCursor::new(&[0x25]);
        assert_eq!(cur.read_varint(), Ok(37));

        // 2 bytes: 0x7BBD = 15293 (RFC 9000 §A.1 example)
        let mut cur = QuicCursor::new(&[0x7B, 0xBD]);
        assert_eq!(cur.read_varint(), Ok(15293));

        // 4 bytes: 0x9D7F3E7D = 494878333 (RFC 9000 §A.1 example)
        let mut cur = QuicCursor::new(&[0x9D, 0x7F, 0x3E, 0x7D]);
        assert_eq!(cur.read_varint(), Ok(494_878_333));

        // 8 bytes: 0xC2197C5EFF14E88C = 151288809941952652 (RFC 9000 §A.1 example)
        let mut cur = QuicCursor::new(&[0xC2, 0x19, 0x7C, 0x5E, 0xFF, 0x14, 0xE8, 0x8C]);
        assert_eq!(cur.read_varint(), Ok(151_288_809_941_952_652));
    }

    #[test]
    fn test_read_varint_truncated() {
        // Empty buffer: not even the first byte.
        let mut cur = QuicCursor::new(&[]);
        assert_eq!(
            cur.read_varint(),
            Err(QuicError::Truncated {
                needed: 1,
                remaining: 0
            })
        );

        // Prefix announces 2 bytes but no continuation byte follows.
        let mut cur = QuicCursor::new(&[0x40]);
        assert_eq!(
            cur.read_varint(),
            Err(QuicError::TruncatedVarint {
                needed: 1,
                remaining: 0
            })
        );

        // Prefix announces 8 bytes but only 3 continuation bytes follow.
        let mut cur = QuicCursor::new(&[0xC0, 0x01, 0x02, 0x03]);
        assert_eq!(
            cur.read_varint(),
            Err(QuicError::TruncatedVarint {
                needed: 7,
                remaining: 3
            })
        );
    }

    #[test]
    fn test_is_plausible_short_header() {
        // Premier octet valide (form=0, fixed=1) et taille suffisante.
        let mut packet = vec![0x4C];
        packet.extend_from_slice(&[0u8; QUIC_SHORT_HEADER_MIN_LEN - 1]);
        assert!(is_plausible_short_header(&packet));

        // Trop court d'un octet.
        assert!(!is_plausible_short_header(
            &packet[..QUIC_SHORT_HEADER_MIN_LEN - 1]
        ));

        // Long Header (form=1) : pas un Short Header.
        packet[0] = 0xC0;
        assert!(!is_plausible_short_header(&packet));

        // Fixed bit absent.
        packet[0] = 0x0C;
        assert!(!is_plausible_short_header(&packet));

        // Buffer vide.
        assert!(!is_plausible_short_header(&[]));
    }

    #[test]
    fn test_validate_long_header() {
        assert_eq!(validate_long_header(true), Ok(()));
        assert_eq!(validate_long_header(false), Err(QuicError::NotLongHeader));
    }

    #[test]
    fn test_validate_fixed_bit() {
        assert_eq!(validate_fixed_bit(true), Ok(()));
        assert_eq!(validate_fixed_bit(false), Err(QuicError::FixedBitNotSet));
    }

    #[test]
    fn test_validate_version() {
        assert_eq!(validate_version(QUIC_V1), Ok(()));
        assert_eq!(
            validate_version(0xff00_001d),
            Err(QuicError::UnsupportedVersion(0xff00_001d))
        );
    }

    #[test]
    fn test_validate_payload_available() {
        assert_eq!(validate_payload_available(10, 10), Ok(()));
        assert_eq!(validate_payload_available(10, 4), Ok(()));
        assert_eq!(
            validate_payload_available(3, 10),
            Err(QuicError::PayloadTooShort {
                expected: 10,
                available: 3
            })
        );
    }

    #[test]
    fn test_validate_length_field() {
        assert_eq!(validate_length_field(22, 1), Ok(21));
        assert_eq!(validate_length_field(4, 4), Ok(0));
        assert_eq!(
            validate_length_field(1, 2),
            Err(QuicError::LengthFieldTooSmall {
                length_field: 1,
                pn_length: 2
            })
        );
    }

    #[test]
    fn test_extract_first_byte_valid_types() {
        // 0xC3 : Long=1, Fixed=1, Initial (00), PN len code=3 -> 4 octets.
        assert_eq!(extract_first_byte(0xC3), Ok((QuicPacketType::Initial, 4)));
        // 0xD0 : 0-RTT, PN len 1.
        assert_eq!(extract_first_byte(0xD0), Ok((QuicPacketType::ZeroRtt, 1)));
        // 0xE1 : Handshake, PN len 2.
        assert_eq!(extract_first_byte(0xE1), Ok((QuicPacketType::Handshake, 2)));
        // 0xF2 : Retry, PN len 3.
        assert_eq!(extract_first_byte(0xF2), Ok((QuicPacketType::Retry, 3)));
    }

    #[test]
    fn test_extract_first_byte_error_precedence() {
        // form=0 et fixed=0 : NotLongHeader doit primer sur FixedBitNotSet.
        assert_eq!(extract_first_byte(0x00), Err(QuicError::NotLongHeader));
        // form=0, fixed=1 : Short Header.
        assert_eq!(extract_first_byte(0x40), Err(QuicError::NotLongHeader));
        // form=1, fixed=0.
        assert_eq!(extract_first_byte(0x80), Err(QuicError::FixedBitNotSet));
    }

    #[test]
    fn test_extract_version() {
        // Version v1, le curseur ne consomme que les 4 octets du champ.
        let mut cur = QuicCursor::new(&[0x00, 0x00, 0x00, 0x01, 0xFF]);
        assert_eq!(extract_version(&mut cur), Ok(1));
        assert_eq!(cur.remaining(), 1);

        // Version inconnue.
        let mut cur = QuicCursor::new(&[0x00, 0x00, 0x00, 0x02]);
        assert_eq!(
            extract_version(&mut cur),
            Err(QuicError::UnsupportedVersion(2))
        );

        // Tronqué : Truncated prime sur UnsupportedVersion.
        let mut cur = QuicCursor::new(&[0x00, 0x00]);
        assert_eq!(
            extract_version(&mut cur),
            Err(QuicError::Truncated {
                needed: 4,
                remaining: 2
            })
        );
    }

    #[test]
    fn test_read_cid() {
        let mut cur = QuicCursor::new(&[0x02, 0xAA, 0xBB, 0xCC]);
        let cid = read_cid(&mut cur).expect("CID valide");
        assert_eq!(cid.len, 2);
        assert_eq!(cid.bytes, &[0xAA, 0xBB]);
        assert_eq!(cur.remaining(), 1);

        let mut cur = QuicCursor::new(&[0x05, 0x01]);
        assert!(matches!(
            read_cid(&mut cur),
            Err(QuicError::Truncated {
                needed: 5,
                remaining: 1
            })
        ));
    }

    /// En-tête factice pour tester `read_pn_and_payload` isolément.
    fn dummy_header<'a>(pn_length: u8) -> QuicLongHeader<'a> {
        QuicLongHeader {
            header_form_long: true,
            fixed_bit: true,
            packet_type: QuicPacketType::Handshake,
            version: QUIC_V1,
            dcid: ConnectionId { len: 0, bytes: &[] },
            scid: ConnectionId { len: 0, bytes: &[] },
            pn_length,
            length_field: 0,
            packet_number: None,
        }
    }

    #[test]
    fn test_read_pn_and_payload_places_after_checks() {
        // length = 3 (PN(1) + payload(2)).
        let data = [0x03, 0x09, 0x01, 0x02];
        let mut cur = QuicCursor::new(&data);
        let mut header = dummy_header(1);
        let payload = read_pn_and_payload(&mut cur, &mut header).expect("valide");
        assert_eq!(payload, &[0x01, 0x02]);
        assert_eq!(header.length_field, 3);
        assert_eq!(header.packet_number, Some(9));
    }

    #[test]
    fn test_read_pn_and_payload_failure_leaves_header_untouched() {
        // length = 10 mais un seul octet disponible après le PN.
        let data = [0x0A, 0x00];
        let mut cur = QuicCursor::new(&data);
        let mut header = dummy_header(1);
        assert!(matches!(
            read_pn_and_payload(&mut cur, &mut header),
            Err(QuicError::PayloadTooShort { .. })
        ));
        assert_eq!(header.length_field, 0);
        assert_eq!(header.packet_number, None);
    }
}
