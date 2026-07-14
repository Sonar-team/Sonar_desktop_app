// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::convert::TryFrom;
use std::fmt;

use crate::{
    checks::application::tls::{
        TLS_RECORD_HEADER_LEN, validate_tls_header_length, validate_tls_payload_length,
        validate_tls_record_complete,
    },
    errors::application::tls::TlsError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// TLS Record Packet
///
/// ```mermaid
/// ---
/// title: TlsPacket
/// ---
/// packet-beta
/// 0-7: "Content Type u8"
/// 8-23: "Protocol Version u16"
/// 24-39: "Length u16"
/// 40-103: "Payload variable"
/// ```
///
/// Représente un enregistrement TLS (TLS Record Layer).
#[derive(Debug)]
pub struct TlsPacket<'a> {
    pub content_type: TlsContentType,
    pub version: TlsVersion,
    pub length: u16,
    pub payload: &'a [u8],
}

impl fmt::Display for TlsPacket<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TLS Packet: content_type={}, version={}, length={}, payload={:02X?}",
            self.content_type, self.version, self.length, self.payload
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsContentType {
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
    ApplicationData = 23,
    Heartbeat = 24,
}

impl fmt::Display for TlsContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TlsContentType::ChangeCipherSpec => "ChangeCipherSpec",
            TlsContentType::Alert => "Alert",
            TlsContentType::Handshake => "Handshake",
            TlsContentType::ApplicationData => "ApplicationData",
            TlsContentType::Heartbeat => "Heartbeat",
        };
        write!(f, "{s}")
    }
}

impl TryFrom<u8> for TlsContentType {
    type Error = TlsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            20 => Ok(TlsContentType::ChangeCipherSpec),
            21 => Ok(TlsContentType::Alert),
            22 => Ok(TlsContentType::Handshake),
            23 => Ok(TlsContentType::ApplicationData),
            24 => Ok(TlsContentType::Heartbeat),
            _ => Err(TlsError::InvalidContentType(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TlsVersion {
    pub major: u8,
    pub minor: u8,
}

impl TlsVersion {
    pub fn new(major: u8, minor: u8) -> Result<Self, TlsError> {
        match (major, minor) {
            (3, 1) | // TLS 1.0
            (3, 2) | // TLS 1.1
            (3, 3) | // TLS 1.2 (utilisé aussi comme "legacy version" TLS 1.3)
            (3, 4)   // TLS 1.3 (si jamais tu le vois dans le record header)
                => Ok(Self { major, minor }),
            _ => Err(TlsError::InvalidVersion { major, minor }),
        }
    }
}

impl fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version_str = match (self.major, self.minor) {
            (3, 1) => "TLS 1.0",
            (3, 2) => "TLS 1.1",
            (3, 3) => "TLS 1.2",
            (3, 4) => "TLS 1.3",
            _ => return write!(f, "{}.{}", self.major, self.minor),
        };
        write!(f, "{version_str}")
    }
}

impl<'a> TryFrom<&'a [u8]> for TlsPacket<'a> {
    type Error = TlsError;

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        validate_tls_header_length(buf)?;

        let content_type = TlsContentType::try_from(buf[0])?;
        let version = TlsVersion::new(buf[1], buf[2])?;
        let length = u16::from_be_bytes([buf[3], buf[4]]);

        let header_len = TLS_RECORD_HEADER_LEN;
        let available = buf.len().saturating_sub(header_len);

        validate_tls_payload_length(length, available)?;

        let start = header_len;
        let end = start + length as usize;
        let payload = &buf[start..end];

        Ok(TlsPacket {
            content_type,
            version,
            length,
            payload,
        })
    }
}

