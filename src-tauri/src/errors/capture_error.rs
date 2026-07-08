use crossbeam::channel::TrySendError;
use thiserror::Error;

use crate::state::capture::capture_handle::messages::CaptureMessage;

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
    // #[error("Erreur lors de l'application du filtre : {0}")]
    // FilterError(String),
}

#[derive(serde::Serialize)]
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
            // Self::FilterError(e) => CaptureErrorKind::FilterError(e.to_string()),
        };
        kind.serialize(serializer)
    }
}
