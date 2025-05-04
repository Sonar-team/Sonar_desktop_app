use crossbeam::channel::TrySendError;
use thiserror::Error;

use crate::tauri_state::capture::capture_handle::capture_message::CaptureMessage;


#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Interface réseau introuvable : {0}")]
    InterfaceNotFound(String),

    #[error("Erreur lors de la récupération de la liste des interfaces : {0}")]
    DeviceListError(#[from] pcap::Error),

    #[error("Erreur lors de l'initialisation de la capture : {0}")]
    CaptureInitError(#[from] std::io::Error),

    #[error("Erreur lors de l'envoi via le canal : {0}")]
    ChannelSendError(#[from] TrySendError<CaptureMessage>),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum CaptureErrorKind {
    InterfaceNotFound(String),
    DeviceListError(String),
    CaptureInitError(String),
    ChannelSendError(String),
}

impl serde::Serialize for CaptureError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let kind = match self {
            Self::InterfaceNotFound(msg) => CaptureErrorKind::InterfaceNotFound(msg.clone()),
            Self::DeviceListError(e) => CaptureErrorKind::DeviceListError(e.to_string()),
            Self::CaptureInitError(e) => CaptureErrorKind::CaptureInitError(e.to_string()),
            Self::ChannelSendError(e) => CaptureErrorKind::ChannelSendError(e.to_string()),
        };
        kind.serialize(serializer)
    }
}
