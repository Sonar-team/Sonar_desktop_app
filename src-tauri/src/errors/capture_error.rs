//! Erreurs du démarrage et du pipeline de capture réseau.

use crossbeam::channel::TrySendError;
use thiserror::Error;

use crate::state::capture::capture_handle::messages::CaptureMessage;

/// Erreur survenue à la configuration, à l'initialisation ou pendant la
/// capture (canal interne ou IPC).
#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Configuration de capture invalide : {0}")]
    InvalidConfig(String),

    #[error("Erreur de persistance de la configuration : {0}")]
    ConfigPersistence(String),

    #[error("Interface réseau introuvable : {0}")]
    InterfaceNotFound(String),

    #[error("Erreur lors de la récupération de la liste des interfaces : {0}")]
    DeviceListError(#[from] pcap::Error),

    #[error("Erreur lors de l'initialisation de la capture : {0}")]
    CaptureInitError(#[from] std::io::Error),

    #[error("Erreur lors de l'envoi via le canal : {0}")]
    ChannelSendError(#[from] TrySendError<CaptureMessage>),

    #[error("Erreur lors de l'envoi de l'evenement : {0}")]
    EventSendError(#[from] tauri::Error),

    /// Type de liaison de l'interface sans décodeur dans cette version :
    /// refusé au démarrage, avant tout événement `Started` (un DLT non
    /// supporté ne doit jamais être parsé comme de l'Ethernet).
    #[error("Type de liaison non supporté par cette version : {0}")]
    UnsupportedLinkType(String),

    /// Interface d'un autre type de liaison que le relevé en cours : capture
    /// refusée (un relevé = un réseau = un DLT, arbitrage du 14/07/2026).
    #[error("{0}")]
    MixedLinkType(String),
    // #[error("Erreur lors de l'application du filtre : {0}")]
    // FilterError(String),
}

/// Représentation sérialisable de [`CaptureError`] (forme `{ kind, message }`).
#[derive(serde::Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum CaptureErrorKind {
    InvalidConfig(String),
    ConfigPersistence(String),
    InterfaceNotFound(String),
    DeviceListError(String),
    CaptureInitError(String),
    ChannelSendError(String),
    EventSendError(String),
    UnsupportedLinkType(String),
    MixedLinkType(String),
    // FilterError(String),
}

impl serde::Serialize for CaptureError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let kind = match self {
            Self::InvalidConfig(msg) => CaptureErrorKind::InvalidConfig(msg.clone()),
            Self::ConfigPersistence(msg) => CaptureErrorKind::ConfigPersistence(msg.clone()),
            Self::InterfaceNotFound(msg) => CaptureErrorKind::InterfaceNotFound(msg.clone()),
            Self::DeviceListError(e) => CaptureErrorKind::DeviceListError(e.to_string()),
            Self::CaptureInitError(e) => CaptureErrorKind::CaptureInitError(e.to_string()),
            Self::ChannelSendError(e) => CaptureErrorKind::ChannelSendError(e.to_string()),
            Self::EventSendError(e) => CaptureErrorKind::EventSendError(e.to_string()),
            Self::UnsupportedLinkType(label) => {
                CaptureErrorKind::UnsupportedLinkType(label.clone())
            }
            Self::MixedLinkType(message) => CaptureErrorKind::MixedLinkType(message.clone()),
            // Self::FilterError(e) => CaptureErrorKind::FilterError(e.to_string()),
        };
        kind.serialize(serializer)
    }
}
