/// Modélisation minimale d'un paquet QUIC v1 (RFC 9000/9001) avec Long Header:
/// couvre `Initial` et `Handshake`, ainsi que quelques frames fréquentes.
///
/// Remarques :
/// - Un paquet `Initial` peut contenir un Token.
/// - Un paquet `Handshake` n'a pas de Token.
/// - Le champ `length` du Long Header inclut PN + payload chiffré (frames).
/// - Le `packet_number` est encodé sur 1..=4 octets ; on expose ici la longueur et la valeur étendue.
/// - Les frames peuvent rester chiffrées selon le contexte ; si tu ne déchiffres pas,
///   utilise `QuicFrame::EncryptedPayload(Vec<u8>)`.
#[derive(Debug, Clone)]
pub enum QuicPacket {
    /// Paquet QUIC avec Long Header de type Initial (Packet Type = 0x00)
    Initial {
        /// Entête Long Header commun.
        header: QuicLongHeader,
        /// Jeton fourni/retourné par le serveur (anti-DoS, address validation).
        token: Vec<u8>,
        /// Liste des frames décodées (si déchiffrement réussi) ou charge brute.
        payload: QuicPayload,
    },
    /// Paquet QUIC avec Long Header de type Handshake (Packet Type = 0x02)
    Handshake {
        /// Entête Long Header commun.
        header: QuicLongHeader,
        /// Liste des frames décodées (si déchiffrement réussi) ou charge brute.
        payload: QuicPayload,
    },
    /// Autres Long Headers (0-RTT, Retry) si besoin plus tard.
    OtherLong {
        header: QuicLongHeader,
        payload: QuicPayload,
    },
}

/// Entête commun aux paquets QUIC à Long Header.
#[derive(Debug, Clone)]
pub struct QuicLongHeader {
    /// Doit valoir 1 pour Long Header.
    pub header_form_long: bool,
    /// Bit fixé à 1 par la spec.
    pub fixed_bit: bool,
    /// Type de paquet (Initial, 0-RTT, Handshake, Retry).
    pub packet_type: QuicPacketType,
    /// Version QUIC (ex: 0x00000001 pour QUIC v1).
    pub version: u32,
    /// Destination Connection ID (DCID).
    pub dcid: ConnectionId,
    /// Source Connection ID (SCID).
    pub scid: ConnectionId,
    /// Longueur du champ `packet_number` en octets (1..=4).
    pub pn_length: u8,
    /// Longueur (varint) annoncée pour PN + payload chiffré.
    pub length_field: u64,
    /// Numéro de paquet reconstruit (valeur étendue), si PN decoding dispo.
    pub packet_number: Option<u64>,
}

/// Type de paquet Long Header.
#[derive(Debug, Clone, Copy)]
pub enum QuicPacketType {
    Initial,
    ZeroRtt,
    Handshake,
    Retry,
    Unknown(u8),
}

/// Connection ID générique (0..=20 octets courants, mais extensible).
#[derive(Debug, Clone)]
pub struct ConnectionId {
    /// Longueur du CID (0..=20 dans ta capture).
    pub len: u8,
    /// Octets du CID.
    pub bytes: Vec<u8>,
}

/// Charge utile d’un paquet QUIC une fois l’entête parsé.
#[derive(Debug, Clone)]
pub enum QuicPayload {
    /// Ensemble de frames décodées (après déchiffrement).
    Frames(Vec<QuicFrame>),
    /// Payload encore chiffré ou non interprété.
    EncryptedPayload(Vec<u8>),
}

/// Ensemble minimal de frames QUIC utiles au handshake.
#[derive(Debug, Clone)]
pub enum QuicFrame {
    /// Frame ACK (RFC 9000 §19.3)
    Ack(AckFrame),
    /// Frame CRYPTO (RFC 9001 §4) — transporte les enregistrements TLS 1.3.
    Crypto(CryptoFrame),
    /// PADDING (0x00)
    Padding { length: u64 },
    /// PING (0x01)
    Ping,
    /// Autre type de frame non gérée ici.
    Unknown { frame_type: u64, raw: Vec<u8> },
}

/// Frame ACK (schéma simplifié : first range + ranges supplémentaires).
#[derive(Debug, Clone)]
pub struct AckFrame {
    /// Plus grand numéro de paquet accusé de réception.
    pub largest_acknowledged: u64,
    /// Délai d’ACK en microsecondes (valeur QUIC encodée en exponentiel; ici post-décodée).
    pub ack_delay_us: u64,
    /// Nombre de ranges additionnels (peut être 0).
    pub ack_range_count: u64,
    /// Premier intervalle (taille du range commençant à `largest_acknowledged`).
    pub first_ack_range: u64,
    /// Ranges additionnels : (gap, ack_range_len)
    pub additional_ranges: Vec<AckRange>,
}

