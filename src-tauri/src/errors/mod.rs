use capture_error::{CaptureError, CaptureErrorKind};
use serde::Serialize;

use crate::errors::{
    export::{ExportError, ExportErrorKind},
    import::{PcapImportError, PcapImportErrorKind},
};

pub mod capture_error;
pub mod export;
pub mod import;

#[derive(Debug, thiserror::Error)]
pub enum CaptureStateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("the mutex was poisoned")]
    PoisonError(String),
    #[error(transparent)]
    Capture(#[from] CaptureError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Import(#[from] PcapImportError),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum CaptureStateErrorKind {
    Io(String),
    PoisonError(String),
    Capture(CaptureErrorKind),
    Export(ExportErrorKind),
    Import(PcapImportErrorKind),
}

impl Serialize for CaptureStateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let kind = match self {
            Self::Io(e) => CaptureStateErrorKind::Io(e.to_string()),
            Self::PoisonError(e) => CaptureStateErrorKind::PoisonError(e.clone()),
            Self::Capture(e) => {
                // Convert `CaptureError` into `CaptureErrorKind`
                let kind = match e {
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
                };
                CaptureStateErrorKind::Import(kind)
            }
        };
        kind.serialize(serializer)
    }
}

impl<T> From<std::sync::PoisonError<T>> for CaptureStateError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        CaptureStateError::PoisonError(err.to_string())
    }
}
