//! Erreurs de l'import de fichiers PCAP.

use thiserror::Error;

/// Erreur d'import PCAP (fichier illisible, format non reconnu ou lecture
/// interrompue en cours de fichier).
#[derive(Error, Debug)]
pub enum PcapImportError {
    #[error("Failed to open pcap file {0}: {1}")]
    OpenFileError(String, String),
    /// Erreur de lecture au milieu du fichier (tronqué, corrompu) : distincte
    /// de la fin normale pour ne jamais produire une matrice partielle en
    /// silence.
    #[error("Read error in pcap file {0}: {1}")]
    ReadPacketError(String, String),
}

/// Représentation sérialisable de [`PcapImportError`] (forme `{ kind, message }`).
#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum PcapImportErrorKind {
    OpenFileError(String, String),
    ReadPacketError(String, String),
}
