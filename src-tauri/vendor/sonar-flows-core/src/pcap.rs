//! Conversion PCAP/PCAPNG → matrice de flux (feature `pcap`).
//!
//! Port sans Tauri du chemin d'import PCAP de l'application desktop : chaque
//! paquet est parsé par `packet_parser`, aplati en niveaux de tunnel
//! (externe + internes partageant le même `encap_id`), puis agrégé dans la
//! [`FlowMatrix`]. Les paquets que le parser ne comprend pas sont comptés,
//! pas fatals. Une erreur de *lecture* (fichier tronqué, enregistrement
//! corrompu) est en revanche fatale : elle ne doit pas être confondue avec
//! une fin de fichier, sous peine de produire une matrice silencieusement
//! partielle.

use std::path::{Path, PathBuf};

use packet_parser::{LinkType, PacketFlow};
use pcap::Capture;

use crate::matrix::FlowMatrix;
use crate::packet::CapturedPacket;
use crate::{Result, SonarCoreError, validate_batch_paths};

// Le bilan de conversion et sa classification vivent dans [`crate::report`],
// hors feature `pcap` : la boucle d'import du desktop les partage sans tirer
// libpcap. Réexportés ici pour les consommateurs historiques du module.
pub use crate::report::{PacketDisposition, PcapFileReport, classify_packet};

impl<'a> CapturedPacket<'a> {
    /// Construit un [`CapturedPacket`] depuis l'en-tête d'un paquet pcap et son
    /// flux déjà parsé (les types de `ts_sec`/`ts_usec` suivent le `timeval`
    /// de chaque OS, comme les déclinaisons de la structure).
    pub fn from_pcap(header: &pcap::PacketHeader, flow: PacketFlow<'a>) -> Self {
        Self {
            ts_sec: header.ts.tv_sec,
            ts_usec: header.ts.tv_usec,
            caplen: header.caplen,
            len: header.len,
            flow,
        }
    }
}

/// Un PCAPNG multi-interfaces où deux interfaces annoncent un type de
/// liaison ou un snaplen différents n'est pas lisible par libpcap (son
/// modèle de capture ne connaît qu'un DLT et un snaplen par fichier, hérité
/// du format PCAP classique). Le message brut de libpcap ne dit pas quoi
/// faire : on l'enrichit d'une piste concrète plutôt que de laisser
/// l'utilisateur face à une erreur FFI opaque.
fn friendly_pcap_error_message(e: &pcap::Error) -> String {
    let raw = e.to_string();
    if raw.contains("different from the snapshot length of the first interface") {
        format!(
            "{raw}\nCe fichier a des interfaces avec des tailles de capture (snaplen) \
             différentes ; libpcap ne peut lire qu'un snaplen unique par fichier. \
             Normalisez-le avant import, par exemple : \
             editcap -F pcapng --snaplen 262144 <entrée> <sortie>, ou via Wireshark \
             (Fichier → Exporter des paquets spécifiés… → pcapng)."
        )
    } else if raw.contains("different from the type of the first interface") {
        format!(
            "{raw}\nCe fichier a des interfaces avec des types de liaison différents \
             (ex. Ethernet + Wi-Fi + tunnel) ; libpcap ne peut lire qu'un type de \
             liaison unique par fichier. Séparez les interfaces avant import, par \
             exemple avec Wireshark (Statistiques → Hiérarchie des interfaces, puis \
             Fichier → Exporter des paquets spécifiés… par interface)."
        )
    } else {
        raw
    }
}

/// Type de liaison (LINKTYPE/DLT) annoncé par l'en-tête d'un fichier
/// PCAP/PCAPNG (celui de la première interface pour un PCAPNG).
pub fn pcap_file_datalink(path: &Path) -> Result<pcap::Linktype> {
    let cap = Capture::from_file(path).map_err(|e| SonarCoreError::PcapOpen {
        path: path.to_path_buf(),
        message: friendly_pcap_error_message(&e),
    })?;
    Ok(cap.get_datalink())
}

