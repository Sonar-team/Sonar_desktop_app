use capture_error::{CaptureError, CaptureErrorKind};
use serde::Serialize;

pub mod capture_error;

#[derive(Debug, thiserror::Error)]
pub enum CaptureStateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("the mutex was poisoned")]
    PoisonError(String),
    #[error(transparent)]
    Capture(#[from] CaptureError),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum CaptureStateErrorKind {
    Io(String),
    PoisonError(String),
    Capture(CaptureErrorKind),
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
                };
                CaptureStateErrorKind::Capture(kind)
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
