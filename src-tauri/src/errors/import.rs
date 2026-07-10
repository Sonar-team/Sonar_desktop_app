//! Erreurs de l'import de fichiers PCAP.

use thiserror::Error;

/// Erreur d'import PCAP (fichier illisible ou format non reconnu).
#[derive(Error, Debug)]
pub enum PcapImportError {
    #[error("Failed to open pcap file {0}: {1}")]
    OpenFileError(String, String),
}

/// Représentation sérialisable de [`PcapImportError`] (forme `{ kind, message }`).
#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum PcapImportErrorKind {
    OpenFileError(String, String),
}
