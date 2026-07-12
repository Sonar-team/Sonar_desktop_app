//! Erreurs de l'application. [`CaptureStateError`] agrège les erreurs de
//! chaque domaine (capture, export, import, labels) et se sérialise vers le
//! frontend sous la forme discriminée `{ kind, message }` attendue par
//! `src/errors/capture.ts`.

use capture_error::{CaptureError, CaptureErrorKind};
use serde::Serialize;

use crate::errors::{
    export::{ExportError, ExportErrorKind},
    import::{PcapImportError, PcapImportErrorKind},
    label::{LabelError, LabelErrorKind},
};

pub mod capture_error;
pub mod export;
pub mod import;
pub mod label;

/// Erreur agrégée retournée par les commandes Tauri : chaque variante
/// enveloppe l'erreur d'un domaine (IO, verrou empoisonné, capture, export,
/// import PCAP, labels, Tauri).
#[derive(Debug, thiserror::Error)]
pub enum CaptureStateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("the mutex was poisoned")]
    PoisonError(String),
    /// Transition refusée par la machine d'état de capture (ex. démarrage
    /// pendant qu'une capture tourne déjà).
    #[error("transition de capture refusée : {from} → {to}")]
    InvalidTransition { from: String, to: String },
    #[error(transparent)]
    Capture(#[from] CaptureError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Import(#[from] PcapImportError),
    #[error(transparent)]
    Label(#[from] LabelError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

/// Représentation sérialisable de [`CaptureStateError`] : forme discriminée
/// `{ kind, message }` consommée telle quelle par le frontend.
#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum CaptureStateErrorKind {
    Io(String),
    PoisonError(String),
    InvalidTransition(String),
    Capture(CaptureErrorKind),
    Export(ExportErrorKind),
    Import(PcapImportErrorKind),
    Label(LabelErrorKind),
    Tauri(String),
}

impl Serialize for CaptureStateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let kind = match self {
            Self::Io(e) => CaptureStateErrorKind::Io(e.to_string()),
            Self::PoisonError(e) => CaptureStateErrorKind::PoisonError(e.clone()),
            Self::InvalidTransition { .. } => {
                CaptureStateErrorKind::InvalidTransition(self.to_string())
            }
            Self::Capture(e) => {
                // Convert `CaptureError` into `CaptureErrorKind`
                let kind = match e {
                    CaptureError::InvalidConfig(msg) => {
                        CaptureErrorKind::InvalidConfig(msg.clone())
                    }
                    CaptureError::ConfigPersistence(msg) => {
                        CaptureErrorKind::ConfigPersistence(msg.clone())
                    }
                    CaptureError::InterfaceNotFound(msg) => {
                        CaptureErrorKind::InterfaceNotFound(msg.clone())
                    }
                    CaptureError::DeviceListError(e) => {
                        CaptureErrorKind::DeviceListError(e.to_string())
                    }
                    CaptureError::CaptureInitError(e) => {
                        CaptureErrorKind::CaptureInitError(e.to_string())
                    }
                    CaptureError::ChannelSendError(e) => {
                        CaptureErrorKind::ChannelSendError(e.to_string())
                    }
                    CaptureError::EventSendError(e) => {
                        CaptureErrorKind::EventSendError(e.to_string())
                    } //   CaptureError::FilterError(e) => CaptureErrorKind::FilterError(e.to_string()),
                };
                CaptureStateErrorKind::Capture(kind)
            }
            Self::Export(e) => {
                let kind = match e {
                    ExportError::EmptyPath => ExportErrorKind::EmptyPath,
                    ExportError::Io(e) => ExportErrorKind::Io(e.to_string()),
                    ExportError::Csv(e) => ExportErrorKind::Csv(e.to_string()),
                    ExportError::PoisonError(e) => ExportErrorKind::PoisonError(e.clone()),
                    ExportError::LogNotFound => ExportErrorKind::LogNotFound,
                };
                CaptureStateErrorKind::Export(kind)
            }
            Self::Import(e) => {
                let kind = match e {
                    PcapImportError::OpenFileError(msg, msgg) => {
                        PcapImportErrorKind::OpenFileError(msg.clone(), msgg.clone())
                    }
                    PcapImportError::ReadPacketError(file, msg) => {
                        PcapImportErrorKind::ReadPacketError(file.clone(), msg.clone())
                    }
                };
                CaptureStateErrorKind::Import(kind)
            }
            Self::Label(e) => {
                let kind = match e {
                    LabelError::InvalidMacIpFormat {
                        invalid_mac,
                        invalid_ip,
                    } => {
                        LabelErrorKind::InvalidMacIpFormat(invalid_mac.clone(), invalid_ip.clone())
                    }
                    LabelError::LabelLinesConflicts {
                        same_ip_diff_mac,
                        same_ip_diff_label,
                    } => LabelErrorKind::LabelLinesConflicts(
                        same_ip_diff_mac.clone(),
                        same_ip_diff_label.clone(),
                    ),
                    LabelError::InvalidRowsFormat { invalid_lines } => {
                        LabelErrorKind::InvalidRowsFormat(invalid_lines.clone())
                    }
                    LabelError::EditRejected(reason) => {
                        LabelErrorKind::EditRejected(reason.clone())
                    }
                };
                CaptureStateErrorKind::Label(kind)
            }
            Self::Tauri(e) => CaptureStateErrorKind::Tauri(e.to_string()),
        };
        kind.serialize(serializer)
    }
}

impl<T> From<std::sync::PoisonError<T>> for CaptureStateError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        CaptureStateError::PoisonError(err.to_string())
    }
}

/// Les erreurs du cœur partagé gardent leur distinction ouverture/lecture
/// (préservée côté front par `PcapImportErrorKind`) ; le reste (CSV invalide,
/// IO…) passe par la variante `Io` avec son message d'origine.
impl From<sonar_flows_core::SonarCoreError> for CaptureStateError {
    fn from(err: sonar_flows_core::SonarCoreError) -> Self {
        use sonar_flows_core::SonarCoreError;
        match err {
            SonarCoreError::PcapOpen { path, message } => CaptureStateError::Import(
                PcapImportError::OpenFileError(path.display().to_string(), message),
            ),
            SonarCoreError::PcapRead { path, message } => CaptureStateError::Import(
                PcapImportError::ReadPacketError(path.display().to_string(), message),
            ),
            SonarCoreError::Io(e) => CaptureStateError::Io(e),
            other => CaptureStateError::Io(std::io::Error::other(other.to_string())),
        }
    }
}
