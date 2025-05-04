use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct CaptureStatus {
    pub is_running: bool,
}

impl CaptureStatus {
    pub fn default() -> Self {
        Self {
            is_running: false,
        }
    }
    pub fn toggle(&mut self) {
        self.is_running = !self.is_running;
    }
}
