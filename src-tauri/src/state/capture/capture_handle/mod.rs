//! Pipeline de capture live : deux threads (capture pcap → canal borné →
//! traitement) pilotés par un [`CaptureHandle`], avec pool de buffers pour
//! éviter les allocations par paquet.

pub mod messages;
pub mod setup;
pub mod threads;

use crossbeam::channel::{Receiver, Sender, bounded};
use log::{debug, info};
use pcap::Device;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use tauri::{AppHandle, ipc::Channel};

use crate::{
    errors::capture_error::CaptureError,
    events::CaptureEvent,
    state::capture::{
        capture_config::CaptureConfig,
        capture_handle::{
            messages::{
                CaptureMessage,
                stats::{AppDropCounters, SharedCaptureStats},
            },
            setup::{setup_capture, setup_filter},
            threads::{
                capture::spawn_capture_thread_with_pool, packet_buffer::PacketBufferPool,
                processing::spawn_processing_thread,
            },
        },
    },
};

/// Causes terminales partagées par les deux threads du pipeline.
///
/// Une corruption/panique a priorité sur une erreur autonome (pcap,
/// plafond, IPC), elle-même prioritaire sur « arrêt demandé ». Les workers
/// ne publient jamais eux-mêmes `Stopped` : l'orchestrateur attend les
/// jointures, normalise d'abord la phase, puis émet exactement une cause.
#[derive(Default)]
pub(crate) struct TerminalState {
    fatal_reason: OnceLock<String>,
    autonomous_reason: OnceLock<String>,
}

impl TerminalState {
    pub(crate) fn record_fatal(&self, reason: String) {
        let _ = self.fatal_reason.set(reason);
    }

    pub(crate) fn record_autonomous(&self, reason: String) {
        let _ = self.autonomous_reason.set(reason);
    }

    pub(crate) fn preferred_reason(&self) -> Option<String> {
        if let Some(reason) = self.fatal_reason.get() {
            return Some(reason.clone());
        }
        self.autonomous_reason.get().cloned()
    }
}

/// Résultat possédé d'une jointure. Sa publication est volontairement
/// séparée : `CaptureState::complete_stop` doit passer à `Idle` avant que le
/// frontend puisse réagir au `Stopped`.
pub(crate) struct CaptureTermination {
    session_id: u64,
    reason: Option<String>,
}

impl CaptureTermination {
    pub(crate) fn publish(
        self,
        on_event: &Channel<CaptureEvent<'static>>,
    ) -> Result<(), CaptureError> {
        if let Some(reason) = self.reason {
            on_event.send(CaptureEvent::Stopped {
                session_id: self.session_id,
                reason,
            })?;
        }
        Ok(())
    }
}

/// Garde posé au début de chaque thread du pipeline. Toute sortie lève le
/// drapeau commun et notifie le coordinateur ; une panique mémorise en plus
/// une cause fatale, qui sera publiée après la jointure (jamais depuis `Drop`).
pub(crate) struct PipelineThreadGuard {
    name: &'static str,
    completion: Sender<()>,
    stop_flag: Arc<AtomicBool>,
    terminal: Arc<TerminalState>,
}

impl PipelineThreadGuard {
    pub(crate) fn new(
        name: &'static str,
        completion: Sender<()>,
        stop_flag: Arc<AtomicBool>,
        terminal: Arc<TerminalState>,
    ) -> Self {
        Self {
            name,
            completion,
            stop_flag,
            terminal,
        }
    }
}

impl Drop for PipelineThreadGuard {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
        if std::thread::panicking() {
            self.terminal.record_fatal(format!(
                "erreur fatale : le thread {} du pipeline a paniqué",
                self.name
            ));
        }
        // Canal borné à deux places pour exactement deux producteurs : ne
        // jamais bloquer pendant un unwind.
        let _ = self.completion.try_send(());
    }
}

/// Poignée d'une capture en cours : drapeau d'arrêt partagé et threads du
/// pipeline, joints à l'arrêt.
pub struct CaptureHandle {
    /// Identifiant de la session, repris dans tous les événements émis.
    session_id: u64,
    stop_flag: Arc<AtomicBool>,
    terminal: Arc<TerminalState>,
    /// Deux notifications, une par thread. Le coordinateur autonome ne
    /// démarre qu'après l'attachement du handle à `CaptureState`.
    completion_rx: Option<Receiver<()>>,
    /// Threads capture + processing, joints au `stop()` pour garantir qu'un
    /// redémarrage immédiat ne fasse pas cohabiter deux pipelines.
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        // Un échec entre le lancement des workers et leur attachement à
        // `CaptureState` ne doit jamais laisser une capture fantôme. Les
        // chemins nominaux ont déjà joint les threads avant ce Drop.
        self.stop_flag.store(true, Ordering::Release);
    }
}

