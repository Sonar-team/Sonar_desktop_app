//! Reusable Sonar domain logic.
//!
//! This crate must stay independent from Tauri so it can be shared by the
//! desktop app, the CLI, tests, and eventually external Rust consumers.

pub mod csv;
pub mod graph;
pub mod link;
pub mod matrix;
pub mod packet;
#[cfg(feature = "pcap")]
pub mod pcap;
pub mod report;
pub mod sfms;
#[cfg(feature = "xlsx")]
pub mod xlsx;

pub use error::{Result, SonarCoreError};

pub mod error {
    use std::path::PathBuf;

    #[derive(Debug, thiserror::Error)]
    pub enum SonarCoreError {
        #[error("at least one input file is required")]
        MissingInput,

        #[error("output path is required")]
        MissingOutput,

        #[error("input file does not exist: {0}")]
        MissingInputFile(PathBuf),

        #[error("{path}: {message}")]
        InvalidCsv { path: PathBuf, message: String },

        /// Une ligne déjà lue ne peut pas être reconstruite dans le DLT
        /// annoncé. Ce garde interne complète la validation numérotée du CSV
        /// et évite tout repli silencieux si l'API est appelée directement.
        #[error("ligne de matrice non reconstructible : {0}")]
        InvalidMatrixRow(String),

        #[error(transparent)]
        Io(#[from] std::io::Error),

        /// Fichier PCAP impossible à ouvrir (absent, illisible, format non
        /// reconnu par libpcap).
        #[cfg(feature = "pcap")]
        #[error("{path}: {message}")]
        PcapOpen { path: PathBuf, message: String },

        /// Erreur de lecture au milieu d'un fichier PCAP (tronqué, corrompu) :
        /// distincte de la fin normale pour ne jamais produire une matrice
        /// silencieusement partielle.
        #[cfg(feature = "pcap")]
        #[error("{path}: {message}")]
        PcapRead { path: PathBuf, message: String },

        /// Type de liaison (LINKTYPE/DLT) sans décodeur dans cette version :
        /// refusé avant toute mutation d'état, jamais parsé « comme si »
        /// c'était de l'Ethernet.
        #[cfg(feature = "pcap")]
        #[error("{path}: type de liaison non supporté : {label}")]
        UnsupportedLinkType { path: PathBuf, label: String },

        /// Fusion de relevés de types de liaison différents : refusée
        /// explicitement (arbitrage du 14/07/2026 — une fusion ne concerne
        /// que des relevés du même réseau, donc du même DLT).
        #[error(
            "{path}: type de liaison {found} incompatible avec le relevé en cours ({expected}) : \
             une fusion ne concerne que des relevés du même réseau (même type de liaison)"
        )]
        MixedLinkTypes {
            path: PathBuf,
            found: String,
            expected: String,
        },

        /// Relevé dont la projection SFMS n'est pas définie dans cette
        /// version (IEEE 802.11 ou LINKTYPE futur) : refusé plutôt que
        /// réimporté sous un autre type de liaison.
        #[error(
            "{path}: relevé {label} non réimportable : aucune projection d'identité SFMS n'est définie pour ce type de liaison"
        )]
        UnreimportableLinkType { path: PathBuf, label: String },
    }

    pub type Result<T> = std::result::Result<T, SonarCoreError>;
}

pub(crate) fn validate_batch_paths(
    inputs: &[std::path::PathBuf],
    output: &std::path::Path,
) -> Result<()> {
    if inputs.is_empty() {
        return Err(SonarCoreError::MissingInput);
    }

    if output.as_os_str().is_empty() {
        return Err(SonarCoreError::MissingOutput);
    }

    for input in inputs {
        if !input.exists() {
            return Err(SonarCoreError::MissingInputFile(input.clone()));
        }
    }

    Ok(())
}
