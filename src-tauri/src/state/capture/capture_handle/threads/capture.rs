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
            if let Ok(stats) = cap.stats() {
                let _ = tx.try_send(CaptureMessage::Stats(stats));
            }

            match cap.next_packet() {
                Ok(packet) => {
                    if let Some(buffer_arc) = buffer_pool.get() {
                        // On copie les octets DANS UN SCOPE LIMITE pour drop le guard avant tout move
                        let mut buffer_too_small = false;

                        {
                            if let Ok(mut buffer) = buffer_arc.lock() {
                                let size = packet.header.caplen as usize;

                                if size <= buffer.data.len() {
                                    buffer.header = *packet.header;
                                    buffer.data[..size].copy_from_slice(&packet.data[..size]);
                                } else {
                                    error!("Buffer trop petit ({size} > {})", buffer.data.len());
                                    buffer_too_small = true;
                                }
                            } else {
                                // impossible d'obtenir le lock => on remettra au pool
                                buffer_too_small = true;
                            }
                        } // <-- le MutexGuard est LIBÉRÉ ici

                        if buffer_too_small {
                            // On restitue car on ne peut pas envoyer ce buffer
                            buffer_pool.put(buffer_arc);
                        } else {
                            // On tente l'envoi : on clone pour transférer AU CANAL,
                            // et on garde notre Arc local pour éventuellement le remettre au pool si échec.
                            match tx.try_send(CaptureMessage::Packet(buffer_arc.clone())) {
                                Ok(()) => {
                                    // Succès : le processing thread RENDRA le buffer au pool.
                                }
                                Err(err) => {
                                    error!("Erreur try_send paquet: {}", err);
                                    let _ = on_event.send(CaptureEvent::ChannelCapacityPayload {
                                        channel_size: channel_capacity as usize,
                                        current_size: tx.len(),
                                        backpressure: true,
                                    });
                                    // Échec d'envoi => on remet IMMÉDIATEMENT le buffer au pool
                                    buffer_pool.put(buffer_arc);
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

                Err(e) => {
                    error!("Erreur capture : {:?}", e);
                    break;
                }
            }
        }

        debug!("Thread de capture terminé.");
    });
}