/// Parse un ou plusieurs enregistrements TLS consécutifs dans `buf`.
///
/// - Retourne un `Vec<TlsPacket>` avec tous les records complets trouvés.
/// - S'arrête dès que :
///   - le header (5 octets) n'est plus disponible, ou
///   - la longueur annoncée dépasse la taille restante (record tronqué en fin de buffer), ou
///   - on rencontre quelque chose qui n'est manifestement pas du TLS.
pub fn parse_tls_records<'a>(buf: &'a [u8]) -> Vec<TlsPacket<'a>> {
    let mut records = Vec::new();
    let mut offset = 0usize;

    while buf.len().saturating_sub(offset) >= TLS_RECORD_HEADER_LEN {
        let slice = &buf[offset..];

        match TlsPacket::try_from(slice) {
            Ok(packet) => {
                let remaining = buf.len().saturating_sub(offset);
                if validate_tls_record_complete(remaining, packet.length).is_err() {
                    // Record annoncé mais tronqué → on s'arrête, on ne le compte pas.
                    break;
                }

                let record_total_len = TLS_RECORD_HEADER_LEN + packet.length as usize;

                // On garde le packet (avec des slices dans le buffer d'origine).
                records.push(packet);

                // On avance à l'enregistrement TLS suivant.
                offset += record_total_len;
            }
            Err(TlsError::TooShort) => {
                // Plus assez de données pour un header complet → on s'arrête.
                break;
            }
            Err(TlsError::InconsistentLength { .. }) => {
                // Longueur incohérente -> soit tronqué, soit pas du TLS → on s'arrête.
                break;
            }
            Err(_) => {
                // InvalidContentType / InvalidVersion → probablement pas (ou plus) du TLS.
                break;
            }
        }
    }

    records
}

