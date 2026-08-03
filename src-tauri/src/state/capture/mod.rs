//! État de la capture réseau : configuration, machine d'état, handle des
//! threads en cours et channel d'événements vers le frontend.

use std::sync::{Arc, Mutex, Weak};

use capture_config::CaptureConfig;
use capture_handle::CaptureHandle;
use capture_status::{CapturePhase, CaptureStatus};
use crossbeam::channel::Receiver;

use crate::{errors::CaptureStateError, events::CaptureEvent};
use tauri::ipc::Channel;

pub mod capture_config;
pub mod capture_handle;
pub mod capture_status;

/// Handle retiré de l'état avant une jointure. Le `session_id` empêche le
/// finaliseur d'un ancien arrêt de normaliser une session plus récente.
pub(crate) struct DetachedCapture {
    pub(crate) session_id: u64,
    pub(crate) handle: CaptureHandle,
    pub(crate) on_event: Option<Channel<CaptureEvent<'static>>>,
}

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
    /// Génération monotone des opérations destructives réservées. Elle est
    /// distincte de `session_id` (capture live) et empêche un ancien import
    /// de committer le snapshot construit pour une réservation périmée.
    operation_generation: u64,
    /// Identifiant de la réservation qui possède actuellement la phase
    /// `Importing` (`None` hors opération destructive).
    active_operation_id: Option<u64>,
    /// Configuration appliquée au prochain démarrage.
    pub config: CaptureConfig,
    /// Filtre BPF actif, s'il y en a un.
    pub filter: Option<String>,
    /// Channel d'événements de la capture live : les commandes d'import s'en
    /// servent aussi pour joindre le front pendant une capture.
    pub on_event: Option<Channel<CaptureEvent<'static>>>,
    /// Demande d'annulation de l'import en cours, consultée sans verrou par
    /// la boucle d'import. Remise à zéro par chaque nouvelle réservation
    /// (`ImportGuard::acquire`) : une annulation périmée ne peut pas
    /// interrompre l'import suivant.
    import_cancel: Arc<std::sync::atomic::AtomicBool>,
    /// Révision du relevé : incrémentée à chaque opération qui produit ou
    /// modifie des données (démarrage de capture, imports, édition de
    /// labels). Comparée à `saved_revision` pour savoir si du travail non
    /// enregistré serait perdu (#159). Granularité volontairement à
    /// l'opération, pas au paquet : le pipeline de capture n'a pas à
    /// verrouiller cet état.
    revision: u64,
    /// Révision au moment du dernier enregistrement de projet (ou du
    /// dernier point où il n'y avait rien à perdre : reset, ouverture).
    saved_revision: u64,
}

impl CaptureState {
    /// État initial : aucune capture, statut et configuration par défaut.
    pub fn new() -> Self {
        Self {
            capture: None,
            phase: CapturePhase::Idle,
            session_id: 0,
            operation_generation: 0,
            active_operation_id: None,
            config: CaptureConfig::default(),
            filter: None,
            on_event: None,
            import_cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            revision: 0,
            saved_revision: 0,
        }
    }

    /// Marque le relevé comme modifié depuis le dernier enregistrement.
    pub fn mark_dirty(&mut self) {
        self.revision += 1;
    }

    /// Révision courante, à capturer AVANT un enregistrement long : si des
    /// modifications arrivent pendant l'écriture, `mark_clean_at` avec cette
    /// valeur les laisse comptées comme non enregistrées.
    pub fn current_revision(&self) -> u64 {
        self.revision
    }

    /// Marque comme enregistré tout ce qui existait à `revision`.
    pub fn mark_clean_at(&mut self, revision: u64) {
        self.saved_revision = revision;
    }

    /// Marque l'état courant comme n'ayant rien à perdre (reset, projet
    /// fraîchement ouvert ou enregistré à l'instant).
    pub fn mark_clean(&mut self) {
        self.saved_revision = self.revision;
    }

    /// Vrai si des données seraient perdues sans enregistrement.
    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    /// Demande l'annulation de l'import en cours. Sans import actif
    /// (course avec la fin de l'import), la demande est ignorée : retourne
    /// vrai seulement si un import va effectivement être interrompu.
    pub fn request_import_cancel(&self) -> bool {
        if self.phase != CapturePhase::Importing {
            return false;
        }
        self.import_cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        true
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
        session_id: u64,
        capture: CaptureHandle,
        on_event: Channel<CaptureEvent<'static>>,
    ) -> Result<(), CaptureStateError> {
        if self.phase != CapturePhase::Starting || self.session_id != session_id {
            return Err(CaptureStateError::InvalidTransition {
                from: format!("{} (session {})", self.phase, self.session_id),
                to: format!("running pour la session {session_id}"),
            });
        }
        self.capture = Some(capture);
        self.on_event = Some(on_event);
        self.phase = CapturePhase::Running;
        Ok(())
    }