impl CaptureHandle {
    pub fn new(session_id: u64) -> Self {
        Self {
            session_id,
            stop_flag: Arc::new(AtomicBool::new(false)),
            terminal: Arc::new(TerminalState::default()),
            completion_rx: None,
            threads: Vec::new(),
        }
    }

    pub fn start(
        &mut self,
        config: CaptureConfig,
        app: AppHandle,
        on_event: Channel<CaptureEvent<'static>>,
        filter: Option<String>,
        matrix: &mut crate::state::flow_matrix::FlowMatrix,
    ) -> Result<(), CaptureError> {
        config.validate()?;
        debug!(
            "Démarrage de la capture sur l'interface {}...",
            config.device_name
        );

        let stop_flag = self.stop_flag.clone();

        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == config.device_name)
            .ok_or_else(|| CaptureError::InterfaceNotFound(config.device_name.clone()))?;

        info!("Interface trouvée : {}", device.name);

        let mut cap = setup_capture(config.clone())?;

        setup_filter(&mut cap, filter)?;

        // Le DLT de l'interface est refusé avant l'événement `Started` si
        // cette version n'a pas de décodeur pour lui : un DLT non supporté
        // ne doit jamais être parsé comme de l'Ethernet (#150).
        let datalink = cap.get_datalink();
        let parser_link_type = sonar_flows_core::pcap::parser_link_type(datalink);
        if !packet_parser::parse::is_supported(parser_link_type) {
            return Err(CaptureError::UnsupportedLinkType(
                sonar_flows_core::pcap::datalink_label(datalink),
            ));
        }

        // Un relevé = un réseau = un DLT (arbitrage 14/07/2026) : capturer sur
        // une interface d'un autre type de liaison que la matrice en cours
        // est refusé avant tout démarrage.
        let previous_link_type = matrix.link_type;
        matrix
            .bind_link_type(parser_link_type, std::path::Path::new(&config.device_name))
            .map_err(|e| CaptureError::MixedLinkType(e.to_string()))?;

        // `Started` part seulement une fois l'interface ouverte et le filtre
        // appliqué : un échec de démarrage ne produit jamais un « démarré »
        // suivi d'une erreur.
        let link_type = sonar_flows_core::pcap::datalink_label(datalink);
        if let Err(e) = on_event.send(CaptureEvent::Started {
            session_id: self.session_id,
            device: &config.device_name,
            buffer_size: config.buffer_size,
            chan_capacity: config.chan_capacity,
            timeout: config.timeout,
            snaplen: config.snaplen,
            link_type: &link_type,
            protocol_version: crate::events::CAPTURE_EVENT_PROTOCOL_VERSION,
        }) {
            // Démarrage avorté : le DLT éventuellement fixé ci-dessus ne doit
            // pas rester lié à un relevé qui n'a rien reçu.
            matrix.link_type = previous_link_type;
            return Err(e.into());
        }

        let (tx, rx): (Sender<CaptureMessage>, Receiver<CaptureMessage>) =
            bounded(config.chan_capacity as usize);

        // 🔑 Utilisation du nouveau PacketBufferPool
        let arc_buffer_pool = Arc::new(PacketBufferPool::new(
            config.chan_capacity as usize + 2,
            config.snaplen as usize,
        ));
        let drop_counters = Arc::new(AppDropCounters::default());
        let shared_stats = Arc::new(SharedCaptureStats::default());
        let (completion_tx, completion_rx) = bounded(2);
        self.completion_rx = Some(completion_rx);

