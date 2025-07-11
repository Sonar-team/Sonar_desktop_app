use crossbeam::channel::Sender;
use log::{debug, error};
use pcap::{Active, Capture};
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};

use crate::tauri_state::capture::capture_handle::{messages::{channel::ChannelCapacityPayload, CaptureMessage}, threads::packet_buffer::PacketBufferPool};

pub fn spawn_capture_thread_with_pool(
    tx: Sender<CaptureMessage>,
    mut cap: Capture<Active>,
    stop_flag: Arc<AtomicBool>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de capture avec pool");

        let mut backoff = 1;

        while !stop_flag.load(Ordering::Relaxed) {
            let start_total = Instant::now();  // ⏱️ Chrono total

            if let Ok(stats) = cap.stats() {
                let _ = tx.try_send(CaptureMessage::Stats(stats));
            }

            match cap.next_packet() {
                Ok(packet) => {
                    if let Some(buffer_arc) = buffer_pool.get() {
                        let mut buffer_too_small = false;
                        let lock_start = Instant::now();  // ⏱️ Chrono lock

                        {
                            if let Ok(mut buffer) = buffer_arc.lock() {
                                let lock_elapsed = lock_start.elapsed();
                                debug!("⏱️ Lock buffer : {} µs", lock_elapsed.as_micros());

                                let size = packet.header.caplen as usize;
                                let copy_start = Instant::now();  // ⏱️ Chrono copie


                                if size <= buffer.data.len() {
                                    buffer.header = *packet.header;
                                    buffer.data[..size].copy_from_slice(&packet.data[..size]);
                    
                                    let copy_elapsed = copy_start.elapsed();
                                    debug!("⏱️ Copie packet -> buffer : {} µs", copy_elapsed.as_micros());

                                    let send_start = Instant::now();  // ⏱️ Chrono send

                                    if let Err(err) = tx.try_send(CaptureMessage::Packet(buffer_arc.clone())) {
                                        error!("Erreur try_send paquet: {}", err);
                                        let _ = app.emit(
                                            "channel",
                                            ChannelCapacityPayload {
                                                channel_size: channel_capacity as usize,
                                                current_size: tx.len(),
                                                backpressure: true,
                                            },
                                        );
                                    }

                                    let send_elapsed = send_start.elapsed();
                                    debug!("⏱️ Send paquet : {} µs", send_elapsed.as_micros());
                                } else {
                                    error!("Buffer trop petit ({size} > {})", buffer.data.len());
                                    buffer_too_small = true;
                                }
                            } else {
                                buffer_too_small = true;
                            }
                        } // <-- lock dropped here
                    
                        if buffer_too_small {
                            buffer_pool.put(buffer_arc);
                        }
                    } else {
                        error!("Pas de buffer dispo");
                    }

                    let total_elapsed = start_total.elapsed();
                    debug!("⏱️ Total capture : {} µs", total_elapsed.as_micros());

                    backoff = 1;
                }

                Err(pcap::Error::PcapError(e)) if e.contains("Packets are not available") => {
                    thread::sleep(Duration::from_millis(1));
                }

                Err(pcap::Error::TimeoutExpired) => {
                    thread::sleep(Duration::from_micros(backoff));
                    backoff = (backoff * 2).min(10_000);
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
