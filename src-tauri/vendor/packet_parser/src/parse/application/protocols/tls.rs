use std::convert::TryFrom;
use std::fmt;

/// Erreurs possibles lors du parsing d'un enregistrement TLS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsError {
    TooShort,
    InvalidContentType(u8),
    InvalidVersion { major: u8, minor: u8 },
    InconsistentLength { declared: u16, available: usize },
}

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
        if buf.len() < 5 {
            return Err(TlsError::TooShort);
        }

        let content_type = TlsContentType::try_from(buf[0])?;
        let version = TlsVersion::new(buf[1], buf[2])?;
        let length = u16::from_be_bytes([buf[3], buf[4]]);

        let header_len = 5usize;
        let available = buf.len().saturating_sub(header_len);

        if available < length as usize {
            return Err(TlsError::InconsistentLength {
                declared: length,
                available,
            });
        }

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

    while buf.len().saturating_sub(offset) >= 5 {
        let slice = &buf[offset..];

        match TlsPacket::try_from(slice) {
            Ok(packet) => {
                let record_total_len = 5 + packet.length as usize;
                if buf.len().saturating_sub(offset) < record_total_len {
                    // Record annoncé mais tronqué → on s'arrête, on ne le compte pas.
                    break;
                }

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