/// Libellé lisible d'un type de liaison pour l'UI : description libpcap
/// (« Ethernet », « Linux cooked v1 »…), sinon nom (« EN10MB »), sinon la
/// valeur numérique — jamais de valeur inventée.
pub fn datalink_label(link_type: pcap::Linktype) -> String {
    link_type
        .get_description()
        .or_else(|_| link_type.get_name())
        .unwrap_or_else(|_| format!("DLT {}", link_type.0))
}

/// Normalise un type de liaison pcap (`DLT_*` en capture live, valeur de
/// l'en-tête pour un fichier) vers l'espace canonique `LINKTYPE_*` de
/// `packet_parser`. Seul `DLT_RAW` diffère parmi les liaisons supportées :
/// il vaut 12 sur les plateformes visées alors que `LINKTYPE_RAW` vaut 101
/// (même normalisation que libpcap à l'écriture d'un fichier).
pub fn parser_link_type(link_type: pcap::Linktype) -> LinkType {
    match link_type.0 {
        // DLT_RAW vaut 12 en capture live sur les plateformes visées, alors
        // que LINKTYPE_RAW (et pcap::Linktype::RAW) vaut 101.
        12 => LinkType::RAW,
        other => LinkType(other.unsigned_abs()),
    }
}

/// LinkType `packet_parser` d'un fichier PCAP/PCAPNG, refusé avant toute
/// lecture de paquet si cette version n'a pas de décodeur pour lui : un DLT
/// non supporté échoue explicitement au lieu d'être parsé comme de l'Ethernet.
pub fn supported_parser_link_type(path: &Path) -> Result<LinkType> {
    let file_link_type = pcap_file_datalink(path)?;
    let link_type = parser_link_type(file_link_type);
    if !packet_parser::parse::is_supported(link_type) {
        return Err(SonarCoreError::UnsupportedLinkType {
            path: path.to_path_buf(),
            label: datalink_label(file_link_type),
        });
    }
    Ok(link_type)
}