/// Helper : détection simple "est-ce que ça ressemble à du TLS ?"
///
/// Utile si tu veux juste classifier un flux comme TLS/Non-TLS.
pub fn looks_like_tls(buf: &[u8]) -> bool {
    TlsPacket::try_from(buf).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_parse_valid_tls_packet() {
        // Handshake, TLS 1.2, length 5, payload = [1,2,3,4,5]
        let tls_payload = vec![22, 3, 3, 0, 5, 1, 2, 3, 4, 5];

        let packet = TlsPacket::try_from(tls_payload.as_slice()).expect("Expected TLS packet");

        assert_eq!(packet.content_type, TlsContentType::Handshake);
        assert_eq!(packet.version, TlsVersion { major: 3, minor: 3 });
        assert_eq!(packet.length, 5);
        assert_eq!(packet.payload, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_invalid_content_type() {
        let invalid = vec![99, 3, 3, 0, 5, 1, 2, 3, 4, 5];
        let err = TlsPacket::try_from(invalid.as_slice()).unwrap_err();
        assert!(matches!(err, TlsError::InvalidContentType(99)));
    }

    #[test]
    fn test_invalid_tls_version() {
        // Handshake, version 3.9 (invalide)
        let invalid = vec![22, 3, 9, 0, 5, 1, 2, 3, 4, 5];
        let err = TlsPacket::try_from(invalid.as_slice()).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InvalidVersion { major: 3, minor: 9 }
        ));
    }

    #[test]
    fn test_inconsistent_length() {
        // Handshake, TLS 1.2, length 6 mais seulement 5 octets de payload
        let invalid = vec![22, 3, 3, 0, 6, 1, 2, 3, 4, 5];
        let err = TlsPacket::try_from(invalid.as_slice()).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InconsistentLength {
                declared: 6,
                available: 5
            }
        ));
    }

    #[test]
    fn test_too_short() {
        // 4 octets seulement
        let short = vec![22, 3, 3, 0];
        let err = TlsPacket::try_from(short.as_slice()).unwrap_err();
        assert!(matches!(err, TlsError::TooShort));
    }

    #[test]
    fn test_parse_multiple_tls_records_in_one_buffer() {
        // Record 1 : ChangeCipherSpec, TLS 1.2, length 1, payload = [0x00]
        // Record 2 : ApplicationData, TLS 1.2, length 3, payload = [0x01,0x02,0x03]
        let buf = vec![
            20, 3, 3, 0, 1, 0x00, // CCS
            23, 3, 3, 0, 3, 0x01, 0x02, 0x03, // AppData
        ];

        let records = parse_tls_records(&buf);
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].content_type, TlsContentType::ChangeCipherSpec);
        assert_eq!(records[0].version, TlsVersion { major: 3, minor: 3 });
        assert_eq!(records[0].length, 1);
        assert_eq!(records[0].payload, &[0x00]);

        assert_eq!(records[1].content_type, TlsContentType::ApplicationData);
        assert_eq!(records[1].version, TlsVersion { major: 3, minor: 3 });
        assert_eq!(records[1].length, 3);
        assert_eq!(records[1].payload, &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_parse_tls_records_truncated_last_record() {
        // Record complet puis record tronqué
        // Record 1 : ApplicationData, length 2, payload [0xAA, 0xBB]
        // Record 2 : ApplicationData, length 4, mais seulement 1 octet de payload (tronqué)
        let buf = vec![
            23, 3, 3, 0, 2, 0xAA, 0xBB, // record 1 complet
            23, 3, 3, 0, 4, 0xCC, // record 2 incomplet
        ];

        let records = parse_tls_records(&buf);

        // On doit récupérer uniquement le premier record, le deuxième est tronqué
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content_type, TlsContentType::ApplicationData);
        assert_eq!(records[0].length, 2);
        assert_eq!(records[0].payload, &[0xAA, 0xBB]);
    }

    #[test]
    fn test_parse_tls_records_non_tls_content() {
        // Premier octet = 0x01 -> content type invalide
        let buf = vec![1, 3, 3, 0, 5, 0, 0, 0, 0, 0];

        let records = parse_tls_records(&buf);
        // On ne doit rien parser, on considère que ce n'est pas du TLS.
        assert!(records.is_empty());
    }

    #[test]
    fn test_parse_tls_records_header_too_short_at_end() {
        // Record valide, suivi de 4 octets "résiduels" (< 5 octets pour un header)
        let buf = vec![
            22, 3, 3, 0, 1, 0x01, // Handshake, length 1
            0x23, 0x00, 0x00, 0x00, // 4 octets, pas assez pour un header complet
        ];

        let records = parse_tls_records(&buf);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content_type, TlsContentType::Handshake);
        assert_eq!(records[0].payload, &[0x01]);
    }

    // --- Tests pour looks_like_tls ---

    #[test]
    fn test_looks_like_tls_when_true() {
        let tls_buf = vec![22, 3, 3, 0, 2, 0xAA, 0xBB];
        assert!(looks_like_tls(&tls_buf));
    }

    #[test]
    fn test_looks_like_tls_when_false_invalid_content_type() {
        let non_tls = vec![0, 3, 3, 0, 2, 0xAA, 0xBB];
        assert!(!looks_like_tls(&non_tls));
    }

    #[test]
    fn test_looks_like_tls_when_false_too_short() {
        let too_short = vec![22, 3, 3, 0]; // 4 octets seulement
        assert!(!looks_like_tls(&too_short));
    }

    // --- Golden tests : trame réelle TLSv1.3 ---
    //
    // ClientHello TLSv1.3 réel (trame 199 capturée sur eno1 vers
    // unleash.codeium.com:443), dissection tshark complète documentée en fin
    // de fichier. Le record layer annonce la version legacy 0x0301 (TLS 1.0),
    // la vraie version se négocie dans l'extension supported_versions (0x0304).
    const TLS13_CLIENT_HELLO_RECORD_HEX: &str = concat!(
        "1603010200010001fc0303900986c5d29c4072ed85dec8067e2dd2cd3e8f3ee763e4ae",
        "030986410e5b1e8d20046d7d07df148587017273a2b93bfd1f061ffc3a42066ce3bfcc",
        "ced6f2f7db2e0024130113021303c02fc02bc030c02cc027cca9cca8c009c013c00ac0",
        "14009c009d002f00350100018f000000180016000013756e6c656173682e636f646569",
        "756d2e636f6d00170000ff01000100000a00080006001d00170018000b000201000023",
        "0000000d00140012040308040401050308050501080606010201003300260024001d00",
        "2019e36da4275dad5fe69c13a2c7cd81991f0d4bd0fdfe0d7daa390876845db21b002d",
        "00020101002b00050403040303001500a0000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000000000",
        "0000000000000000000000000000000000000000000000000000000000000000000000",
        "00000029005b00260020432e29c0ce0e79f70e48238f7d619ec9e7e9c9def0f8df4b53",
        "c965d4935c443700a48f55003130966f759b98683dd7d866812f9e8d5af8ea8ad65045",
        "e20ec0e0f0d1af9b01b376c0b9d2c31667cb1dbd67bac24ccd500a",
    );

    fn tls13_client_hello_record() -> Vec<u8> {
        hex::decode(TLS13_CLIENT_HELLO_RECORD_HEX).expect("valid golden hex fixture")
    }

    #[test]
    fn golden_tls13_client_hello_record_header() {
        let buf = tls13_client_hello_record();

        let packet = TlsPacket::try_from(buf.as_slice()).expect("golden TLS 1.3 record");

        assert_eq!(packet.content_type, TlsContentType::Handshake);
        // Legacy version du record layer : 0x0301, comme dissequé par tshark.
        assert_eq!(packet.version, TlsVersion { major: 3, minor: 1 });
        assert_eq!(packet.version.to_string(), "TLS 1.0");
        assert_eq!(packet.length, 512);
        assert_eq!(packet.payload.len(), 512);
    }

    #[test]
    fn golden_tls13_client_hello_handshake_payload() {
        let buf = tls13_client_hello_record();
        let packet = TlsPacket::try_from(buf.as_slice()).expect("golden TLS 1.3 record");
        let payload = packet.payload;

        // Handshake Type: Client Hello (1), Length: 508.
        assert_eq!(payload[0], 1);
        assert_eq!(payload[1..4], [0x00, 0x01, 0xfc]);
        // Version legacy du ClientHello : TLS 1.2 (0x0303).
        assert_eq!(payload[4..6], [0x03, 0x03]);
        // Random.
        assert_eq!(
            payload[6..38],
            hex::decode("900986c5d29c4072ed85dec8067e2dd2cd3e8f3ee763e4ae030986410e5b1e8d")
                .unwrap()[..]
        );
        // Session ID Length: 32.
        assert_eq!(payload[38], 32);
        // Cipher Suites Length: 36, la première étant TLS_AES_128_GCM_SHA256.
        assert_eq!(payload[71..73], [0x00, 0x24]);
        assert_eq!(payload[73..75], [0x13, 0x01]);
        // SNI : unleash.codeium.com.
        assert!(
            payload
                .windows(b"unleash.codeium.com".len())
                .any(|w| w == b"unleash.codeium.com"),
            "SNI attendu dans le ClientHello"
        );
        // Extension supported_versions (43) : TLS 1.3 (0x0304) puis TLS 1.2
        // (0x0303) — c'est elle qui fait de cette trame un ClientHello TLSv1.3.
        let supported_versions = [0x00, 0x2b, 0x00, 0x05, 0x04, 0x03, 0x04, 0x03, 0x03];
        assert!(
            payload
                .windows(supported_versions.len())
                .any(|w| w == supported_versions),
            "extension supported_versions TLS 1.3 attendue"
        );
    }

    #[test]
    fn golden_tls13_client_hello_is_a_single_complete_record() {
        let buf = tls13_client_hello_record();

        assert!(looks_like_tls(&buf));

        let records = parse_tls_records(&buf);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content_type, TlsContentType::Handshake);
        assert_eq!(records[0].length, 512);
        assert_eq!(records[0].payload, &buf[TLS_RECORD_HEADER_LEN..]);
    }

    #[test]
    fn golden_tls13_client_hello_truncated_is_rejected() {
        let buf = tls13_client_hello_record();
        let truncated = &buf[..buf.len() - 1];

        let err = TlsPacket::try_from(truncated).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InconsistentLength {
                declared: 512,
                available: 511
            }
        ));
        assert!(parse_tls_records(truncated).is_empty());
    }

    // --- Tests sur les types et versions ---

    #[test]
    fn test_tls_content_type_from_u8_all_valid_values() {
        for (value, expected) in [
            (20u8, TlsContentType::ChangeCipherSpec),
            (21, TlsContentType::Alert),
            (22, TlsContentType::Handshake),
            (23, TlsContentType::ApplicationData),
            (24, TlsContentType::Heartbeat),
        ] {
            let ct = TlsContentType::try_from(value).unwrap();
            assert_eq!(ct, expected);
        }
    }

    #[test]
    fn test_tls_content_type_from_u8_invalid_value() {
        let err = TlsContentType::try_from(0xFF).unwrap_err();
        assert!(matches!(err, TlsError::InvalidContentType(0xFF)));
    }

    #[test]
    fn test_tls_version_new_valid_versions() {
        for (maj, min) in [(3, 1), (3, 2), (3, 3), (3, 4)] {
            let v = TlsVersion::new(maj, min).expect("valid version");
            assert_eq!(v.major, maj);
            assert_eq!(v.minor, min);
        }
    }

    #[test]
    fn test_tls_version_new_invalid_version() {
        let err = TlsVersion::new(3, 0).unwrap_err();
        assert!(matches!(
            err,
            TlsError::InvalidVersion { major: 3, minor: 0 }
        ));
    }
}

