//! Erreurs de l'import de fichiers PCAP.

use thiserror::Error;

/// Erreur d'import PCAP (fichier illisible, format non reconnu ou lecture
/// interrompue en cours de fichier).
#[derive(Error, Debug)]
pub enum PcapImportError {
    /// La commande a été appelée sans aucune source. Cette erreur est
    /// distinguée de l'IO pour que le frontend puisse expliquer le refus et
    /// pour garantir qu'il intervient avant tout événement ou mutation.
    #[error("Aucun fichier sélectionné pour {0}")]
    MissingInput(String),
    #[error("Failed to open pcap file {0}: {1}")]
    OpenFileError(String, String),
    /// Erreur de lecture au milieu du fichier (tronqué, corrompu) : distincte
    /// de la fin normale pour ne jamais produire une matrice partielle en
    /// silence.
    #[error("Read error in pcap file {0}: {1}")]
    ReadPacketError(String, String),
    /// Type de liaison du fichier sans décodeur dans cette version : refusé
    /// avant toute mutation, jamais parsé comme de l'Ethernet.
    #[error("Type de liaison non supporté dans {0} : {1}")]
    UnsupportedLinkType(String, String),
}

/// Représentation sérialisable de [`PcapImportError`] (forme `{ kind, message }`).
#[derive(serde::Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum PcapImportErrorKind {
    MissingInput(String),
    OpenFileError(String, String),
    ReadPacketError(String, String),
    UnsupportedLinkType(String, String),
}