    /// Démarrage échoué : retour à `Idle`, rien n'est attaché.
    pub fn abort_start(&mut self, session_id: u64) -> Result<(), CaptureStateError> {
        if self.phase != CapturePhase::Starting || self.session_id != session_id {
            return Err(CaptureStateError::InvalidTransition {
                from: format!("{} (session {})", self.phase, self.session_id),
                to: format!("annulation du démarrage de la session {session_id}"),
            });
        }
        self.phase = CapturePhase::Idle;
        Ok(())
    }

    /// Extrait un pipeline qui s'est arrêté de lui-même et réserve la phase
    /// `Stopping`. La jointure est volontairement laissée à l'orchestrateur :
    /// aucun thread ne doit être joint pendant que `CaptureState` est verrouillé
    /// (#166).
    fn take_terminated_capture(
        &mut self,
        expected_session: Option<u64>,
    ) -> Option<DetachedCapture> {
        if self.phase != CapturePhase::Running
            || expected_session.is_some_and(|expected| expected != self.session_id)
            || !self.capture.as_ref().is_some_and(|c| c.is_terminated())
        {
            return None;
        }
        self.phase = CapturePhase::Stopping;
        let on_event = self.on_event.clone();
        self.capture.take().map(|handle| DetachedCapture {
            session_id: self.session_id,
            handle,
            on_event,
        })
    }

    /// Réserve un arrêt explicite et extrait le handle sous le verrou. Le
    /// handle est ensuite stoppé/joint après libération du verrou.
    pub(crate) fn begin_stop(&mut self) -> Result<Option<DetachedCapture>, CaptureStateError> {
        match self.phase {
            CapturePhase::Idle if self.capture.is_none() => Ok(None),
            CapturePhase::Running => {
                let Some(handle) = self.capture.take() else {
                    return Err(CaptureStateError::InvalidTransition {
                        from: "running sans handle".to_string(),
                        to: CapturePhase::Stopping.to_string(),
                    });
                };
                self.phase = CapturePhase::Stopping;
                let on_event = self.on_event.clone();
                Ok(Some(DetachedCapture {
                    session_id: self.session_id,
                    handle,
                    on_event,
                }))
            }
            _ => Err(CaptureStateError::InvalidTransition {
                from: self.phase.to_string(),
                to: CapturePhase::Stopping.to_string(),
            }),
        }
    }

    /// Termine une jointure déjà effectuée hors verrou.
    pub(crate) fn complete_stop(
        &mut self,
        session_id: u64,
    ) -> Result<CaptureStatus, CaptureStateError> {
        if self.phase != CapturePhase::Stopping
            || self.capture.is_some()
            || self.session_id != session_id
        {
            return Err(CaptureStateError::InvalidTransition {
                from: format!(
                    "{} (session {}, handle={})",
                    self.phase,
                    self.session_id,
                    self.capture.is_some()
                ),
                to: format!("finalisation de l'arrêt de la session {session_id}"),
            });
        }
        self.phase = CapturePhase::Idle;
        self.on_event = None;
        Ok(self.status())
    }
}

/// Récolte un pipeline terminé sans jamais conserver le mutex de
/// [`CaptureState`] pendant la jointure des threads. Pendant la jointure, la
/// phase reste `Stopping` : tout démarrage/import concurrent est refusé de
/// façon déterministe.
pub fn reap_terminated_capture(
    state: &Arc<Mutex<CaptureState>>,
) -> Result<bool, CaptureStateError> {
    reap_terminated_capture_for_session(state, None)
}

fn reap_terminated_capture_for_session(
    state: &Arc<Mutex<CaptureState>>,
    expected_session: Option<u64>,
) -> Result<bool, CaptureStateError> {
    let capture = {
        let mut locked = state.lock()?;
        locked.take_terminated_capture(expected_session)
    };

    let Some(capture) = capture else {
        return Ok(false);
    };

    let termination = capture.handle.join_threads();

    {
        let mut locked = state.lock()?;
        locked.complete_stop(capture.session_id)?;
    }

    // La phase est déjà `Idle` quand le frontend observe `Stopped`. Pour un
    // pipeline autonome, le channel peut être mort : l'envoi reste best-effort.
    if let Some(on_event) = capture.on_event
        && let Err(e) = termination.publish(&on_event)
    {
        log::error!("Impossible de publier la fin autonome de la capture: {e}");
    }
    Ok(true)
}

