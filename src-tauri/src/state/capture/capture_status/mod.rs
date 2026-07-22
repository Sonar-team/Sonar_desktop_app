//! Machine d'état de la capture et statut exposé au frontend.

use serde::Serialize;

/// Phase du cycle de vie de la capture. Les transitions valides sont
/// `Idle → Starting → Running → Stopping → Idle` ; un échec de démarrage
/// ramène `Starting → Idle`. Un import (PCAP ou matrice CSV) réserve
/// `Idle → Importing → Idle` pendant TOUTE la conversion : capture et import
/// sont mutuellement exclusifs (#139). Toute autre transition est refusée
/// par [`super::CaptureState`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapturePhase {
    Idle,
    Starting,
    Running,
    Stopping,
    Importing,
}

impl std::fmt::Display for CapturePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Idle => "idle",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Importing => "importing",
        };
        f.write_str(name)
    }
}

/// Statut courant de la capture, renvoyé par les commandes
/// `start_capture`/`stop_capture`. `session_id` identifie la session de
/// capture live en cours (0 tant qu'aucune capture n'a démarré) ; le
/// frontend s'en sert pour ignorer les événements d'une session périmée.
#[derive(Clone, Serialize)]
pub struct CaptureStatus {
    pub is_running: bool,
    pub phase: CapturePhase,
    pub session_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_the_snake_case_serde_form() {
        assert_eq!(CapturePhase::Idle.to_string(), "idle");
        assert_eq!(CapturePhase::Starting.to_string(), "starting");
        assert_eq!(CapturePhase::Running.to_string(), "running");
        assert_eq!(CapturePhase::Stopping.to_string(), "stopping");
        assert_eq!(CapturePhase::Importing.to_string(), "importing");
    }
}
