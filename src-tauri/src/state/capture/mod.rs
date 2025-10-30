use capture_config::CaptureConfig;
use capture_handle::CaptureHandle;
use capture_status::CaptureStatus;

pub mod capture_config;
pub mod capture_handle;
pub mod capture_status;

pub struct CaptureState {
    pub capture: Option<CaptureHandle>,
    pub status: CaptureStatus,
    pub config: CaptureConfig,
    pub filter: Option<String>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            capture: None,
            status: CaptureStatus::default(),
            config: CaptureConfig::default(),
            filter: None,
        }
    }
}