/// Coordinateur détaché d'une session live. Le premier thread qui termine
/// lève le drapeau commun via [`capture_handle::PipelineThreadGuard`] ; le
/// coordinateur retire alors le handle sous `CaptureState`, joint les deux
/// threads hors verrou et normalise la phase. `Weak` évite tout cycle avec
/// l'état qui possède le handle.
pub(crate) fn spawn_pipeline_reaper(
    state: Weak<Mutex<CaptureState>>,
    session_id: u64,
    completion: Receiver<()>,
) {
    std::thread::spawn(move || {
        if completion.recv().is_err() {
            return;
        }
        let Some(state) = state.upgrade() else {
            return;
        };
        if let Err(e) = reap_terminated_capture_for_session(&state, Some(session_id)) {
            log::error!("Erreur de finalisation autonome de la session {session_id}: {e}");
        }
    });
}

/// Réservation RAII de la phase [`CapturePhase::Importing`] (#139).
///
/// La vérification « aucune capture en cours » et la bascule en `Importing`
/// se font sous **le même verrou** : rien ne peut s'intercaler entre le
/// contrôle et la réservation. La réservation est ensuite détenue pendant
/// toute la vie du guard — donc pendant TOUTE la conversion, jusqu'au swap
/// de la matrice inclus — et un `start_capture` concurrent est refusé par
/// la machine d'état (`InvalidTransition`) au lieu de démarrer un pipeline
/// dont le relevé serait écrasé en fin d'import.
///
/// La phase revient à `Idle` au `Drop`, sur tous les chemins de sortie :
/// succès, erreur (`?` en cours de conversion) ou panique.
pub struct ImportGuard {
    state: Arc<Mutex<CaptureState>>,
    operation_id: u64,
    cancel: Arc<std::sync::atomic::AtomicBool>,
}

impl ImportGuard {
    /// Vérifie `Idle` et bascule en `Importing`, atomiquement. `operation`
    /// nomme l'import refusé dans l'erreur (`InvalidTransition`) si une
    /// capture tourne déjà. Récolte d'abord un pipeline mort pour ne pas
    /// refuser à tort après un arrêt autonome.
    pub fn acquire(
        state: &Arc<Mutex<CaptureState>>,
        operation: &str,
    ) -> Result<Self, CaptureStateError> {
        reap_terminated_capture(state)?;
        let mut locked = state.lock()?;
        if locked.phase != CapturePhase::Idle {
            return Err(CaptureStateError::InvalidTransition {
                from: locked.phase.to_string(),
                to: operation.to_string(),
            });
        }
        locked.operation_generation = locked.operation_generation.wrapping_add(1);
        // 0 reste la valeur sentinelle « aucune opération » même après un
        // débordement théorique du compteur.
        if locked.operation_generation == 0 {
            locked.operation_generation = 1;
        }
        let operation_id = locked.operation_generation;
        locked.active_operation_id = Some(operation_id);
        locked.phase = CapturePhase::Importing;
        // Une demande d'annulation d'une opération précédente ne doit jamais
        // interrompre cette nouvelle réservation.
        locked
            .import_cancel
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let cancel = Arc::clone(&locked.import_cancel);
        drop(locked);
        Ok(Self {
            state: Arc::clone(state),
            operation_id,
            cancel,
        })
    }

    /// Jeton d'annulation de cette réservation, consultable sans verrou par
    /// la boucle d'import (chargements `Relaxed` : simple drapeau, aucune
    /// donnée synchronisée par sa lecture).
    pub fn cancel_token(&self) -> Arc<std::sync::atomic::AtomicBool> {
        Arc::clone(&self.cancel)
    }

    /// Vérifie que le résultat sur le point d'être committé appartient
    /// toujours à la réservation qui l'a construit. Un guard périmé ne peut
    /// donc jamais remplacer la matrice, le graphe ou les labels courants.
    pub fn verify_current(&self, commit: &str) -> Result<(), CaptureStateError> {
        let locked = self.state.lock()?;
        if locked.phase != CapturePhase::Importing
            || locked.active_operation_id != Some(self.operation_id)
        {
            return Err(CaptureStateError::InvalidTransition {
                from: format!(
                    "{} (opération active {:?})",
                    locked.phase, locked.active_operation_id
                ),
                to: commit.to_string(),
            });
        }
        Ok(())
    }
}

