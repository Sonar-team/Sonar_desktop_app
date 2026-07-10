//! État de la capture réseau : configuration, statut, handle des threads en
//! cours et channel d'événements vers le frontend.

use capture_config::CaptureConfig;
use capture_handle::CaptureHandle;
use capture_status::CaptureStatus;

use crate::events::CaptureEvent;
use tauri::ipc::Channel;

pub mod capture_config;
pub mod capture_handle;
pub mod capture_status;

/// État global d'une session de capture, partagé via `Arc<Mutex<…>>`.
pub struct CaptureState {
    /// Threads et canaux de la capture en cours (`None` à l'arrêt).
    pub capture: Option<CaptureHandle>,
    /// Statut courant (démarrée/arrêtée, interface, compteurs).
    pub status: CaptureStatus,
    /// Configuration appliquée au prochain démarrage.
    pub config: CaptureConfig,
    /// Filtre BPF actif, s'il y en a un.
    pub filter: Option<String>,
    /// Channel d'événements de la capture live : les commandes d'import s'en
    /// servent aussi pour joindre le front pendant une capture.
    pub on_event: Option<Channel<CaptureEvent<'static>>>,
}

impl CaptureState {
    /// État initial : aucune capture, statut et configuration par défaut.
    pub fn new() -> Self {
        Self {
            capture: None,
            status: CaptureStatus::default(),
            config: CaptureConfig::default(),
            filter: None,
            on_event: None,
        }
    }

    /// Récolte un pipeline qui s'est arrêté de lui-même (erreur pcap, canal
    /// IPC cassé) : joint les threads, libère le handle et normalise le
    /// statut, pour qu'un redémarrage ne réponde pas « déjà en cours ».
    /// Retourne vrai si un handle terminé a été récolté.
    pub fn reap_terminated_capture(&mut self) -> bool {
        if !self.capture.as_ref().is_some_and(|c| c.is_terminated()) {
            return false;
        }
        if let Some(capture) = self.capture.take() {
            capture.join_threads();
        }
        self.status.is_running = false;
        self.on_event = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reap_without_capture_does_nothing() {
        let mut state = CaptureState::new();
        assert!(!state.reap_terminated_capture());
        assert!(!state.status.is_running);
    }

    #[test]
    fn reap_ignores_a_live_capture() {
        let mut state = CaptureState::new();
        state.capture = Some(CaptureHandle::new());
        state.status.is_running = true;

        assert!(!state.reap_terminated_capture(), "handle vivant : intouché");
        assert!(state.capture.is_some());
        assert!(state.status.is_running);
    }

    /// Scénario « erreur -> stopped -> redémarrage » : après un arrêt
    /// autonome du pipeline, la récolte libère le handle et normalise le
    /// statut, si bien qu'un start suivant ne répond plus « déjà en cours ».
    #[test]
    fn reap_frees_a_terminated_capture_for_restart() {
        let mut state = CaptureState::new();
        state.capture = Some(CaptureHandle::terminated_for_tests());
        state.status.is_running = true;

        assert!(state.reap_terminated_capture());
        assert!(state.capture.is_none(), "le handle mort est libéré");
        assert!(!state.status.is_running, "statut backend normalisé");
        assert!(state.on_event.is_none(), "channel IPC mort détaché");
    }
}
