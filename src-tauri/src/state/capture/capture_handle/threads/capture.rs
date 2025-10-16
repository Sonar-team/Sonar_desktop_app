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

        let mut backoff = 1;

        while !stop_flag.load(Ordering::Relaxed) {
            if let Ok(stats) = cap.stats() {
                let _ = tx.try_send(CaptureMessage::Stats(stats));
            }

            match cap.next_packet() {
                Ok(packet) => {
                    if let Some(buffer_arc) = buffer_pool.get() {
                        let mut buffer_too_small = false;

                        {
                            if let Ok(mut buffer) = buffer_arc.lock() {
                                let size = packet.header.caplen as usize;

                                if size <= buffer.data.len() {
                                    buffer.header = *packet.header;
                                    buffer.data[..size].copy_from_slice(&packet.data[..size]);

                                    if let Err(err) =
                                        tx.try_send(CaptureMessage::Packet(buffer_arc.clone()))
                                    {
                                        error!("Erreur try_send paquet: {}", err);
                                        let _ = on_event
                                            .send(CaptureEvent::ChannelCapacityPayload {
                                                channel_size: channel_capacity as usize,
                                                current_size: tx.len(),
                                                backpressure: true,
                                            })
                                            .unwrap();
                                    }
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
