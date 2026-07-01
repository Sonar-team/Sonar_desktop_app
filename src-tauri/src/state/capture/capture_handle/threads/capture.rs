use crossbeam::channel::Sender;
use log::{debug, error};
use pcap::{Active, Capture};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tauri::ipc::Channel;

use crate::{
    events::CaptureEvent,
    state::capture::capture_handle::{
        messages::CaptureMessage, threads::packet_buffer::PacketBufferPool,
    },
};

const STATS_POLL_INTERVAL_MS: u64 = 250;

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
        let stats_poll_interval = Duration::from_millis(STATS_POLL_INTERVAL_MS);
        let mut last_stats_poll = Instant::now()
            .checked_sub(stats_poll_interval)
            .unwrap_or_else(Instant::now);

        while !stop_flag.load(Ordering::Relaxed) {
            if last_stats_poll.elapsed() >= stats_poll_interval {
                last_stats_poll = Instant::now();
                if let Ok(stats) = cap.stats()
                    && let Err(e) = tx.try_send(CaptureMessage::Stats(stats))
                {
                    error!("Erreur try_send stats: {}", e);
                }
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
                Err(pcap::Error::TimeoutExpired) => continue,
                Err(e) => {
                    error!("Erreur capture : {:?}", e);
                    break;
                }
            }
        }
        debug!("Thread de capture terminé.");
    });
}
