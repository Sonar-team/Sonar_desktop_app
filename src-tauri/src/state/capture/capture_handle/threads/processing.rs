use crossbeam::channel::{Receiver, RecvTimeoutError};
use log::{debug, error, info};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, ipc::Channel};

use crate::{
    events::CaptureEvent,
    state::{
        capture::capture_handle::{
            messages::{
                CaptureMessage,
                capture::{PacketMinimal, PacketOwnedStats},
                channel::ChannelCapacityPayload,
                stats::{StatTriple, StatsPayload},
            },
            threads::packet_buffer::PacketBufferPool,
        },
        flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};
use packet_parser::PacketFlow;

// Flush le batch de paquets vers le frontend. Retourne false si le canal est cassé.
macro_rules! flush_batch {
    ($batch:expr, $on_event:expr, $last_flush:expr) => {{
        if $batch.is_empty() {
            true
        } else {
            let packets = std::mem::take(&mut $batch);
            $last_flush = Instant::now();
            match $on_event.send(CaptureEvent::PacketBatch { packets }) {
                Ok(_) => true,
                Err(e) => {
                    error!("[TAURI] Erreur envoi PacketBatch: {}", e);
                    false
                }
            }
        }
    }};
}

pub fn spawn_processing_thread(
    rx: Receiver<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
    stop_flag: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        let mut processed = 0;
        let mut last_channel = ChannelCapacityPayload::default();
        let mut last_update = Instant::now();
        let mut last_len = 0usize;
        static TEMPO: Duration = Duration::from_millis(40);
        let mut stats = StatTriple::default();
        let batch_max: usize = 64;
        let batch_interval = Duration::from_millis(50);
        let mut packet_batch: Vec<PacketOwnedStats> = Vec::with_capacity(batch_max);
        let mut last_batch_flush = Instant::now();

        loop {
            let timeout = batch_interval.saturating_sub(last_batch_flush.elapsed());
            match rx.recv_timeout(timeout.max(Duration::from_millis(1))) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        // Drain quietly after stop: no packet event and no matrix update.
                        packet_batch.clear();
                        buffer_pool.put(pkt);
                        continue;
                    }

                    let flow = match PacketFlow::try_from(pkt.as_ref()) {
                        Ok(flow) => flow,
                        Err(e) => {
                            error!("Failed to parse PacketFlow: {}", e);
                            buffer_pool.put(pkt);
                            continue;
                        }
                    };

                    let packet = PacketMinimal {
                        ts_sec: pkt.header.ts.tv_sec,
                        ts_usec: pkt.header.ts.tv_usec,
                        caplen: pkt.header.caplen,
                        len: pkt.header.len,
                        flow,
                    };

                    // ajout les paquets à la matrice de flux
                    let record_owned = packet.to_owned_packet();
                    let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>();
                    let (source_label, destination_label) =
                        if let Ok(locked_state) = flow_matrix.lock() {
                            let source_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.source_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();
                            let destination_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.destination_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();

                            (
                                locked_state
                                    .get_label(&record_owned.flow.data_link.source_mac, &source_ip),
                                locked_state.get_label(
                                    &record_owned.flow.data_link.destination_mac,
                                    &destination_ip,
                                ),
                            )
                        } else {
                            (None, None)
                        };
                    if let Ok(mut locked_state) = flow_matrix.lock() {
                        locked_state.update_flow(&record_owned);
                        processed = locked_state.matrix.len() as u32;
                    };

                    let graph = app.state::<Arc<Mutex<GraphData>>>();
                    if let Ok(mut g) = graph.lock() {
                        let updates =
                            g.add_packet_flow(&record_owned.flow, source_label, destination_label);
                        if !updates.is_empty() {
                            for update in updates {
                                if let Err(e) =
                                    on_event.send(CaptureEvent::Graph { update: &update })
                                {
                                    error!("[TAURI] Erreur envoi GraphUpdate: {}", e);
                                    break;
                                }
                            }
                        }
                    };

                    // Accumuler dans le batch
                    packet_batch.push(record_owned);

                    // Flush si le batch est plein ou si l'intervalle est écoulé
                    if packet_batch.len() >= batch_max
                        || last_batch_flush.elapsed() >= batch_interval
                    {
                        if !flush_batch!(packet_batch, on_event, last_batch_flush) {
                            buffer_pool.put(pkt);
                            break;
                        }
                    }

                    buffer_pool.put(pkt);
                }

                Ok(CaptureMessage::Stats(new_stats)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        continue;
                    }

                    if let Err(e) = StatsPayload::maybe_send(
                        &mut stats,
                        new_stats, processed,
                        &on_event,
                    ) {
                        error!("[TAURI] Erreur envoi Stats: {}", e);
                    }
                }

                Err(RecvTimeoutError::Timeout) => {
                    // Flush le batch restant après inactivité
                    if !flush_batch!(packet_batch, on_event, last_batch_flush) {
                        break;
                    }
                }

                Err(RecvTimeoutError::Disconnected) => {
                    error!("Erreur réception canal : canal déconnecté");
                    break;
                }
            }

            let current_len = rx.len();

            if last_len != current_len || last_update.elapsed() >= TEMPO {
                last_update = Instant::now();
                last_len = current_len;

                if let Err(e) = ChannelCapacityPayload::send_if_changed(
                    &mut last_channel,
                    current_len,
                    channel_capacity as usize,
                    &app,
                ) {
                    error!("[TAURI] Erreur émission canal : {}", e);
                }
            }
        }
    });
}