/// Un intervalle d’ACK supplémentaire (gap + longueur du range).
#[derive(Debug, Clone)]
pub struct AckRange {
    /// Ecart (paquets non accusés) avant le prochain range.
    pub gap: u64,
    /// Taille du range suivant.
    pub ack_range_len: u64,
}

/// Frame CRYPTO : transporte des fragments TLS 1.3 (ClientHello, ServerHello, etc.).
#[derive(Debug, Clone)]
pub struct CryptoFrame {
    /// Offset dans le flux CRYPTO (peut arriver en fragments).
    pub offset: u64,
    /// Taille des données (facultatif si `data.len()` suffit).
    pub length: u64,
    /// Données TLS (potentiellement fragmentées) — par ex. ServerHello, EncryptedExtensions…
    pub data: Vec<u8>,
}

// Helper cursor for parsing byte slices
struct Cur<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Cur<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self { b, i: 0 }
    }

    fn left(&self) -> usize {
        self.b.len().saturating_sub(self.i)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], crate::parse::application::ApplicationError> {
        if self.left() < n {
            return Err(crate::parse::application::ApplicationError::QuicParseError);
        }
        let s = &self.b[self.i..self.i + n];
        self.i += n;
        Ok(s)
    }

    fn take_u8(&mut self) -> Result<u8, crate::parse::application::ApplicationError> {
        Ok(*self.take(1)?.first().unwrap())
    }
}

// Implement Clone for Cur to allow cloning the cursor
impl<'a> Clone for Cur<'a> {
    fn clone(&self) -> Self {
        Self {
            b: self.b,
            i: self.i,
        }
    }
}

impl TryFrom<&[u8]> for QuicPacket {
    type Error = crate::parse::application::ApplicationError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        // Helper function to create errors
        fn err<S: Into<String>>(_s: S) -> crate::parse::application::ApplicationError {
            crate::parse::application::ApplicationError::QuicParseError
        }

        // QUIC varint (RFC 9000 §16)
        fn read_varint(cur: &mut Cur) -> Result<u64, crate::parse::application::ApplicationError> {
            let first = cur.take_u8()?;
            let prefix = first >> 6; // 00, 01, 10, 11
            let (total, mask): (usize, u64) = match prefix {
                0 => (1, 0b0011_1111),
                1 => (2, 0b0011_1111),
                2 => (4, 0b0011_1111),
                3 => (8, 0b0011_1111),
                _ => unreachable!(),
            };
            let mut val = (first & (mask as u8)) as u64;
            for _ in 1..total {
                val = (val << 8) | (cur.take_u8()? as u64);
            }
            Ok(val)
        }

        fn read_cid(
            cur: &mut Cur,
        ) -> Result<ConnectionId, crate::parse::application::ApplicationError> {
            let len = cur.take_u8()?;
            let bytes = cur.take(len as usize)?.to_vec();
            Ok(ConnectionId { len, bytes })
        }

        // --- Parsing --------------------------------------------------------
        let mut cur = Cur::new(buf);

        // 1) Octet 0 : Long Header bits
        let b0 = cur.take_u8()?;
        let header_form_long = (b0 & 0b1000_0000) != 0;
        if !header_form_long {
            return Err(err(
                "Not a QUIC Long Header (short header not supported here)",
            ));
        }
        let fixed_bit = (b0 & 0b0100_0000) != 0;
        if !fixed_bit {
            return Err(err("Fixed bit must be 1 for QUIC v1 Long Header"));
        }
        let lptype = (b0 >> 4) & 0b11; // Long Packet Type (2 bits)
        let _reserved = (b0 >> 2) & 0b11; // reserved
        let pn_len_code = b0 & 0b11; // PN length code
        let pn_length = pn_len_code + 1; // 1..=4

        let packet_type = match lptype {
            0 => QuicPacketType::Initial,
            1 => QuicPacketType::ZeroRtt,
            2 => QuicPacketType::Handshake,
            3 => QuicPacketType::Retry,
            x => QuicPacketType::Unknown(x),
        };

        // 2) Version
        let ver_bytes = cur.take(4)?;
        let version = u32::from_be_bytes([ver_bytes[0], ver_bytes[1], ver_bytes[2], ver_bytes[3]]);