impl Drop for ImportGuard {
    fn drop(&mut self) {
        // Restitution best-effort : même si un thread a paniqué avec le
        // verrou (mutex empoisonné), la phase doit être rendue, sinon toute
        // capture serait refusée à jamais.
        let mut locked = match self.state.lock() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        };
        // Un guard périmé ne doit surtout pas libérer la réservation d'une
        // opération plus récente.
        if locked.active_operation_id == Some(self.operation_id) {
            debug_assert_eq!(locked.phase, CapturePhase::Importing);
            locked.active_operation_id = None;
            locked.phase = CapturePhase::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel::bounded;
    use std::{thread, time::Duration};

    #[test]
    fn new_state_is_clean() {
        assert!(!CaptureState::new().is_dirty());
    }

    #[test]
    fn mark_dirty_then_clean_round_trip() {
        let mut state = CaptureState::new();
        state.mark_dirty();
        assert!(state.is_dirty());
        state.mark_clean();
        assert!(!state.is_dirty());
    }

    #[test]
    fn modifications_during_a_save_stay_dirty() {
        let mut state = CaptureState::new();
        state.mark_dirty();
        let snapshot = state.current_revision();
        // Une mutation arrive pendant l'écriture du projet…
        state.mark_dirty();
        // …l'enregistrement ne blanchit que ce qui existait au snapshot.
        state.mark_clean_at(snapshot);
        assert!(
            state.is_dirty(),
            "les modifications postérieures au snapshot doivent rester non enregistrées"
        );
    }

    #[test]
    fn reap_without_capture_does_nothing() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        assert!(!reap_terminated_capture(&state).unwrap());
        assert_eq!(state.lock().unwrap().phase, CapturePhase::Idle);
    }

