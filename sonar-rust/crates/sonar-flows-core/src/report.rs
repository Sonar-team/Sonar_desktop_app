//! Comptabilité de lecture d'un relevé (#150).
//!
//! Chaque paquet lu appartient à **exactement une** catégorie : décodé et
//! intégré, ou rejeté pour une cause nommée. Le total de paquets lus n'est
//! pas un compteur indépendant — il est **dérivé** de la somme des
//! catégories, donc l'équation
//! `lus = décodés + tronqués + DLT non supportés + malformés`
//! est vraie par construction et ne peut pas dériver silencieusement.
//!
//! La classification est la **seule** du dépôt : le convertisseur du cœur
//! ([`crate::pcap::append_pcap_file`]) et la boucle d'import du desktop
//! consomment [`classify_packet`] plutôt que de réimplémenter chacun leur
//! partition — deux implémentations finiraient par diverger.

use packet_parser::ParseError;

/// Catégorie d'un paquet lu, exhaustive et exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDisposition {
    /// Parsé et intégrable à la matrice.
    Decoded,
    /// Le parse a échoué et la capture était coupée (`caplen < len`) : la
    /// trame d'origine était plus longue que ce qui a été conservé, l'échec
    /// est imputé à la troncature (snaplen trop court à la capture), pas au
    /// réseau observé.
    RejectedTruncated,
    /// Le parser ne connaît pas ce type de liaison. Structurellement
    /// impossible pour un fichier — la garde DLT rejette le fichier entier
    /// avant toute lecture — mais la catégorie existe pour que le bilan
    /// puisse **prouver** ce zéro au lieu de le supposer.
    RejectedUnsupportedLinkType,
    /// Trame complète (`caplen == len`) que le parser refuse : malformée ou
    /// trop courte sur le réseau lui-même.
    RejectedMalformed,
}

/// Classe un paquet lu à partir de son en-tête de capture et du résultat de
/// parse. `error = None` signifie que le parse a réussi.
///
/// L'ordre des tests compte : un DLT inconnu est un défaut de couverture du
/// parser quel que soit l'état de la trame, il prime donc sur la troncature.
pub fn classify_packet(caplen: u32, len: u32, error: Option<&ParseError>) -> PacketDisposition {
    match error {
        None => PacketDisposition::Decoded,
        Some(ParseError::UnsupportedLinkType(_)) => PacketDisposition::RejectedUnsupportedLinkType,
        Some(_) if caplen < len => PacketDisposition::RejectedTruncated,
        Some(_) => PacketDisposition::RejectedMalformed,
    }
}

/// Bilan de lecture d'un fichier PCAP/PCAPNG, par catégorie fine.
///
/// Le nombre de paquets lus s'obtient par [`Self::read`] — il n'existe pas de
/// champ `packets` à maintenir en cohérence avec les catégories.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PcapFileReport {
    /// Paquets parsés et intégrés à la matrice.
    pub decoded: usize,
    /// Rejetés parce que la capture était coupée (`caplen < len`).
    pub rejected_truncated: usize,
    /// Rejetés pour type de liaison inconnu du parser (zéro attendu pour un
    /// fichier : la garde DLT a déjà rejeté le fichier sinon).
    pub rejected_unsupported_link_type: usize,
    /// Rejetés trame complète : malformés sur le réseau observé.
    pub rejected_malformed: usize,
}

impl PcapFileReport {
    /// Enregistre un paquet dans sa catégorie.
    pub fn record(&mut self, disposition: PacketDisposition) {
        match disposition {
            PacketDisposition::Decoded => self.decoded += 1,
            PacketDisposition::RejectedTruncated => self.rejected_truncated += 1,
            PacketDisposition::RejectedUnsupportedLinkType => {
                self.rejected_unsupported_link_type += 1
            }
            PacketDisposition::RejectedMalformed => self.rejected_malformed += 1,
        }
    }

    /// Paquets lus : somme des catégories. L'équation du bilan est vraie par
    /// construction — il n'y a pas de total indépendant qui puisse diverger.
    pub fn read(&self) -> usize {
        self.decoded
            + self.rejected_truncated
            + self.rejected_unsupported_link_type
            + self.rejected_malformed
    }

    /// Paquets lus mais non intégrés, toutes causes confondues.
    pub fn rejected(&self) -> usize {
        self.read() - self.decoded
    }

    /// Cumule un autre bilan (conversion multi-fichiers).
    pub fn absorb(&mut self, other: &PcapFileReport) {
        self.decoded += other.decoded;
        self.rejected_truncated += other.rejected_truncated;
        self.rejected_unsupported_link_type += other.rejected_unsupported_link_type;
        self.rejected_malformed += other.rejected_malformed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packet_parser::LinkType;

    #[test]
    fn a_successful_parse_is_decoded_regardless_of_truncation() {
        // Une trame coupée au snaplen dont les en-têtes tiennent dans la
        // partie conservée se décode : elle est intégrée, pas rejetée. La
        // matrice utilise la longueur d'origine (`len`) pour les octets.
        assert_eq!(classify_packet(60, 1500, None), PacketDisposition::Decoded);
        assert_eq!(classify_packet(60, 60, None), PacketDisposition::Decoded);
    }

    #[test]
    fn a_failed_parse_on_a_cut_capture_is_imputed_to_truncation() {
        let error = ParseError::PacketTooShort(10);
        assert_eq!(
            classify_packet(10, 1500, Some(&error)),
            PacketDisposition::RejectedTruncated
        );
    }

    #[test]
    fn a_failed_parse_on_a_complete_frame_is_malformed() {
        let error = ParseError::PacketTooShort(10);
        assert_eq!(
            classify_packet(10, 10, Some(&error)),
            PacketDisposition::RejectedMalformed
        );
    }

    #[test]
    fn an_unknown_link_type_wins_over_truncation() {
        // Un DLT inconnu est un défaut de couverture du parser, pas un défaut
        // de la trame : la catégorie prime même si la capture est coupée.
        let error = ParseError::UnsupportedLinkType(LinkType(147));
        assert_eq!(
            classify_packet(10, 1500, Some(&error)),
            PacketDisposition::RejectedUnsupportedLinkType
        );
    }

    #[test]
    fn the_equation_holds_by_construction() {
        let mut report = PcapFileReport::default();
        for disposition in [
            PacketDisposition::Decoded,
            PacketDisposition::Decoded,
            PacketDisposition::RejectedTruncated,
            PacketDisposition::RejectedMalformed,
            PacketDisposition::RejectedUnsupportedLinkType,
        ] {
            report.record(disposition);
        }
        assert_eq!(report.read(), 5);
        assert_eq!(report.rejected(), 3);
        assert_eq!(
            report.read(),
            report.decoded
                + report.rejected_truncated
                + report.rejected_unsupported_link_type
                + report.rejected_malformed
        );
    }

    #[test]
    fn absorb_sums_every_category() {
        let mut total = PcapFileReport {
            decoded: 1,
            rejected_truncated: 2,
            rejected_unsupported_link_type: 3,
            rejected_malformed: 4,
        };
        total.absorb(&PcapFileReport {
            decoded: 10,
            rejected_truncated: 20,
            rejected_unsupported_link_type: 30,
            rejected_malformed: 40,
        });
        assert_eq!(total.read(), 110);
        assert_eq!(total.decoded, 11);
        assert_eq!(total.rejected_malformed, 44);
    }
}
