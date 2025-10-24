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
    state::capture::capture_handle::{
        messages::CaptureMessage,
        threads::{
            capture::spawn_capture_thread_with_pool, packet_buffer::PacketBufferPool,
            processing::spawn_processing_thread,
        },
    },
};

pub struct CaptureHandle {
    stop_flag: Arc<AtomicBool>,
}

impl CaptureHandle {
    pub fn new() -> Self {
        println!("[DEBUG] CaptureHandle créé");
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(
        &self,
        config: (String, i32, i32, i32),
        app: AppHandle,
        on_event: Channel<CaptureEvent<'static>>,
    ) -> Result<(), CaptureError> {
        debug!("Démarrage de la capture sur l'interface {}...", config.0);

        on_event.send(CaptureEvent::Started {
            device: &config.0,
            buffer_size: config.1,
            chan_capacity: config.2,
            timeout: config.3,
        })?;

        let stop_flag = self.stop_flag.clone();

        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == config.0)
            .ok_or_else(|| CaptureError::InterfaceNotFound(config.0.clone()))?;

        info!("Interface trouvée : {}", device.name);

        let cap = setup::setup_capture(config.clone())?;
        let (tx, rx): (Sender<CaptureMessage>, Receiver<CaptureMessage>) =
            bounded(config.2 as usize);

        // 🔑 Utilisation du nouveau PacketBufferPool
        let arc_buffer_pool = Arc::new(PacketBufferPool::new(1000, 65536));

        // Démarrage des threads avec le nouveau buffer_pool
        spawn_processing_thread(
            rx,
            on_event.clone(),
            config.2,
            app.clone(),
            arc_buffer_pool.clone(),
        );
        spawn_capture_thread_with_pool(tx, on_event, cap, stop_flag, config.2, arc_buffer_pool);

        Ok(())
    }

    pub fn stop(&self, on_event: Channel<CaptureEvent<'static>>) -> Result<(), CaptureError> {
        info!("Arrêt de la capture demandé");
        self.stop_flag.store(true, Ordering::Relaxed);
        on_event.send(CaptureEvent::Stats {
            received: 0,
            dropped: 0,
            if_dropped: 0,
            processed: 0,
        })?;
        Ok(())
    }
}
