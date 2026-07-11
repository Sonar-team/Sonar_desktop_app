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

use packet_parser::PacketFlow;
use pcap::Capture;

use crate::matrix::FlowMatrix;
use crate::packet::PacketMinimal;
use crate::{Result, SonarCoreError, validate_batch_paths};

/// Bilan de conversion d'un fichier PCAP.
#[derive(Debug, Default, Clone, Copy)]
pub struct PcapFileReport {
    /// Paquets lus dans le fichier.
    pub packets: usize,
    /// Paquets parsés et intégrés à la matrice.
    pub parse_ok: usize,
    /// Paquets lus mais non supportés par le parser (ignorés).
    pub parse_errors: usize,
}

/// Ajoute le contenu d'un fichier PCAP/PCAPNG à une matrice existante.
///
/// Les paquets non parsés sont comptés dans le bilan sans interrompre la
/// conversion ; une erreur de lecture (autre que la fin de fichier) est
/// fatale et laisse la matrice possiblement partielle — les appelants qui
/// veulent une sémantique transactionnelle passent par
/// [`convert_pcap_files`], qui ne retourne jamais de matrice partielle.
pub fn append_pcap_file(matrix: &mut FlowMatrix, path: &Path) -> Result<PcapFileReport> {
    let pcap_error = |e: &pcap::Error| SonarCoreError::Pcap {
        path: path.to_path_buf(),
        message: e.to_string(),
    };
    let mut cap = Capture::from_file(path).map_err(|e| pcap_error(&e))?;

    let mut report = PcapFileReport::default();
    loop {
        let packet = match cap.next_packet() {
            Ok(packet) => packet,
            Err(pcap::Error::NoMorePackets) => break,
            Err(e) => return Err(pcap_error(&e)),
        };
        report.packets += 1;
        match PacketFlow::try_from(packet.data) {
            Ok(flow) => {
                report.parse_ok += 1;
                // Même chemin de dépliage des tunnels que le desktop
                // (`PacketMinimal::to_owned_packets`) : encap_id identiques,
                // matrices joignables entre CLI et desktop.
                let packet_min = PacketMinimal {
                    ts_sec: packet.header.ts.tv_sec,
                    ts_usec: packet.header.ts.tv_usec,
                    caplen: packet.header.caplen,
                    len: packet.header.len,
                    flow,
                };
                for owned in packet_min.to_owned_packets() {
                    matrix.update_flow(&owned);
                }
            }
            Err(_) => report.parse_errors += 1,
        }
    }
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
    use super::append_pcap_file;
    use crate::matrix::FlowMatrix;

    /// En-tête global PCAP minimal (little-endian, Ethernet).
    fn pcap_global_header() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0xa1b2_c3d4u32.to_le_bytes()); // magic
        bytes.extend_from_slice(&2u16.to_le_bytes()); // version majeure
        bytes.extend_from_slice(&4u16.to_le_bytes()); // version mineure
        bytes.extend_from_slice(&0i32.to_le_bytes()); // thiszone
        bytes.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
        bytes.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
        bytes.extend_from_slice(&1u32.to_le_bytes()); // LINKTYPE_ETHERNET
        bytes
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
