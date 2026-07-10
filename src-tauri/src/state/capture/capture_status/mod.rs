//! Statut de la capture exposé au frontend.

use serde::Serialize;

/// Statut courant de la capture (démarrée ou non), renvoyé par les commandes
/// `start_capture`/`stop_capture`.
#[derive(Clone, Serialize)]
pub struct CaptureStatus {
    pub is_running: bool,
}

impl CaptureStatus {
    pub fn default() -> Self {
        Self { is_running: false }
    }
    pub fn toggle(&mut self) {
        self.is_running = !self.is_running;
    }
}