        // 3) DCID / SCID
        let dcid = read_cid(&mut cur)?;
        let scid = read_cid(&mut cur)?;

        // 4) En-tête commun assemblé (length_field/pn/… à compléter après)
        let mut header = QuicLongHeader {
            header_form_long,
            fixed_bit,
            packet_type,
            version,
            dcid,
            scid,
            pn_length,
            length_field: 0,     // placeholder, on remplit après
            packet_number: None, // idem
        };

        // 5) Champs spécifiques selon type
        match packet_type {
            QuicPacketType::Initial => {
                // Token Length (varint) + Token
                let token_len = read_varint(&mut cur)? as usize;
                let token = cur.take(token_len)?.to_vec();

                // Length (varint) = PN + payload chiffré
                let length_field = read_varint(&mut cur)?;
                header.length_field = length_field;

                // PN
                let pn_raw = cur.take(header.pn_length as usize)?;
                let mut pn: u64 = 0;
                for &b in pn_raw {
                    pn = (pn << 8) | (b as u64);
                }
                header.packet_number = Some(pn);

                // Payload (length_field inclut PN + payload)
                let remaining_for_payload = length_field as usize - (header.pn_length as usize);
                if cur.left() < remaining_for_payload {
                    return Err(err("Truncated payload for Initial packet"));
                }
                let payload_bytes = cur.take(remaining_for_payload)?.to_vec();

                Ok(QuicPacket::Initial {
                    header,
                    token,
                    payload: QuicPayload::EncryptedPayload(payload_bytes),
                })
            }

            QuicPacketType::Handshake => {
                // Pas de Token, directement Length (varint)
                let length_field = read_varint(&mut cur)?;
                header.length_field = length_field;

                // PN
                let pn_raw = cur.take(header.pn_length as usize)?;
                let mut pn: u64 = 0;
                for &b in pn_raw {
                    pn = (pn << 8) | (b as u64);
                }
                header.packet_number = Some(pn);

                // Payload
                let remaining_for_payload = length_field as usize - (header.pn_length as usize);
                if cur.left() < remaining_for_payload {
                    return Err(err("Truncated payload for Handshake packet"));
                }
                let payload_bytes = cur.take(remaining_for_payload)?.to_vec();

                Ok(QuicPacket::Handshake {
                    header,
                    payload: QuicPayload::EncryptedPayload(payload_bytes),
                })
            }

            QuicPacketType::ZeroRtt => {
                // Length (varint), PN, payload (non déchiffré ici)
                let length_field = read_varint(&mut cur)?;
                header.length_field = length_field;

                let pn_raw = cur.take(header.pn_length as usize)?;
                let mut pn: u64 = 0;
                for &b in pn_raw {
                    pn = (pn << 8) | (b as u64);
                }
                header.packet_number = Some(pn);

                let remaining_for_payload = length_field as usize - (header.pn_length as usize);
                if cur.left() < remaining_for_payload {
                    return Err(err("Truncated payload for 0-RTT packet"));
                }
                let payload_bytes = cur.take(remaining_for_payload)?.to_vec();

                Ok(QuicPacket::OtherLong {
                    header,
                    payload: QuicPayload::EncryptedPayload(payload_bytes),
                })
            }

            QuicPacketType::Retry => {
                // Retry a un format spécifique (pas de Length ni PN).
                // On met tout le reste en payload brut.
                let rest = cur.take(cur.left())?.to_vec();
                Ok(QuicPacket::OtherLong {
                    header,
                    payload: QuicPayload::EncryptedPayload(rest),
                })
            }

            QuicPacketType::Unknown(t) => {
                // Tentative générique: Length (varint) si possible, sinon tout en brut
                let mut snapshot = cur.clone();
                let length_field = match read_varint(&mut cur) {
                    Ok(v) => v,
                    Err(_) => {
                        // pas de varint plausible, tout en brut
                        let rest = snapshot
                            .take(snapshot.left())
                            .map_err(|_| err("internal"))?
                            .to_vec();
                        return Ok(QuicPacket::OtherLong {
                            header,
                            payload: QuicPayload::EncryptedPayload(rest),
                        });
                    }
                };
                header.length_field = length_field;

                // PN
                let pn_raw = cur.take(header.pn_length as usize)?;
                let mut pn: u64 = 0;
                for &b in pn_raw {
                    pn = (pn << 8) | (b as u64);
                }
                header.packet_number = Some(pn);

                let remaining_for_payload = length_field as usize - (header.pn_length as usize);
                if cur.left() < remaining_for_payload {
                    return Err(err(format!(
                        "Truncated payload for unknown long type {}",
                        t
                    )));
                }
                let payload_bytes = cur.take(remaining_for_payload)?.to_vec();

                Ok(QuicPacket::OtherLong {
                    header,
                    payload: QuicPayload::EncryptedPayload(payload_bytes),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // adapte si tes types sont dans un autre module

    // -------- Tests --------

    #[test]
    fn test_valid_quic_initial_minimal() {
        // construit un QUIC Initial minimal (comme tes autres tests),
        // pas besoin ETH/IPv6/UDP pour l’instant.
        let mut buf = Vec::new();
        buf.push(0xC0); // Long=1, Fixed=1, Initial, PN len=1
        buf.extend_from_slice(&0x00000001u32.to_be_bytes());
        buf.push(0); // dcid len
        buf.push(0); // scid len
        buf.push(0x00); // token len = 0
        buf.push(0x01); // length = 1 (PN seul)
        buf.push(0x00); // PN

        let pkt = QuicPacket::try_from(buf.as_slice()).expect("must parse");
        match pkt {
            QuicPacket::Initial {
                header,
                token,
                payload,
            } => {
                assert_eq!(header.version, 1);
                assert!(token.is_empty());
                matches!(payload, QuicPayload::EncryptedPayload(ref v) if v.is_empty());
            }
            _ => panic!("expected Initial"),
        }
    }

    // #[test]
    // fn test_error_short_buffer() {
    //     let buf = [0xC0u8]; // beaucoup trop court
    //     let res = QuicPacket::try_from(&buf);
    //     assert!(matches!(res, Err(crate::parse::application::ApplicationError::QuicParseError)));
    // }

    #[test]
    fn test_error_not_long_header() {
        // MSB = 0 → Short Header → notre parseur Long Header doit refuser
        // 0x40 = 0100_0000 (fixed bit = 1, mais header_form_long = 0)
        let mut buf = Vec::new();
        buf.push(0x40);
        buf.extend_from_slice(&0x00000001u32.to_be_bytes()); // version (ne sera pas lu)
        buf.push(0); // dcid len
        buf.push(0); // scid len
        let res = QuicPacket::try_from(buf.as_slice());
        assert!(matches!(
            res,
            Err(crate::parse::application::ApplicationError::QuicParseError)
        ));
    }

    #[test]
    fn test_error_fixed_bit_zero() {
        // Long Header (MSB=1) mais fixed bit = 0 → doit échouer
        // 1000_0000 = 0x80 : header_form_long=1, fixed=0, type=00, pnlen=00
        let mut buf = Vec::new();
        buf.push(0x80);
        buf.extend_from_slice(&0x00000001u32.to_be_bytes()); // version
        buf.push(0); // dcid len
        buf.push(0); // scid len
        // Pour Initial: token_len=0 varint (0x00)
        buf.push(0x00);
        // length varint: 1 (PN seul)
        buf.push(0x01);
        // PN sur 1 octet (pn_length=1 via b0=0x80 -> pn_len_code=0 -> 1)
        buf.push(0x00);
        let res = QuicPacket::try_from(buf.as_slice());
        assert!(matches!(
            res,
            Err(crate::parse::application::ApplicationError::QuicParseError)
        ));
    }

    #[test]
    fn test_error_truncated_payload_length() {
        // Construire un Initial valide en apparence mais avec length trop grand
        // b0: 1100_0000 = 0xC0 (Long=1, Fixed=1, Type=Initial(00), PN len code=00 -> 1 octet)
        let mut buf = Vec::new();
        buf.push(0xC0);
        // version v1
        buf.extend_from_slice(&0x00000001u32.to_be_bytes());
        // dcid/scid vides
        buf.push(0); // dcid len
        buf.push(0); // scid len
        // token_len = 0 (varint 1o)
        buf.push(0x00);
        // length = 5 (varint 1o) => PN(1) + payload(4) attendus
        buf.push(0x05);
        // PN (1 octet)
        buf.push(0xAA);
        // MAIS on ne met que 2 octets de payload au lieu de 4 → doit échouer
        buf.extend_from_slice(&[0x01, 0x02]);

        let res = QuicPacket::try_from(buf.as_slice());
        assert!(matches!(
            res,
            Err(crate::parse::application::ApplicationError::QuicParseError)
        ));
    }
}