// 44152420a564e0d55e289bd40800450002398fa340004006d42cc0a801b523dfeeb2c40a01bb462b3ca1cdac346c8018003fd71a00000101080a7032e8b2c27b69501603010200010001fc0303900986c5d29c4072ed85dec8067e2dd2cd3e8f3ee763e4ae030986410e5b1e8d20046d7d07df148587017273a2b93bfd1f061ffc3a42066ce3bfccced6f2f7db2e0024130113021303c02fc02bc030c02cc027cca9cca8c009c013c00ac014009c009d002f00350100018f000000180016000013756e6c656173682e636f646569756d2e636f6d00170000ff01000100000a00080006001d00170018000b0002010000230000000d00140012040308040401050308050501080606010201003300260024001d002019e36da4275dad5fe69c13a2c7cd81991f0d4bd0fdfe0d7daa390876845db21b002d00020101002b00050403040303001500a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029005b00260020432e29c0ce0e79f70e48238f7d619ec9e7e9c9def0f8df4b53c965d4935c443700a48f55003130966f759b98683dd7d866812f9e8d5af8ea8ad65045e20ec0e0f0d1af9b01b376c0b9d2c31667cb1dbd67bac24ccd500a
// tlsv3 part:
// 1603010200010001fc0303900986c5d29c4072ed85dec8067e2dd2cd3e8f3ee763e4ae030986410e5b1e8d20046d7d07df148587017273a2b93bfd1f061ffc3a42066ce3bfccced6f2f7db2e0024130113021303c02fc02bc030c02cc027cca9cca8c009c013c00ac014009c009d002f00350100018f000000180016000013756e6c656173682e636f646569756d2e636f6d00170000ff01000100000a00080006001d00170018000b0002010000230000000d00140012040308040401050308050501080606010201003300260024001d002019e36da4275dad5fe69c13a2c7cd81991f0d4bd0fdfe0d7daa390876845db21b002d00020101002b00050403040303001500a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000029005b00260020432e29c0ce0e79f70e48238f7d619ec9e7e9c9def0f8df4b53c965d4935c443700a48f55003130966f759b98683dd7d866812f9e8d5af8ea8ad65045e20ec0e0f0d1af9b01b376c0b9d2c31667cb1dbd67bac24ccd500a
// Frame 199: Packet, 583 bytes on wire (4664 bits), 583 bytes captured (4664 bits) on interface eno1, id 0
// Ethernet II, Src: GigaByteTech_28:9b:d4 (e0:d5:5e:28:9b:d4), Dst: SagemcomBroa_20:a5:64 (44:15:24:20:a5:64)
// Internet Protocol Version 4, Src: 192.168.1.181, Dst: 35.223.238.178
// Transmission Control Protocol, Src Port: 50186, Dst Port: 443, Seq: 1, Ack: 1, Len: 517
// Transport Layer Security
//     [Stream index: 5]
//     TLSv1.3 Record Layer: Handshake Protocol: Client Hello
//         Content Type: Handshake (22)
//         Version: TLS 1.0 (0x0301)
//         Length: 512
//         Handshake Protocol: Client Hello
//             Handshake Type: Client Hello (1)
//             Length: 508
//             Version: TLS 1.2 (0x0303)
//                 [Expert Info (Chat/Deprecated): This legacy_version field MUST be ignored. The supported_versions extension is present and MUST be used instead.]
//                     [This legacy_version field MUST be ignored. The supported_versions extension is present and MUST be used instead.]
//                     [Severity level: Chat]
//                     [Group: Deprecated]
//             Random: 900986c5d29c4072ed85dec8067e2dd2cd3e8f3ee763e4ae030986410e5b1e8d
//             Session ID Length: 32
//             Session ID: 046d7d07df148587017273a2b93bfd1f061ffc3a42066ce3bfccced6f2f7db2e
//             Cipher Suites Length: 36
//             Cipher Suites (18 suites)
//                 Cipher Suite: TLS_AES_128_GCM_SHA256 (0x1301)
//                 Cipher Suite: TLS_AES_256_GCM_SHA384 (0x1302)
//                 Cipher Suite: TLS_CHACHA20_POLY1305_SHA256 (0x1303)
//                 Cipher Suite: TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 (0xc02f)
//                 Cipher Suite: TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 (0xc02b)
//                 Cipher Suite: TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 (0xc030)
//                 Cipher Suite: TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 (0xc02c)
//                 Cipher Suite: TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256 (0xc027)
//                 Cipher Suite: TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 (0xcca9)
//                 Cipher Suite: TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 (0xcca8)
//                 Cipher Suite: TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA (0xc009)
//                 Cipher Suite: TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA (0xc013)
//                 Cipher Suite: TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA (0xc00a)
//                 Cipher Suite: TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA (0xc014)
//                 Cipher Suite: TLS_RSA_WITH_AES_128_GCM_SHA256 (0x009c)
//                 Cipher Suite: TLS_RSA_WITH_AES_256_GCM_SHA384 (0x009d)
//                 Cipher Suite: TLS_RSA_WITH_AES_128_CBC_SHA (0x002f)
//                 Cipher Suite: TLS_RSA_WITH_AES_256_CBC_SHA (0x0035)
//             Compression Methods Length: 1
//             Compression Methods (1 method)
//                 Compression Method: null (0)
//             Extensions Length: 399
//             Extension: server_name (len=24) name=unleash.codeium.com
//                 Type: server_name (0)
//                 Length: 24
//                 Server Name Indication extension
//                     Server Name list length: 22
//                     Server Name Type: host_name (0)
//                     Server Name length: 19
//                     Server Name: unleash.codeium.com
//             Extension: extended_master_secret (len=0)
//                 Type: extended_master_secret (23)
//                 Length: 0
//             Extension: renegotiation_info (len=1)
//                 Type: renegotiation_info (65281)
//                 Length: 1
//                 Renegotiation Info extension
//                     Renegotiation info extension length: 0
//             Extension: supported_groups (len=8)
//                 Type: supported_groups (10)
//                 Length: 8
//                 Supported Groups List Length: 6
//                 Supported Groups (3 groups)
//                     Supported Group: x25519 (0x001d)
//                     Supported Group: secp256r1 (0x0017)
//                     Supported Group: secp384r1 (0x0018)
//             Extension: ec_point_formats (len=2)
//                 Type: ec_point_formats (11)
//                 Length: 2
//                 EC point formats Length: 1
//                 Elliptic curves point formats (1)
//                     EC point format: uncompressed (0)
//             Extension: session_ticket (len=0)
//                 Type: session_ticket (35)
//                 Length: 0
//                 Session Ticket: <MISSING>
//             Extension: signature_algorithms (len=20)
//                 Type: signature_algorithms (13)
//                 Length: 20
//                 Signature Hash Algorithms Length: 18
//                 Signature Hash Algorithms (9 algorithms)
//                     Signature Algorithm: ecdsa_secp256r1_sha256 (0x0403)
//                         Signature Hash Algorithm Hash: SHA256 (4)
//                         Signature Hash Algorithm Signature: ECDSA (3)
//                     Signature Algorithm: rsa_pss_rsae_sha256 (0x0804)
//                         Signature Hash Algorithm Hash: Unknown (8)
//                         Signature Hash Algorithm Signature: Unknown (4)
//                     Signature Algorithm: rsa_pkcs1_sha256 (0x0401)
//                         Signature Hash Algorithm Hash: SHA256 (4)
//                         Signature Hash Algorithm Signature: RSA (1)
//                     Signature Algorithm: ecdsa_secp384r1_sha384 (0x0503)
//                         Signature Hash Algorithm Hash: SHA384 (5)
//                         Signature Hash Algorithm Signature: ECDSA (3)
//                     Signature Algorithm: rsa_pss_rsae_sha384 (0x0805)
//                         Signature Hash Algorithm Hash: Unknown (8)
//                         Signature Hash Algorithm Signature: Unknown (5)
//                     Signature Algorithm: rsa_pkcs1_sha384 (0x0501)
//                         Signature Hash Algorithm Hash: SHA384 (5)
//                         Signature Hash Algorithm Signature: RSA (1)
//                     Signature Algorithm: rsa_pss_rsae_sha512 (0x0806)
//                         Signature Hash Algorithm Hash: Unknown (8)
//                         Signature Hash Algorithm Signature: Unknown (6)
//                     Signature Algorithm: rsa_pkcs1_sha512 (0x0601)
//                         Signature Hash Algorithm Hash: SHA512 (6)
//                         Signature Hash Algorithm Signature: RSA (1)
//                     Signature Algorithm: rsa_pkcs1_sha1 (0x0201)
//                         Signature Hash Algorithm Hash: SHA1 (2)
//                         Signature Hash Algorithm Signature: RSA (1)
//             Extension: key_share (len=38) x25519
//                 Type: key_share (51)
//                 Length: 38
//                 Key Share extension
//                     Client Key Share Length: 36
//                     Key Share Entry: Group: x25519, Key Exchange length: 32
//                         Group: x25519 (29)
//                         Key Exchange Length: 32
//                         Key Exchange: 19e36da4275dad5fe69c13a2c7cd81991f0d4bd0fdfe0d7daa390876845db21b
//             Extension: psk_key_exchange_modes (len=2)
//                 Type: psk_key_exchange_modes (45)
//                 Length: 2
//                 PSK Key Exchange Modes Length: 1
//                 PSK Key Exchange Mode: PSK with (EC)DHE key establishment (psk_dhe_ke) (1)
//             Extension: supported_versions (len=5) TLS 1.3, TLS 1.2
//                 Type: supported_versions (43)
//                 Length: 5
//                 Supported Versions length: 4
//                 Supported Version: TLS 1.3 (0x0304)
//                 Supported Version: TLS 1.2 (0x0303)
//             Extension: padding (len=160)
//                 Type: padding (21)
//                 Length: 160
//                 Padding Data […]: 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
//             Extension: pre_shared_key (len=91)
//                 Type: pre_shared_key (41)
//                 Length: 91
//                 Pre-Shared Key extension
//                     Identities Length: 38
//                     PSK Identity (length: 32)
//                         Identity Length: 32
//                         Identity: 432e29c0ce0e79f70e48238f7d619ec9e7e9c9def0f8df4b53c965d4935c4437
//                         Obfuscated Ticket Age: 10784597
//                     PSK Binders length: 49
//                     PSK Binders
//                         PSK Binder (length: 48)
//                             Binder Length: 48
//                             Binder: 966f759b98683dd7d866812f9e8d5af8ea8ad65045e20ec0e0f0d1af9b01b376c0b9d2c31667cb1dbd67bac24ccd500a
//             [JA4: t13d181200_5d04281c6031_02c8e53ee398]
//             [JA4_r: t13d181200_002f,0035,009c,009d,1301,1302,1303,c009,c00a,c013,c014,c027,c02b,c02c,c02f,c030,cca8,cca9_000a,000b,000d,0015,0017,0023,0029,002b,002d,0033,ff01_0403,0804,0401,0503,0805,0501,0806,0601,0201]
//             [JA3 Fullstring: 771,4865-4866-4867-49199-49195-49200-49196-49191-52393-52392-49161-49171-49162-49172-156-157-47-53,0-23-65281-10-11-35-13-51-45-43-21-41,29-23-24,0]
//             [JA3: d92981146534550ae85075b70b1c352a]
