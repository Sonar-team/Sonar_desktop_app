use crossbeam::channel::Sender;
use log::{debug, error};
use pcap::{Active, Capture};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tauri::ipc::Channel;

use crate::{
    events::CaptureEvent,
    state::capture::capture_handle::{
        messages::CaptureMessage, threads::packet_buffer::PacketBufferPool,
    },
};

pub fn spawn_capture_thread_with_pool(
    tx: Sender<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    mut cap: Capture<Active>,
    stop_flag: Arc<AtomicBool>,
    channel_capacity: i32,
    buffer_pool: Arc<PacketBufferPool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de capture avec pool");

        while !stop_flag.load(Ordering::Relaxed) {
            if let Ok(stats) = cap.stats()
                && let Err(e) = tx.try_send(CaptureMessage::Stats(stats))
            {
                error!("Erreur try_send stats: {}", e);
            }

            match cap.next_packet() {
                Ok(packet) => {
                    if let Some(mut buffer) = buffer_pool.get() {
                        // On copie les octets DANS UN SCOPE LIMITE 
                        buffer.write_from_parts(packet.header, packet.data);

                        match tx.try_send(CaptureMessage::Packet(buffer)) {
                            Ok(()) => {
                                // Succès : le processing thread RENDRA le buffer au pool.
                            }
                            Err(err) => {
                                error!("Erreur try_send paquet: {}", err);
                                if let Err(e) =
                                    on_event.send(CaptureEvent::ChannelCapacityPayload {
                                        channel_size: channel_capacity as usize,
                                        current_size: tx.len(),
                                        backpressure: true,
                                    })
                                {
                                    error!("Erreur send channel capacity payload: {}", e);
                                }
                                // Échec d'envoi => on remet IMMÉDIATEMENT le buffer au pool
                                if let CaptureMessage::Packet(buffer) = err.into_inner() {
                                    buffer_pool.put(buffer);
                                }
                            }
                        }
                    } else {
                        error!("Pas de buffer dispo");
                    }
                }
                Err(pcap::Error::PcapError(e)) if e.contains("Packets are not available") => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(pcap::Error::TimeoutExpired) => {
                    println!("TimeoutExpired");
                    continue;
                }
                Err(e) => {
                    error!("Erreur capture : {:?}", e);
                    break;
                }
            }
        }
        debug!("Thread de capture terminé.");
    });
}