/// Itère les paquets bruts d'un fichier PCAP/PCAPNG et retourne leur nombre.
/// Une erreur de lecture en cours de fichier ([`SonarCoreError::PcapRead`])
/// est distinguée de la fin normale : un fichier tronqué échoue au lieu de
/// passer pour une fin de fichier.
pub fn for_each_raw_packet(
    path: &Path,
    mut on_packet: impl FnMut(&pcap::Packet<'_>),
) -> Result<usize> {
    let mut cap = Capture::from_file(path).map_err(|e| SonarCoreError::PcapOpen {
        path: path.to_path_buf(),
        message: friendly_pcap_error_message(&e),
    })?;

    let mut count: usize = 0;
    loop {
        match cap.next_packet() {
            Ok(packet) => {
                count += 1;
                on_packet(&packet);
            }
            Err(pcap::Error::NoMorePackets) => break,
            Err(e) => {
                return Err(SonarCoreError::PcapRead {
                    path: path.to_path_buf(),
                    message: friendly_pcap_error_message(&e),
                });
            }
        }
    }
    Ok(count)
}

/// Compte les paquets d'un fichier PCAP (pré-passe de progression). Comme
/// [`for_each_raw_packet`], échoue sur un fichier tronqué au lieu de
/// retourner un compte partiel.
pub fn count_pcap_packets(path: &Path) -> Result<usize> {
    for_each_raw_packet(path, |_| {})
}

/// Ajoute le contenu d'un fichier PCAP/PCAPNG à une matrice existante.
///
/// Les paquets non parsés sont comptés dans le bilan sans interrompre la
/// conversion ; une erreur de lecture (autre que la fin de fichier) est
/// fatale et laisse la matrice possiblement partielle — les appelants qui
/// veulent une sémantique transactionnelle passent par
/// [`convert_pcap_files`], qui ne retourne jamais de matrice partielle.
pub fn append_pcap_file(matrix: &mut FlowMatrix, path: &Path) -> Result<PcapFileReport> {
    // Un DLT sans décodeur échoue ici, avant toute mutation de la matrice :
    // le fichier ne doit pas être parsé « comme si » c'était de l'Ethernet.
    let link_type = supported_parser_link_type(path)?;
    // Un relevé = un réseau = un DLT : un fichier d'un autre type de liaison
    // que la matrice en cours est refusé (fusion inter-DLT, arbitrage 14/07).
    matrix.bind_link_type(link_type, path)?;
    let mut report = PcapFileReport::default();
    for_each_raw_packet(path, |packet| {
        match packet_parser::parse::parse(link_type, packet.data) {
            Ok(flow) => {
                report.record(PacketDisposition::Decoded);
                // Même chemin de dépliage des tunnels que le desktop
                // (`CapturedPacket::to_owned_packets`) : encap_id identiques,
                // matrices joignables entre CLI et desktop.
                for owned in CapturedPacket::from_pcap(packet.header, flow).to_owned_packets() {
                    matrix.update_flow(&owned);
                }
            }
            // La cause du rejet est classée par la partition commune (#150) :
            // troncature de capture, DLT inconnu ou trame malformée.
            Err(error) => report.record(classify_packet(
                packet.header.caplen,
                packet.header.len,
                Some(&error),
            )),
        }
    })?;
    Ok(report)
}

/// Convertit un ou plusieurs fichiers PCAP en une matrice de flux unique.
/// `on_file` est appelé après chaque fichier avec son bilan (progression).
pub fn convert_pcap_files(
    inputs: &[PathBuf],
    mut on_file: impl FnMut(&Path, &PcapFileReport),
) -> Result<FlowMatrix> {
    if inputs.is_empty() {
        return Err(SonarCoreError::MissingInput);
    }

    let mut matrix = FlowMatrix::new();
    for path in inputs {
        let report = append_pcap_file(&mut matrix, path)?;
        on_file(path, &report);
    }
    Ok(matrix)
}

/// Convertit des fichiers PCAP et exporte la matrice en CSV (MVP CLI).
/// Retourne le nombre de lignes (flux distincts) de la matrice produite.
pub fn convert_pcap_files_to_csv(
    inputs: &[PathBuf],
    output: &Path,
    on_file: impl FnMut(&Path, &PcapFileReport),
) -> Result<usize> {
    validate_batch_paths(inputs, output)?;
    let matrix = convert_pcap_files(inputs, on_file)?;
    matrix.export_to_csv(output.to_string_lossy().into_owned())?;
    Ok(matrix.row_count())
}

#[cfg(test)]
mod tests {
    use super::{PcapFileReport, append_pcap_file, parser_link_type, supported_parser_link_type};
    use crate::SonarCoreError;
    use crate::matrix::FlowMatrix;
    use packet_parser::LinkType;

    /// En-tête global PCAP minimal (little-endian) pour le LINKTYPE donné.
    fn pcap_global_header_for(link_type: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xa1b2_c3d4u32.to_le_bytes()); // magic
        bytes.extend_from_slice(&2u16.to_le_bytes()); // version majeure
        bytes.extend_from_slice(&4u16.to_le_bytes()); // version mineure
        bytes.extend_from_slice(&0i32.to_le_bytes()); // thiszone
        bytes.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
        bytes.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
        bytes.extend_from_slice(&link_type.to_le_bytes());
        bytes
    }

    fn pcap_global_header() -> Vec<u8> {
        pcap_global_header_for(1) // LINKTYPE_ETHERNET
    }

    /// Ajoute un enregistrement de paquet au PCAP en construction.
    fn push_packet_record(bytes: &mut Vec<u8>, data: &[u8]) {
        push_cut_packet_record(bytes, data, data.len() as u32);
    }

    /// Enregistrement dont la capture a été coupée : `data` est ce qui a été
    /// conservé (`incl_len`), `orig_len` la longueur de la trame sur le fil.
    fn push_cut_packet_record(bytes: &mut Vec<u8>, data: &[u8], orig_len: u32) {
        bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_sec
        bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes()); // incl_len
        bytes.extend_from_slice(&orig_len.to_le_bytes()); // orig_len
        bytes.extend_from_slice(data);
    }

    /// Paquet Linux cooked v1 (SLL, 16 octets d'en-tête) transportant un
    /// datagramme IPv4/UDP minimal.
    fn sll_ipv4_udp_packet() -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&4u16.to_be_bytes()); // packet_type OUTGOING
        packet.extend_from_slice(&1u16.to_be_bytes()); // ARPHRD_ETHER
        packet.extend_from_slice(&6u16.to_be_bytes()); // address_length
        packet.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0, 0]); // adresse (slot 8)
        packet.extend_from_slice(&0x0800u16.to_be_bytes()); // protocole IPv4
        packet.extend_from_slice(&[
            0x45, 0x00, 0x00, 0x1c, // IPv4, IHL 5, longueur totale 28
            0x00, 0x00, 0x00, 0x00, // id, flags
            0x40, 0x11, 0x00, 0x00, // TTL 64, UDP, checksum 0
            192, 168, 1, 10, // source
            192, 168, 1, 20, // destination
            0x13, 0x88, 0x13, 0x89, // ports 5000 → 5001
            0x00, 0x08, 0x00, 0x00, // longueur UDP 8, checksum 0
        ]);
        packet
    }

    fn write_pcap(dir_name: &str, file_name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(dir_name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join(file_name);
        std::fs::write(&path, bytes).expect("écriture pcap");
        path
    }

    /// Fixture du corpus commun (mêmes fichiers que les tests CSV). Les
    /// captures SLL/SLL2 réelles viennent de `packet_parser` (commit
    /// bd03272) : `tcpdump -i any`, cooked v1 et v2.
    fn fixture(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../src-tauri/test_files/pcaps/import/pcap_tshark_corpus")
            .join(name)
    }

    /// Exporte puis réimporte une matrice et compare toute son identité SFMS.
    /// Seule `origin`, ajoutée au réimport pour tracer le fichier, est hors
    /// comparaison.
    fn assert_matrix_identity_round_trip(
        matrix: &FlowMatrix,
        link_type: LinkType,
        dir_name: &str,
        file_name: &str,
    ) {
        let dir = std::env::temp_dir().join(dir_name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let out = dir.join(file_name);
        matrix
            .export_to_csv(out.to_string_lossy().into_owned())
            .expect("export du relevé cooked");

        let first_line = std::fs::read_to_string(&out)
            .expect("relecture")
            .lines()
            .next()
            .map(str::to_string)
            .expect("fichier non vide");
        assert_eq!(
            first_line,
            crate::sfms::format_preamble(Some(link_type)),
            "le DLT réel reste dans le préambule"
        );

        let reimported =
            crate::csv::merge_matrix_files(std::slice::from_ref(&out)).expect("réimport cooked");
        assert_eq!(reimported.link_type, Some(link_type));
        assert!(
            reimported
                .matrix
                .keys()
                .all(|flow| flow.data_link.link_type() == link_type),
            "toutes les clés sont reconstruites dans le vrai DLT"
        );

        let mut expected = matrix.to_flat_vec();
        let mut actual = reimported.to_flat_vec();
        for row in expected.iter_mut().chain(actual.iter_mut()) {
            row.origin.clear();
        }
        assert_eq!(actual, expected, "l'identité SFMS doit être inversible");
    }
    /// Le DLT en capture live (`DLT_RAW` = 12) et celui des fichiers
    /// (`LINKTYPE_RAW` = 101) se normalisent vers le même LinkType ; les
    /// autres valeurs passent inchangées.
    #[test]
    fn parser_link_type_normalizes_dlt_raw() {
        assert_eq!(parser_link_type(pcap::Linktype(12)), LinkType::RAW);
        assert_eq!(parser_link_type(pcap::Linktype::RAW), LinkType::RAW);
        assert_eq!(
            parser_link_type(pcap::Linktype::ETHERNET),
            LinkType::ETHERNET
        );
        assert_eq!(
            parser_link_type(pcap::Linktype::LINUX_SLL),
            LinkType::LINUX_SLL
        );
        assert_eq!(parser_link_type(pcap::Linktype(276)), LinkType::LINUX_SLL2);
    }

    /// Un PCAP Linux cooked (SLL) est parsé avec son vrai LINKTYPE : le flux
    /// IPv4 transporté atteint la matrice au lieu d'échouer en « Ethernet
    /// invalide ».
    #[test]
    fn sll_pcap_is_parsed_with_its_real_linktype() {
        let mut bytes = pcap_global_header_for(113); // LINKTYPE_LINUX_SLL
        push_packet_record(&mut bytes, &sll_ipv4_udp_packet());
        let path = write_pcap("sonar_core_pcap_sll_test", "cooked.pcap", &bytes);

        let mut matrix = FlowMatrix::new();
        let report = append_pcap_file(&mut matrix, &path).expect("pcap SLL lisible");

        assert_eq!(report.read(), 1);
        assert_eq!(report.decoded, 1, "le paquet SLL doit être parsé");
        assert_eq!(report.rejected(), 0);
        assert_eq!(matrix.row_count(), 1, "le flux IPv4 atteint la matrice");
    }

    /// Trame ARP Ethernet minimale (42 octets), partagée par les tests de
    /// conversion qui n'ont pas besoin d'une capture réelle.
    fn arp_frame() -> Vec<u8> {
        let mut arp = Vec::new();
        arp.extend_from_slice(&[0xff; 6]);
        arp.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        arp.extend_from_slice(&[0x08, 0x06, 0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01]);
        arp.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        arp.extend_from_slice(&[192, 168, 1, 10]);
        arp.extend_from_slice(&[0x00; 6]);
        arp.extend_from_slice(&[192, 168, 1, 1]);
        arp
    }

    /// Blocs PCAPNG minimaux (little-endian) pour forger des fichiers
    /// multi-interfaces : Section Header, Interface Description, Enhanced
    /// Packet. Chaque bloc porte sa longueur totale en tête ET en queue.
    fn pcapng_shb() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&0x0A0D_0D0Au32.to_le_bytes()); // type SHB
        b.extend_from_slice(&28u32.to_le_bytes());
        b.extend_from_slice(&0x1A2B_3C4Du32.to_le_bytes()); // byte-order magic
        b.extend_from_slice(&1u16.to_le_bytes()); // version majeure
        b.extend_from_slice(&0u16.to_le_bytes()); // version mineure
        b.extend_from_slice(&(-1i64).to_le_bytes()); // longueur de section inconnue
        b.extend_from_slice(&28u32.to_le_bytes());
        b
    }

    fn pcapng_idb(link_type: u16, snaplen: u32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&1u32.to_le_bytes()); // type IDB
        b.extend_from_slice(&20u32.to_le_bytes());
        b.extend_from_slice(&link_type.to_le_bytes());
        b.extend_from_slice(&0u16.to_le_bytes()); // réservé
        b.extend_from_slice(&snaplen.to_le_bytes());
        b.extend_from_slice(&20u32.to_le_bytes());
        b
    }

    fn pcapng_epb(interface_id: u32, data: &[u8]) -> Vec<u8> {
        let padded = data.len().div_ceil(4) * 4;
        let total = (32 + padded) as u32;
        let mut b = Vec::new();
        b.extend_from_slice(&6u32.to_le_bytes()); // type EPB
        b.extend_from_slice(&total.to_le_bytes());
        b.extend_from_slice(&interface_id.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes()); // timestamp haut
        b.extend_from_slice(&0u32.to_le_bytes()); // timestamp bas
        b.extend_from_slice(&(data.len() as u32).to_le_bytes()); // caplen
        b.extend_from_slice(&(data.len() as u32).to_le_bytes()); // len
        b.extend_from_slice(data);
        b.resize(b.len() + (padded - data.len()), 0);
        b.extend_from_slice(&total.to_le_bytes());
        b
    }

    /// PCAPNG multi-interfaces à DLT mélangés (#151, arbitrage 14/07/2026 :
    /// un relevé = un DLT) : libpcap refuse de mélanger des types de liaison
    /// — le fichier échoue **explicitement**, jamais en matrice partielle
    /// silencieuse. Comportement observé le 14/07 sur fichiers réels
    /// (The-Ultimate-PCAP), figé ici de façon déterministe.
    #[test]
    fn mixed_dlt_pcapng_fails_explicitly() {
        let mut bytes = pcapng_shb();
        bytes.extend_from_slice(&pcapng_idb(1, 65535)); // Ethernet
        bytes.extend_from_slice(&pcapng_idb(113, 65535)); // Linux SLL
        bytes.extend_from_slice(&pcapng_epb(0, &arp_frame()));
        bytes.extend_from_slice(&pcapng_epb(1, &sll_ipv4_udp_packet()));
        let path = write_pcap("sonar_core_pcapng_mixed_dlt_test", "mixte.pcapng", &bytes);

        let mut matrix = FlowMatrix::new();
        let Err(err) = append_pcap_file(&mut matrix, &path) else {
            panic!("un PCAPNG à DLT mélangés doit être refusé explicitement");
        };
        assert!(
            matches!(
                err,
                SonarCoreError::PcapOpen { .. } | SonarCoreError::PcapRead { .. }
            ),
            "erreur explicite attendue, obtenu : {err}"
        );
        assert!(err.to_string().contains("mixte.pcapng"));
        // Message enrichi d'une piste concrète plutôt que l'erreur FFI brute.
        assert!(
            err.to_string().contains("types de liaison différents"),
            "message non enrichi : {err}"
        );
    }

    /// Variante snaplens divergents (même DLT) : refusée par libpcap dès
    /// l'ouverture — le cas rencontré le 14/07 avec les PCAPNG réels.
    #[test]
    fn mixed_snaplen_pcapng_fails_explicitly() {
        let mut bytes = pcapng_shb();
        bytes.extend_from_slice(&pcapng_idb(1, 65535));
        bytes.extend_from_slice(&pcapng_idb(1, 8192));
        bytes.extend_from_slice(&pcapng_epb(0, &arp_frame()));
        bytes.extend_from_slice(&pcapng_epb(1, &arp_frame()));
        let path = write_pcap(
            "sonar_core_pcapng_mixed_snaplen_test",
            "snaplen.pcapng",
            &bytes,
        );

        let mut matrix = FlowMatrix::new();
        let Err(err) = append_pcap_file(&mut matrix, &path) else {
            panic!("un PCAPNG à snaplens divergents doit être refusé explicitement");
        };
        assert!(
            matches!(
                err,
                SonarCoreError::PcapOpen { .. } | SonarCoreError::PcapRead { .. }
            ),
            "erreur explicite attendue, obtenu : {err}"
        );
        assert_eq!(matrix.row_count(), 0, "aucune mutation");
        // Message enrichi d'une piste concrète plutôt que l'erreur FFI brute.
        assert!(
            err.to_string().contains("editcap"),
            "message non enrichi : {err}"
        );
    }

    /// Preuve de terminaison (#87) : un PCAP pathologique fait de paquets de
    /// longueur nulle se termine — chaque enregistrement avance la lecture,
    /// chaque paquet est classé (ici : illisible), aucune boucle infinie
    /// possible dans `for_each_raw_packet`.
    /// Preuve terrain de la partition tronqué/malformé/décodé (#150) sur un
    /// même fichier forgé :
    /// - une trame coupée au snaplen dont il ne reste que 10 octets échoue au
    ///   parse → imputée à la **troncature de capture** ;
    /// - la même trame de 10 octets annoncée complète (`caplen == len`)
    ///   échoue pareil → **malformée sur le réseau observé** ;
    /// - une trame ARP complète dont seule la longueur d'origine est plus
    ///   grande (padding non conservé) se décode → **intégrée**, la coupe de
    ///   capture n'est pas un motif de rejet en soi.
    #[test]
    fn truncated_and_malformed_packets_are_distinguished_in_the_report() {
        let arp = arp_frame();
        let mut bytes = pcap_global_header();
        push_cut_packet_record(&mut bytes, &arp[..10], 60); // coupée, illisible
        push_packet_record(&mut bytes, &arp[..10]); // complète, malformée
        push_cut_packet_record(&mut bytes, &arp, 60); // coupée, lisible
        let path = write_pcap("sonar_core_pcap_truncation_test", "cut.pcap", &bytes);

        let mut matrix = FlowMatrix::new();
        let report = append_pcap_file(&mut matrix, &path).expect("lecture terminée");

        assert_eq!(
            report,
            PcapFileReport {
                decoded: 1,
                rejected_truncated: 1,
                rejected_unsupported_link_type: 0,
                rejected_malformed: 1,
            }
        );
        assert_eq!(report.read(), 3, "l'équation couvre les trois trames");
        assert_eq!(
            matrix.row_count(),
            1,
            "seule la trame décodée entre dans la matrice"
        );
    }

    #[test]
    fn zero_length_packets_terminate_with_explicit_accounting() {
        let mut bytes = pcap_global_header();
        for _ in 0..100 {
            push_packet_record(&mut bytes, &[]);
        }
        let path = write_pcap("sonar_core_pcap_zero_len_test", "zero.pcap", &bytes);

        let mut matrix = FlowMatrix::new();
        let report = append_pcap_file(&mut matrix, &path).expect("lecture terminée");

        assert_eq!(report.read(), 100, "tous les enregistrements sont lus");
        assert_eq!(report.decoded, 0);
        assert_eq!(
            report.rejected_malformed, 100,
            "un paquet vide (caplen == len) est une trame malformée du réseau \
             observé, pas une troncature de capture — classé, jamais avalé"
        );
        assert_eq!(report.rejected_truncated, 0);
        assert_eq!(matrix.row_count(), 0);
    }

    /// Chemins avec espaces, Unicode (accents, CJK, emoji) et caractères
    /// spéciaux shell (`'`, `"`, `` ` ``) : la conversion PCAP doit les
    /// traiter comme n'importe quel fichier (#88). Les chemins passent par
    /// `std::path::Path`, jamais par un shell — ces noms sont légitimes.
    #[test]
    fn pcap_paths_with_spaces_unicode_and_special_characters_convert() {
        let mut bytes = pcap_global_header();
        push_packet_record(&mut bytes, &arp_frame());

        for name in [
            "relevé du réseau (été 2026).pcap",
            "capture 中文-網絡 📡.pcap",
            "l'atelier d'essai.pcap",
            "site \"principal\".pcap",
            "sonde `usine`.pcap",
        ] {
            let path = write_pcap("sonar_core_pcap_weird_names_test", name, &bytes);

            let link_type =
                supported_parser_link_type(&path).unwrap_or_else(|e| panic!("DLT de {name}: {e}"));
            assert_eq!(link_type, LinkType::ETHERNET);

            let mut matrix = FlowMatrix::new();
            let report = append_pcap_file(&mut matrix, &path)
                .unwrap_or_else(|e| panic!("conversion de {name}: {e}"));
            assert_eq!(report.read(), 1, "{name}");
            assert_eq!(report.decoded, 1, "{name}");
            assert_eq!(matrix.row_count(), 1, "{name}");
        }
    }

    /// Bout-en-bout sur capture réelle Linux cooked v1 (2702 trames,
    /// `tcpdump -i any`) : conversion avec le vrai DLT puis aller-retour CSV
    /// exact de l'identité SFMS.
    #[test]
    fn real_sll_capture_round_trips_its_matrix_identity() {
        let mut matrix = FlowMatrix::new();
        let report =
            append_pcap_file(&mut matrix, &fixture("linux_sll.pcap")).expect("capture SLL réelle");

        assert_eq!(report.read(), 2702, "toutes les trames du fichier lues");
        // L'équation lus = décodés + rejetés est vraie par construction
        // depuis #150 : le total est dérivé des catégories, plus un compteur
        // indépendant qui pourrait diverger.
        assert!(report.decoded > 0, "des flux SLL sont décodés");
        assert!(matrix.row_count() > 0, "la matrice contient des flux");
        assert_eq!(matrix.link_type, Some(LinkType::LINUX_SLL));

        assert_matrix_identity_round_trip(
            &matrix,
            LinkType::LINUX_SLL,
            "sonar_core_real_sll_test",
            "releve_sll.csv",
        );
    }

    /// Capture réelle Linux cooked v2 (779 trames) : conversion avec le vrai
    /// DLT puis aller-retour CSV exact, y compris après neutralisation de
    /// l'index d'interface propre au point d'observation.
    #[test]
    fn real_sll2_capture_round_trips_its_matrix_identity() {
        let mut matrix = FlowMatrix::new();
        let report = append_pcap_file(&mut matrix, &fixture("linux_sll2.pcap"))
            .expect("capture SLL2 réelle");

        assert_eq!(report.read(), 779, "toutes les trames du fichier lues");
        assert_matrix_identity_round_trip(
            &matrix,
            LinkType::LINUX_SLL2,
            "sonar_core_real_sll2_test",
            "releve_sll2.csv",
        );
        assert!(report.decoded > 0, "des flux SLL2 sont décodés");
        assert!(matrix.row_count() > 0, "la matrice contient des flux");
        assert_eq!(matrix.link_type, Some(LinkType::LINUX_SLL2));
    }

    /// Deux fichiers PCAP de DLT différents ne se convertissent pas dans la
    /// même matrice : rejet explicite (arbitrage du 14/07/2026).
    #[test]
    fn converting_pcaps_of_different_dlt_into_one_matrix_is_refused() {
        let mut ethernet_bytes = pcap_global_header_for(1);
        push_packet_record(&mut ethernet_bytes, &arp_frame());
        let ethernet = write_pcap("sonar_core_pcap_mixed_test", "eth.pcap", &ethernet_bytes);

        let mut sll_bytes = pcap_global_header_for(113);
        push_packet_record(&mut sll_bytes, &sll_ipv4_udp_packet());
        let sll = std::path::Path::new(&ethernet)
            .parent()
            .expect("dossier")
            .join("cooked.pcap");
        std::fs::write(&sll, &sll_bytes).expect("écriture pcap sll");

        let mut matrix = FlowMatrix::new();
        append_pcap_file(&mut matrix, &ethernet).expect("premier fichier Ethernet");
        let err = append_pcap_file(&mut matrix, &sll).expect_err("fusion inter-DLT refusée");
        assert!(
            matches!(err, SonarCoreError::MixedLinkTypes { .. }),
            "erreur dédiée attendue, obtenu : {err}"
        );
        assert_eq!(matrix.link_type, Some(LinkType::ETHERNET));
        assert_eq!(matrix.row_count(), 1, "le fichier refusé n'a rien muté");
    }

    /// Un DLT sans décodeur échoue avant toute mutation : la matrice reste
    /// vide et l'erreur cite le fichier et le type de liaison.
    #[test]
    fn unsupported_dlt_fails_before_any_matrix_mutation() {
        let mut bytes = pcap_global_header_for(108); // LINKTYPE_LOOP (OpenBSD)
        push_packet_record(&mut bytes, &[0u8; 32]);
        let path = write_pcap("sonar_core_pcap_unsupported_test", "loop.pcap", &bytes);

        let mut matrix = FlowMatrix::new();
        let err = append_pcap_file(&mut matrix, &path).expect_err("DLT non supporté");

        assert!(
            matches!(err, SonarCoreError::UnsupportedLinkType { .. }),
            "erreur dédiée attendue, obtenu : {err}"
        );
        assert!(
            err.to_string().contains("loop.pcap"),
            "l'erreur doit citer le fichier : {err}"
        );
        assert_eq!(matrix.row_count(), 0, "aucune mutation avant l'échec");
    }

    #[test]
    fn truncated_pcap_is_an_error_not_an_eof() {
        let dir = std::env::temp_dir().join("sonar_core_pcap_truncated_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("truncated.pcap");

        // En-tête d'enregistrement annonçant 100 octets, suivi de 10 seulement :
        // le cas « PCAP tronqué » qui était avalé comme une fin de fichier.
        let mut bytes = pcap_global_header();
        bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_sec
        bytes.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
        bytes.extend_from_slice(&100u32.to_le_bytes()); // incl_len
        bytes.extend_from_slice(&100u32.to_le_bytes()); // orig_len
        bytes.extend_from_slice(&[0u8; 10]);
        std::fs::write(&path, bytes).expect("écriture pcap tronqué");

        let mut matrix = FlowMatrix::new();
        let err = append_pcap_file(&mut matrix, &path).expect_err("le pcap tronqué doit échouer");
        assert!(
            err.to_string().contains("truncated.pcap"),
            "l'erreur doit citer le fichier fautif: {err}"
        );
    }
}
