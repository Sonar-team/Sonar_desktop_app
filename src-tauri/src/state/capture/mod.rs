//! État de la capture réseau : configuration, machine d'état, handle des
//! threads en cours et channel d'événements vers le frontend.

use capture_config::CaptureConfig;
use capture_handle::CaptureHandle;
use capture_status::{CapturePhase, CaptureStatus};

use crate::{errors::CaptureStateError, events::CaptureEvent};
use tauri::ipc::Channel;

pub mod capture_config;
pub mod capture_handle;
pub mod capture_status;

/// État global d'une session de capture, partagé via `Arc<Mutex<…>>`.
pub struct CaptureState {
    /// Threads et canaux de la capture en cours (`None` à l'arrêt).
    pub capture: Option<CaptureHandle>,
    /// Phase courante de la machine d'état (voir [`CapturePhase`]).
    pub phase: CapturePhase,
    /// Identifiant de la session de capture live : incrémenté à chaque
    /// tentative de démarrage, jamais réutilisé. Repris dans tous les
    /// événements du pipeline pour que le frontend ignore ceux d'une
    /// session périmée (0 = hors session, ex. import).
    pub session_id: u64,
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
            phase: CapturePhase::Idle,
            session_id: 0,
            config: CaptureConfig::default(),
            filter: None,
            on_event: None,
        }
    }

    /// Statut exposé au frontend, dérivé de la machine d'état.
    pub fn status(&self) -> CaptureStatus {
        CaptureStatus {
            is_running: self.phase == CapturePhase::Running,
            phase: self.phase,
            session_id: self.session_id,
        }
    }

    /// Entame un démarrage : refuse toute transition concurrente (seul
    /// `Idle` peut démarrer) et alloue l'identifiant de la nouvelle session.
    pub fn begin_start(&mut self) -> Result<u64, CaptureStateError> {
        if self.phase != CapturePhase::Idle {
            return Err(CaptureStateError::InvalidTransition {
                from: self.phase.to_string(),
                to: CapturePhase::Starting.to_string(),
            });
        }
        self.phase = CapturePhase::Starting;
        self.session_id += 1;
        Ok(self.session_id)
    }

    /// Démarrage réussi : le pipeline et son channel deviennent l'état
    /// courant. À n'appeler qu'après [`Self::begin_start`].
    pub fn complete_start(
        &mut self,
        capture: CaptureHandle,
        on_event: Channel<CaptureEvent<'static>>,
    ) {
        debug_assert_eq!(self.phase, CapturePhase::Starting);
        self.capture = Some(capture);
        self.on_event = Some(on_event);
        self.phase = CapturePhase::Running;
    }

    /// Démarrage échoué : retour à `Idle`, rien n'est attaché.
    pub fn abort_start(&mut self) {
        debug_assert_eq!(self.phase, CapturePhase::Starting);
        self.phase = CapturePhase::Idle;
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
        self.phase = CapturePhase::Idle;
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
        assert_eq!(state.phase, CapturePhase::Idle);
    }

    #[test]
    fn reap_ignores_a_live_capture() {
        let mut state = CaptureState::new();
        state.capture = Some(CaptureHandle::new(1));
        state.phase = CapturePhase::Running;

        assert!(!state.reap_terminated_capture(), "handle vivant : intouché");
        assert!(state.capture.is_some());
        assert_eq!(state.phase, CapturePhase::Running);
    }

    /// Scénario « erreur -> stopped -> redémarrage » : après un arrêt
    /// autonome du pipeline, la récolte libère le handle et normalise le
    /// statut, si bien qu'un start suivant ne répond plus « déjà en cours ».
    #[test]
    fn reap_frees_a_terminated_capture_for_restart() {
        let mut state = CaptureState::new();
        state.capture = Some(CaptureHandle::terminated_for_tests());
        state.phase = CapturePhase::Running;

        assert!(state.reap_terminated_capture());
        assert!(state.capture.is_none(), "le handle mort est libéré");
        assert_eq!(state.phase, CapturePhase::Idle, "statut backend normalisé");
        assert!(state.on_event.is_none(), "channel IPC mort détaché");
    }

    /// Transitions refusées : un démarrage n'est possible que depuis `Idle`.
    #[test]
    fn begin_start_refuses_concurrent_transitions() {
        let mut state = CaptureState::new();

        let first = state.begin_start().expect("démarrage depuis Idle");
        assert_eq!(first, 1);
        assert_eq!(state.phase, CapturePhase::Starting);

        assert!(
            state.begin_start().is_err(),
            "double start refusé pendant Starting"
        );

        state.complete_start(CaptureHandle::new(first), Channel::new(|_| Ok(())));
        assert_eq!(state.phase, CapturePhase::Running);
        assert!(state.begin_start().is_err(), "start refusé pendant Running");
    }

    /// Chaque tentative de démarrage consomme un identifiant de session,
    /// même en cas d'échec : un id n'est jamais réutilisé.
    #[test]
    fn session_ids_are_never_reused() {
        let mut state = CaptureState::new();

        let first = state.begin_start().unwrap();
        state.abort_start();
        let second = state.begin_start().unwrap();

        assert_eq!(first, 1);
        assert_eq!(second, 2, "l'id de la tentative échouée n'est pas repris");
        assert_eq!(state.status().session_id, 2);
    }
}