pub fn spawn_processing_thread_cli(
    rx: Receiver<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
    stop_flag: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        let mut processed = 0;
        let mut last_channel = ChannelCapacityPayload::default();
        let mut last_update = Instant::now();
        let mut last_len = 0usize;
        static TEMPO: Duration = Duration::from_millis(40);
        let mut stats = StatTriple::default();
        let batch_max: usize = 64;
        let batch_interval = Duration::from_millis(50);
        let mut packet_batch: Vec<PacketOwnedStats> = Vec::with_capacity(batch_max);
        let mut last_batch_flush = Instant::now();

        loop {
            let timeout = batch_interval.saturating_sub(last_batch_flush.elapsed());
            match rx.recv_timeout(timeout.max(Duration::from_millis(1))) {
                Ok(CaptureMessage::Packet(pkt)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        packet_batch.clear();
                        buffer_pool.put(pkt);
                        continue;
                    }

                    let flow = match PacketFlow::try_from(pkt.as_ref()) {
                        Ok(flow) => flow,
                        Err(e) => {
                            error!("Failed to parse PacketFlow: {}", e);
                            buffer_pool.put(pkt);
                            continue;
                        }
                    };

                    let packet = PacketMinimal {
                        ts_sec: pkt.header.ts.tv_sec,
                        ts_usec: pkt.header.ts.tv_usec,
                        caplen: pkt.header.caplen,
                        len: pkt.header.len,
                        flow,
                    };

                    let record_owned = packet.to_owned_packet();
                    let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>();
                    let (source_label, destination_label) =
                        if let Ok(locked_state) = flow_matrix.lock() {
                            let source_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.source_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();
                            let destination_ip = record_owned
                                .flow
                                .internet
                                .as_ref()
                                .and_then(|i| i.destination_ip)
                                .map(|ip| ip.to_string())
                                .unwrap_or_default();

                            (
                                locked_state
                                    .get_label(&record_owned.flow.data_link.source_mac, &source_ip),
                                locked_state.get_label(
                                    &record_owned.flow.data_link.destination_mac,
                                    &destination_ip,
                                ),
                            )
                        } else {
                            (None, None)
                        };
                    if let Ok(mut locked_state) = flow_matrix.lock() {
                        let (flow_stats, flow) = locked_state.update_flow_cli(&record_owned);
                        info!("flow_stats: {:?}, {:?}", flow_stats, flow);
                        processed = locked_state.matrix.len() as u32;
                    };

                    let graph = app.state::<Arc<Mutex<GraphData>>>();
                    if let Ok(mut g) = graph.lock() {
                        let updates =
                            g.add_packet_flow(&record_owned.flow, source_label, destination_label);
                        if !updates.is_empty() {
                            for update in updates {
                                if let Err(e) =
                                    on_event.send(CaptureEvent::Graph { update: &update })
                                {
                                    error!("[TAURI] Erreur envoi GraphUpdate: {}", e);
                                    break;
                                }
                            }
                        }
                    };

                    packet_batch.push(record_owned);

                    if packet_batch.len() >= batch_max
                        || last_batch_flush.elapsed() >= batch_interval
                    {
                        if !flush_batch!(packet_batch, on_event, last_batch_flush) {
                            buffer_pool.put(pkt);
                            break;
                        }
                    }

                    buffer_pool.put(pkt);
                }

                Ok(CaptureMessage::Stats(new_stats)) => {
                    if stop_flag.load(Ordering::Relaxed) {
                        continue;
                    }

                    if let Err(e) = StatsPayload::maybe_send(
                        &mut stats,
                        new_stats, processed,
                        &on_event,
                    ) {
                        error!("[TAURI] Erreur envoi Stats: {}", e);
                    }
                }

                Err(RecvTimeoutError::Timeout) => {
                    if !flush_batch!(packet_batch, on_event, last_batch_flush) {
                        break;
                    }
                }

                Err(RecvTimeoutError::Disconnected) => {
                    error!("Erreur réception canal : canal déconnecté");
                    break;
                }
            }

            let current_len = rx.len();

            if last_len != current_len || last_update.elapsed() >= TEMPO {
                last_update = Instant::now();
                last_len = current_len;

                if let Err(e) = ChannelCapacityPayload::send_if_changed(
                    &mut last_channel,
                    current_len,
                    channel_capacity as usize,
                    &app,
                ) {
                    error!("[TAURI] Erreur émission canal : {}", e);
                }
            }
        }
    });
}
