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
    state::capture::{capture_config::CaptureConfig, capture_handle::{
        messages::CaptureMessage, setup::{setup_capture, setup_filter}, threads::{
            capture::spawn_capture_thread_with_pool, packet_buffer::PacketBufferPool,
            processing::spawn_processing_thread,
        }
    }},
};

pub struct CaptureHandle {
    stop_flag: Arc<AtomicBool>
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
        config: CaptureConfig,
        app: AppHandle,
        on_event: Channel<CaptureEvent<'static>>,
        filter: Option<String>,
    ) -> Result<(), CaptureError> {
        debug!("Démarrage de la capture sur l'interface {}...", config.device_name);

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
        let arc_buffer_pool = Arc::new(
            PacketBufferPool::new(
                config.chan_capacity as usize + 2, 
                config.snaplen as usize));

        // Démarrage des threads avec le nouveau buffer_pool
        spawn_processing_thread(
            rx,
            on_event.clone(),
            config.chan_capacity,
            app.clone(),
            arc_buffer_pool.clone(),
        );
        spawn_capture_thread_with_pool(tx, 
            on_event, 
            cap, 
            stop_flag, 
            config.chan_capacity, 
            arc_buffer_pool);

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
