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
    stop_flag: Arc<AtomicBool>,
    /// Threads capture + processing, joints au `stop()` pour garantir qu'un
    /// redémarrage immédiat ne fasse pas cohabiter deux pipelines.
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl CaptureHandle {
    pub fn new() -> Self {
        println!("[DEBUG] CaptureHandle créé");
        Self {
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
    ) -> Result<(), CaptureError> {
        config.validate()?;
        debug!(
            "Démarrage de la capture sur l'interface {}...",
            config.device_name
        );

        on_event.send(CaptureEvent::Started {
            device: &config.device_name,
            buffer_size: config.buffer_size,
            chan_capacity: config.chan_capacity,
            timeout: config.timeout,
            snaplen: config.snaplen,
        })?;

        let stop_flag = self.stop_flag.clone();

        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == config.device_name)
            .ok_or_else(|| CaptureError::InterfaceNotFound(config.device_name.clone()))?;

        info!("Interface trouvée : {}", device.name);

        let mut cap = setup_capture(config.clone())?;

        setup_filter(&mut cap, filter)?;

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
        ));

        Ok(())
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
            reason: "arrêt demandé".to_string(),
        })?;
        on_event.send(CaptureEvent::Stats {
            received: 0,
            dropped: 0,
            if_dropped: 0,
            app_dropped: 0,
            processed: 0,
        })?;
        Ok(())
    }

    pub fn start_no_event(
        &mut self,
        config: CaptureConfig,
        app: AppHandle,
        filter: Option<String>,
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
            Channel::new(|_| Ok(())),
            config.chan_capacity,
            app.clone(),
            arc_buffer_pool.clone(),
            drop_counters.clone(),
            shared_stats.clone(),
            stop_flag.clone(),
        ));
        self.threads.push(spawn_capture_thread_with_pool(
            tx,
            Channel::new(|_| Ok(())),
            cap,
            stop_flag,
            config.chan_capacity,
            arc_buffer_pool,
            drop_counters,
            shared_stats,
        ));

        Ok(())
    }
}
