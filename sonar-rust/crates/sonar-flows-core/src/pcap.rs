//! Conversion PCAP/PCAPNG → matrice de flux (feature `pcap`).
//!
//! Port sans Tauri du chemin d'import PCAP de l'application desktop : chaque
//! paquet est parsé par `packet_parser`, aplati en niveaux de tunnel
//! (externe + internes partageant le même `encap_id`), puis agrégé dans la
//! [`FlowMatrix`]. Les paquets que le parser ne comprend pas sont comptés,
//! pas fatals.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use packet_parser::PacketFlow;
use packet_parser::owned::PacketFlowOwned;
use pcap::Capture;

use crate::matrix::{FlowMatrix, PacketOwnedStats};
use crate::{Result, SonarCoreError, validate_batch_paths};

/// Identifiant de tunnel : hash des extrémités de la paire, indépendant du
/// sens (l'aller et le retour d'un même tunnel partagent le même id). Même
/// calcul que l'application desktop pour que les matrices restent joignables.
fn tunnel_pair_id(flow: &PacketFlowOwned) -> u64 {
    let source = (
        &flow.data_link.source_mac,
        flow.internet.as_ref().and_then(|i| i.source_ip),
        flow.transport.as_ref().and_then(|t| t.source_port),
    );
    let destination = (
        &flow.data_link.destination_mac,
        flow.internet.as_ref().and_then(|i| i.destination_ip),
        flow.transport.as_ref().and_then(|t| t.destination_port),
    );
    let (first, second) = if source <= destination {
        (source, destination)
    } else {
        (destination, source)
    };

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    first.hash(&mut hasher);
    second.hash(&mut hasher);
    flow.data_link.ethertype.hash(&mut hasher);
    flow.data_link.vlan.hash(&mut hasher);
    flow.internet
        .as_ref()
        .map(|i| &i.protocol)
        .hash(&mut hasher);
    flow.transport
        .as_ref()
        .map(|t| &t.protocol)
        .hash(&mut hasher);
    flow.application
        .as_ref()
        .map(|a| &a.protocol)
        .hash(&mut hasher);
    hasher.finish()
}

/// Convertit un paquet parsé et tous ses niveaux encapsulés en une liste de
/// [`PacketOwnedStats`], du plus externe au plus interne (port de
/// `PacketMinimal::to_owned_packets` côté desktop).
///
/// La taille en octets est attribuée **par niveau** — trame complète pour
/// l'externe, taille du segment L3 pour chaque niveau interne — afin de ne
/// pas compter deux fois le même volume. Pour un paquet tunnelé, tous les
/// niveaux partagent le même `encap_id`.
fn owned_packets(
    ts_sec: i64,
    ts_usec: i64,
    caplen: u32,
    len: u32,
    flow: &PacketFlow<'_>,
) -> Vec<PacketOwnedStats> {
    let levels = flow.flatten();
    let tunneled = levels.len() > 1;

    let mut owned: Vec<PacketOwnedStats> = levels
        .into_iter()
        .enumerate()
        .map(|(depth, level)| {
            let level_len = if depth == 0 {
                len
            } else {
                level.data_link.payload.len() as u32
            };
            PacketOwnedStats {
                ts_sec,
                ts_usec,
                caplen,
                len: level_len,
                flow: level.to_owned(),
                encap_id: None,
            }
        })
        .collect();

    if tunneled {
        let id = tunnel_pair_id(&owned[0].flow);
        for packet in &mut owned {
            packet.encap_id = Some(id);
        }
    }

    owned
}

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
pub fn append_pcap_file(matrix: &mut FlowMatrix, path: &Path) -> Result<PcapFileReport> {
    let mut cap = Capture::from_file(path).map_err(|e| SonarCoreError::Pcap {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    let mut report = PcapFileReport::default();
    while let Ok(packet) = cap.next_packet() {
        report.packets += 1;
        match PacketFlow::try_from(packet.data) {
            Ok(flow) => {
                report.parse_ok += 1;
                // Cast nécessaire en cross-platform : les `timeval` de libpcap
                // sont `i32` sous Windows, `i64` sous Linux.
                #[allow(clippy::unnecessary_cast)]
                for owned in owned_packets(
                    packet.header.ts.tv_sec as i64,
                    packet.header.ts.tv_usec as i64,
                    packet.header.caplen,
                    packet.header.len,
                    &flow,
                ) {
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