        // Démarrage des threads avec le nouveau buffer_pool
        self.threads.push(spawn_processing_thread(
            rx,
            on_event.clone(),
            config.chan_capacity,
            app.clone(),
            arc_buffer_pool.clone(),
            drop_counters.clone(),
            shared_stats.clone(),
            stop_flag.clone(),
            Arc::clone(&self.terminal),
            completion_tx.clone(),
            self.session_id,
            parser_link_type,
            std::time::Duration::from_millis(config.timeout.max(0) as u64),
        ));
        self.threads.push(spawn_capture_thread_with_pool(
            tx,
            on_event,
            cap,
            stop_flag,
            Arc::clone(&self.terminal),
            completion_tx,
            config.chan_capacity,
            arc_buffer_pool,
            drop_counters,
            shared_stats,
            self.session_id,
        ));

        Ok(())
    }

    /// Prend le canal de fin une fois les deux threads lancés. Il sera confié
    /// au coordinateur seulement après `complete_start`, pour couvrir un
    /// pipeline qui se termine très vite sans rater son attachement à l'état.
    pub(crate) fn take_completion_receiver(&mut self) -> Option<Receiver<()>> {
        self.completion_rx.take()
    }

    /// Handle simulant un pipeline arrêté de lui-même, pour les tests du
    /// cycle de vie (récolte par `CaptureState::reap_terminated_capture`).
    #[cfg(test)]
    pub(crate) fn terminated_for_tests(session_id: u64) -> Self {
        let handle = Self::new(session_id);
        handle.stop_flag.store(true, Ordering::Relaxed);
        handle
    }

    /// Pipeline factice synchronisable, sans pcap, pour vérifier que les
    /// jointures n'ont lieu sous aucun verrou partagé (#166).
    #[cfg(test)]
    pub(crate) fn with_thread_for_tests<F>(session_id: u64, worker: F) -> Self
    where
        F: FnOnce(Arc<AtomicBool>) + Send + 'static,
    {
        let mut handle = Self::new(session_id);
        let stop_flag = Arc::clone(&handle.stop_flag);
        handle
            .threads
            .push(std::thread::spawn(move || worker(stop_flag)));
        handle
    }

    /// Pipeline à deux workers utilisant les vrais gardes de fin. Le premier
    /// termine avec une cause fatale (ou panique), le second ne peut sortir
    /// qu'après la levée du drapeau commun. Sert à tester le coordinateur
    /// autonome sans ouvrir d'interface pcap.
    #[cfg(test)]
    pub(crate) fn guarded_fatal_pipeline_for_tests(
        session_id: u64,
        reason: &'static str,
        panic_first: bool,
    ) -> Self {
        let mut handle = Self::new(session_id);
        let (completion_tx, completion_rx) = bounded(2);
        handle.completion_rx = Some(completion_rx);

        let first_stop = Arc::clone(&handle.stop_flag);
        let first_terminal = Arc::clone(&handle.terminal);
        let first_completion = completion_tx.clone();
        handle.threads.push(std::thread::spawn(move || {
            let _guard = PipelineThreadGuard::new(
                "processing test",
                first_completion,
                Arc::clone(&first_stop),
                Arc::clone(&first_terminal),
            );
            if panic_first {
                panic!("panique intentionnelle du pipeline pour #166");
            }
            first_terminal.record_fatal(reason.to_string());
        }));

        let second_stop = Arc::clone(&handle.stop_flag);
        let second_terminal = Arc::clone(&handle.terminal);
        handle.threads.push(std::thread::spawn(move || {
            let _guard = PipelineThreadGuard::new(
                "capture test",
                completion_tx,
                Arc::clone(&second_stop),
                second_terminal,
            );
            while !second_stop.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
        }));

        handle
    }

    /// Vrai si le pipeline s'est arrêté de lui-même (erreur pcap, canal IPC
    /// cassé) : le drapeau d'arrêt a été levé par un thread, ou tous les
    /// threads sont terminés, sans passer par [`CaptureHandle::stop`].
    pub fn is_terminated(&self) -> bool {
        self.stop_flag.load(Ordering::Acquire)
            || (!self.threads.is_empty() && self.threads.iter().all(|t| t.is_finished()))
    }

    fn join_all(&mut self) -> bool {
        let mut thread_panicked = false;
        for handle in self.threads.drain(..) {
            if handle.join().is_err() {
                log::error!("Un thread de capture a paniqué avant l'arrêt");
                thread_panicked = true;
            }
        }
        thread_panicked
    }

    /// Joint les threads d'un pipeline déjà arrêté et rend sa cause. Aucun
    /// événement n'est émis ici : la phase doit d'abord être normalisée.
    pub(crate) fn join_threads(mut self) -> CaptureTermination {
        self.stop_flag.store(true, Ordering::Release);
        if self.join_all() {
            self.terminal.record_fatal(
                "erreur fatale : un thread du pipeline de capture a paniqué".to_string(),
            );
        }
        CaptureTermination {
            session_id: self.session_id,
            reason: self.terminal.preferred_reason(),
        }
    }

    /// Arrête le pipeline et attend la fin des threads (borné par le timeout
    /// pcap configuré, 25 ms par défaut, puis la fin du drainage).
    pub(crate) fn stop(mut self) -> CaptureTermination {
        info!("Arrêt de la capture demandé");
        self.stop_flag.store(true, Ordering::Release);
        if self.join_all() {
            self.terminal.record_fatal(
                "erreur fatale : un thread du pipeline de capture a paniqué".to_string(),
            );
        }

        // Pas de Stats à zéro après l'arrêt : les derniers compteurs restent
        // affichés (le relevé final), un reset les effaçait (#154).
        CaptureTermination {
            session_id: self.session_id,
            reason: Some(
                self.terminal
                    .preferred_reason()
                    .unwrap_or_else(|| "arrêt demandé".to_string()),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recording_channel() -> (
        Channel<CaptureEvent<'static>>,
        Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    ) {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        let channel = Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                captured
                    .lock()
                    .unwrap()
                    .push(serde_json::from_str(&json).unwrap());
            }
            Ok(())
        });
        (channel, events)
    }

    #[test]
    fn fresh_handle_is_not_terminated() {
        let handle = CaptureHandle::new(1);
        assert!(!handle.is_terminated(), "handle neuf, rien à récolter");
    }

    #[test]
    fn handle_with_stop_flag_raised_is_terminated() {
        // Simule un pipeline qui s'est arrêté de lui-même (erreur pcap ou
        // canal IPC cassé) : un thread a levé le drapeau avant de sortir.
        let handle = CaptureHandle::new(1);
        handle.stop_flag.store(true, Ordering::Relaxed);
        assert!(handle.is_terminated());
    }

    #[test]
    fn handle_with_finished_threads_is_terminated() {
        let mut handle = CaptureHandle::new(1);
        handle.threads.push(std::thread::spawn(|| {}));
        // Laisse le thread se terminer.
        while !handle.threads[0].is_finished() {
            std::thread::yield_now();
        }
        assert!(handle.is_terminated());
        let termination = handle.join_threads();
        assert!(termination.reason.is_none());
    }

    #[test]
    fn requested_stop_preserves_a_recorded_fatal_reason() {
        let handle = CaptureHandle::new(7);
        handle
            .terminal
            .record_fatal("erreur fatale de test".to_string());
        let termination = handle.stop();
        let (on_event, events) = recording_channel();
        termination.publish(&on_event).unwrap();

        let events = events.lock().unwrap();
        let stopped = events
            .iter()
            .find(|event| event["event"] == "stopped")
            .expect("stop doit retenter le Stopped fatal non livré");
        assert_eq!(
            stopped["data"]["reason"], "erreur fatale de test",
            "la raison fatale ne doit jamais être masquée"
        );
    }

    #[test]
    fn terminal_reason_is_published_once_by_the_orchestrator() {
        let handle = CaptureHandle::new(71);
        handle
            .terminal
            .record_fatal("erreur fatale déjà livrée".to_string());
        let (on_event, events) = recording_channel();

        // Une seule publication appartient à l'orchestrateur après la
        // normalisation de phase ; il n'existe plus d'envoi worker à doubler.
        handle.stop().publish(&on_event).unwrap();

        assert_eq!(events.lock().unwrap().len(), 1);
    }

    #[test]
    fn thread_panic_is_reported_as_a_fatal_stop() {
        let mut handle = CaptureHandle::new(8);
        handle.threads.push(std::thread::spawn(|| {
            panic!("panique intentionnelle du pipeline pour #166");
        }));
        let (on_event, events) = recording_channel();

        handle.stop().publish(&on_event).unwrap();

        let events = events.lock().unwrap();
        let stopped = events
            .iter()
            .find(|event| event["event"] == "stopped")
            .expect("la panique du thread doit produire un Stopped observable");
        assert!(
            stopped["data"]["reason"]
                .as_str()
                .unwrap()
                .contains("paniqué")
        );
    }
}