    #[test]
    fn reap_ignores_a_live_capture() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.capture = Some(CaptureHandle::new(1));
            locked.phase = CapturePhase::Running;
        }

        assert!(
            !reap_terminated_capture(&state).unwrap(),
            "handle vivant : intouché"
        );
        let locked = state.lock().unwrap();
        assert!(locked.capture.is_some());
        assert_eq!(locked.phase, CapturePhase::Running);
    }

    /// Scénario « erreur -> stopped -> redémarrage » : après un arrêt
    /// autonome du pipeline, la récolte libère le handle et normalise le
    /// statut, si bien qu'un start suivant ne répond plus « déjà en cours ».
    #[test]
    fn reap_frees_a_terminated_capture_for_restart() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.capture = Some(CaptureHandle::terminated_for_tests(0));
            locked.phase = CapturePhase::Running;
        }

        assert!(reap_terminated_capture(&state).unwrap());
        let locked = state.lock().unwrap();
        assert!(locked.capture.is_none(), "le handle mort est libéré");
        assert_eq!(locked.phase, CapturePhase::Idle, "statut backend normalisé");
        assert!(locked.on_event.is_none(), "channel IPC mort détaché");
    }

    #[test]
    fn stale_auto_reaper_cannot_take_a_newer_session() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.session_id = 12;
            locked.capture = Some(CaptureHandle::terminated_for_tests(12));
            locked.phase = CapturePhase::Running;
        }

        assert!(!reap_terminated_capture_for_session(&state, Some(11)).unwrap());
        {
            let locked = state.lock().unwrap();
            assert_eq!(locked.phase, CapturePhase::Running);
            assert!(locked.capture.is_some());
        }

        assert!(reap_terminated_capture_for_session(&state, Some(12)).unwrap());
        assert_eq!(state.lock().unwrap().phase, CapturePhase::Idle);
    }

    /// La récolte autonome suit la même discipline que l'arrêt explicite :
    /// elle extrait le handle, libère `CaptureState`, puis seulement joint le
    /// thread. Le timeout rend la reproduction de l'ancien deadlock bornée.
    #[test]
    fn reap_releases_capture_state_before_join() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        let matrix = Arc::new(Mutex::new(()));
        let matrix_guard = matrix.lock().unwrap();
        let (worker_blocked_tx, worker_blocked_rx) = bounded::<()>(1);

        let matrix_for_worker = Arc::clone(&matrix);
        let handle = CaptureHandle::with_thread_for_tests(9, move |stop_flag| {
            // Simule un pipeline qui s'arrête seul, puis dont l'épilogue a
            // encore besoin d'un verrou partagé avant de pouvoir être joint.
            stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
            worker_blocked_tx.send(()).unwrap();
            let _matrix = matrix_for_worker.lock().unwrap();
        });
        worker_blocked_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("le faux pipeline atteint son épilogue");

        {
            let mut locked = state.lock().unwrap();
            locked.session_id = 9;
            locked.phase = CapturePhase::Running;
            locked.capture = Some(handle);
            locked.on_event = Some(Channel::new(|_| Ok(())));
        }

        let state_for_reap = Arc::clone(&state);
        let (reap_started_tx, reap_started_rx) = bounded::<()>(1);
        let (reap_done_tx, reap_done_rx) = bounded(1);
        thread::spawn(move || {
            reap_started_tx.send(()).unwrap();
            let result = reap_terminated_capture(&state_for_reap).map_err(|e| e.to_string());
            let _ = reap_done_tx.send(result);
        });
        reap_started_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("la récolte est lancée");

        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        let state_available = loop {
            if let Ok(locked) = state.try_lock()
                && locked.phase == CapturePhase::Stopping
            {
                break true;
            }
            if std::time::Instant::now() >= deadline {
                break false;
            }
            thread::yield_now();
        };

        // Nettoyage avant les assertions : même une régression libère les
        // deux threads du test au lieu de suspendre le runner.
        drop(matrix_guard);
        let reap_result = reap_done_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("la récolte termine après libération de la matrice");

        assert!(
            state_available,
            "CaptureState ne doit pas être conservé pendant la jointure de récolte"
        );
        assert!(reap_result.unwrap());
        assert_eq!(state.lock().unwrap().phase, CapturePhase::Idle);
    }

    fn assert_guarded_pipeline_finalizes_automatically(
        panic_first: bool,
        expected_reason_fragment: &str,
    ) {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        let session_id = state.lock().unwrap().begin_start().unwrap();
        let mut handle = CaptureHandle::guarded_fatal_pipeline_for_tests(
            session_id,
            "erreur fatale d'intégrité : verrou de la matrice empoisonné",
            panic_first,
        );
        let completion = handle
            .take_completion_receiver()
            .expect("coordinateur du pipeline factice");
        let (event_tx, event_rx) = bounded::<serde_json::Value>(2);
        let on_event = Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                event_tx.send(serde_json::from_str(&json).unwrap()).unwrap();
            }
            Ok(())
        });
        state
            .lock()
            .unwrap()
            .complete_start(session_id, handle, on_event)
            .unwrap();

        spawn_pipeline_reaper(Arc::downgrade(&state), session_id, completion);

        let stopped = event_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("le coordinateur doit publier Stopped dans la borne");
        assert_eq!(stopped["event"], "stopped");
        assert!(
            stopped["data"]["reason"]
                .as_str()
                .unwrap()
                .contains(expected_reason_fragment)
        );
        let locked = state.lock().unwrap();
        assert_eq!(
            locked.phase,
            CapturePhase::Idle,
            "la phase est normalisée avant l'événement terminal"
        );
        assert!(locked.capture.is_none());
        assert!(locked.on_event.is_none());
        drop(locked);
        assert!(
            event_rx.recv_timeout(Duration::from_millis(25)).is_err(),
            "une session ne publie qu'un seul événement terminal"
        );
    }

    #[test]
    fn fatal_pipeline_auto_reaper_returns_idle_and_publishes_once() {
        assert_guarded_pipeline_finalizes_automatically(false, "matrice empoisonné");
    }

    #[test]
    fn panicking_pipeline_auto_reaper_stops_sibling_and_returns_idle() {
        assert_guarded_pipeline_finalizes_automatically(true, "paniqué");
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

        state
            .complete_start(first, CaptureHandle::new(first), Channel::new(|_| Ok(())))
            .unwrap();
        assert_eq!(state.phase, CapturePhase::Running);
        assert!(state.begin_start().is_err(), "start refusé pendant Running");
    }

    /// La réservation `Importing` est atomique et exclusive : pendant toute
    /// la vie du guard, un démarrage de capture est refusé — c'est le trou
    /// de #139 (check-then-act) qui est fermé ici.
    #[test]
    fn import_guard_blocks_capture_start_for_its_whole_lifetime() {
        let state = Arc::new(Mutex::new(CaptureState::new()));

        let guard = ImportGuard::acquire(&state, "import de test").expect("état Idle");
        assert_eq!(state.lock().unwrap().phase, CapturePhase::Importing);

        let err = state.lock().unwrap().begin_start();
        assert!(
            matches!(
                err,
                Err(CaptureStateError::InvalidTransition { ref from, .. }) if from == "importing"
            ),
            "un start pendant l'import doit être refusé par la machine d'état"
        );

        drop(guard);
        assert_eq!(
            state.lock().unwrap().phase,
            CapturePhase::Idle,
            "la phase est rendue au drop du guard"
        );
        state
            .lock()
            .unwrap()
            .begin_start()
            .expect("le démarrage redevient possible après l'import");
    }

    /// L'inverse tient aussi : une capture en cours refuse la réservation,
    /// et deux imports ne peuvent pas se chevaucher.
    #[test]
    fn import_guard_is_refused_while_running_or_importing() {
        let state = Arc::new(Mutex::new(CaptureState::new()));

        let session = state.lock().unwrap().begin_start().unwrap();
        state
            .lock()
            .unwrap()
            .complete_start(
                session,
                CaptureHandle::new(session),
                Channel::new(|_| Ok(())),
            )
            .unwrap();
        assert!(
            ImportGuard::acquire(&state, "import de test").is_err(),
            "import refusé pendant Running"
        );

        // Retour à Idle : la capture est retirée (équivalent d'un stop).
        {
            let mut locked = state.lock().unwrap();
            locked.capture = None;
            locked.phase = CapturePhase::Idle;
        }

        let first = ImportGuard::acquire(&state, "premier import").expect("état Idle");
        assert!(
            ImportGuard::acquire(&state, "second import").is_err(),
            "deux imports ne se chevauchent pas"
        );
        drop(first);
    }

    /// Le guard rend la phase même quand l'import échoue en cours de route
    /// (early return `?`) : simulé par un drop dans un scope d'erreur.
    #[test]
    fn import_guard_restores_idle_on_error_paths() {
        let state = Arc::new(Mutex::new(CaptureState::new()));

        let failing_import = |state: &Arc<Mutex<CaptureState>>| -> Result<(), CaptureStateError> {
            let _guard = ImportGuard::acquire(state, "import qui échoue")?;
            Err(CaptureStateError::PoisonError("erreur simulée".to_string()))
        };

        assert!(failing_import(&state).is_err());
        assert_eq!(
            state.lock().unwrap().phase,
            CapturePhase::Idle,
            "la phase est rendue malgré l'échec de l'import"
        );
    }

    #[test]
    fn stale_import_guard_cannot_commit_or_release_a_newer_operation() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        let stale = ImportGuard::acquire(&state, "ancien import").unwrap();

        // Simule une réservation remplacée entre la construction du snapshot
        // et son commit. Les champs sont privés et ne peuvent être modifiés
        // ainsi qu'ici, mais ce test verrouille le contrat de génération.
        {
            let mut locked = state.lock().unwrap();
            locked.operation_generation += 1;
            locked.active_operation_id = Some(locked.operation_generation);
        }

        assert!(
            matches!(
                stale.verify_current("commit de l'ancien import"),
                Err(CaptureStateError::InvalidTransition { .. })
            ),
            "un snapshot construit par une réservation périmée est refusé"
        );
        drop(stale);

        let locked = state.lock().unwrap();
        assert_eq!(locked.phase, CapturePhase::Importing);
        assert_eq!(locked.active_operation_id, Some(2));
    }

    /// Chaque tentative de démarrage consomme un identifiant de session,
    /// même en cas d'échec : un id n'est jamais réutilisé.
    #[test]
    fn session_ids_are_never_reused() {
        let mut state = CaptureState::new();

        let first = state.begin_start().unwrap();
        state.abort_start(first).unwrap();
        let second = state.begin_start().unwrap();

        assert_eq!(first, 1);
        assert_eq!(second, 2, "l'id de la tentative échouée n'est pas repris");
        assert_eq!(state.status().session_id, 2);
    }

    #[test]
    fn stale_stop_cannot_finalize_another_session() {
        let mut state = CaptureState::new();
        state.phase = CapturePhase::Stopping;
        state.session_id = 12;

        assert!(
            state.complete_stop(11).is_err(),
            "le finaliseur d'une ancienne session doit être refusé"
        );
        assert_eq!(state.phase, CapturePhase::Stopping);
        assert_eq!(state.session_id, 12);

        state.complete_stop(12).unwrap();
        assert_eq!(state.phase, CapturePhase::Idle);
    }
}
