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
    Arc,
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

/// Poignée d'une capture en cours : drapeau d'arrêt partagé et threads du
/// pipeline, joints à l'arrêt.
pub struct CaptureHandle {
    /// Identifiant de la session, repris dans tous les événements émis.
    session_id: u64,
    stop_flag: Arc<AtomicBool>,
    /// Threads capture + processing, joints au `stop()` pour garantir qu'un
    /// redémarrage immédiat ne fasse pas cohabiter deux pipelines.
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl CaptureHandle {
    pub fn new(session_id: u64) -> Self {
        Self {
            session_id,
            stop_flag: Arc::new(AtomicBool::new(false)),
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
            self.session_id,
            parser_link_type,
        ));
        self.threads.push(spawn_capture_thread_with_pool(
            tx,
            on_event,
            cap,
            stop_flag,
            config.chan_capacity,
            arc_buffer_pool,
            drop_counters,
            shared_stats,
            self.session_id,
        ));

        Ok(())
    }

    /// Handle simulant un pipeline arrêté de lui-même, pour les tests du
    /// cycle de vie (récolte par `CaptureState::reap_terminated_capture`).
    #[cfg(test)]
    pub(crate) fn terminated_for_tests() -> Self {
        let handle = Self::new(1);
        handle.stop_flag.store(true, Ordering::Relaxed);
        handle
    }

    /// Vrai si le pipeline s'est arrêté de lui-même (erreur pcap, canal IPC
    /// cassé) : le drapeau d'arrêt a été levé par un thread, ou tous les
    /// threads sont terminés, sans passer par [`CaptureHandle::stop`].
    pub fn is_terminated(&self) -> bool {
        self.stop_flag.load(Ordering::Relaxed)
            || (!self.threads.is_empty() && self.threads.iter().all(|t| t.is_finished()))
    }

    /// Joint les threads d'un pipeline déjà arrêté, sans émettre d'événement
    /// (le canal IPC est en général mort dans ce scénario).
    pub fn join_threads(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        for handle in self.threads.drain(..) {
            if handle.join().is_err() {
                log::error!("Un thread de capture a paniqué avant l'arrêt");
            }
        }
    }

    /// Arrête le pipeline et attend la fin des threads (borné par le timeout
    /// pcap et l'intervalle de batch, ~100 ms avec la config par défaut).
    pub fn stop(mut self, on_event: Channel<CaptureEvent<'static>>) -> Result<(), CaptureError> {
        info!("Arrêt de la capture demandé");
        self.stop_flag.store(true, Ordering::Relaxed);
        for handle in self.threads.drain(..) {
            if handle.join().is_err() {
                log::error!("Un thread de capture a paniqué avant l'arrêt");
            }
        }
        on_event.send(CaptureEvent::Stopped {
            session_id: self.session_id,
            reason: "arrêt demandé".to_string(),
        })?;
        // Pas de Stats à zéro après l'arrêt : les derniers compteurs restent
        // affichés (le relevé final), un reset les effaçait (#154).
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        handle.join_threads();
    }
}
