use crossbeam::channel::Receiver;
use log::{debug, error};
use std::{
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
                capture::PacketMinimal,
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

pub fn spawn_processing_thread(
    rx: Receiver<CaptureMessage>,
    on_event: Channel<CaptureEvent<'static>>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        let mut processed = 0;
        let mut last_channel = ChannelCapacityPayload::default();
        let mut last_update = Instant::now();
        let mut last_len = 0usize;
        static TEMPO: Duration = Duration::from_millis(40);
        let mut stats = StatTriple::default();

        loop {
            match rx.recv() {
                Ok(CaptureMessage::Packet(pkt)) => {
                    let flow = match PacketFlow::try_from(pkt.data.as_ref()) {
                        Ok(flow) => flow,
                        Err(e) => {
                            error!("Failed to parse PacketFlow: {}", e);
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

                    // envoi des packets lue en temps réel
                    if let Err(e) = on_event.send(CaptureEvent::Packet { packet: &packet }) {
                        error!("[TAURI] Erreur envoi Packet: {}", e);
                        break; // évite spammer d’erreurs si le canal est cassé
                    }

                    // ajout les paquets à la matrice de flux
                    let record_owned = packet.to_owned_packet();
                    let flow_matrix = app.state::<Arc<Mutex<FlowMatrix>>>();
                    if let Ok(mut locked_state) = flow_matrix.lock() {
                        locked_state.update_flow(&record_owned);
                        processed = locked_state.matrix.len() as u32;
                    };

                    let graph = app.state::<Arc<Mutex<GraphData>>>();
                    if let Ok(mut g) = graph.lock() {
                        // record_owned.flow est un PacketFlowOwned
                        let updates = g.add_packet_flow(&record_owned.flow);
                        // Envoi 1 par 1 (simple)
                        if !updates.is_empty() {
                            for update in updates {
                                if let Err(e) =
                                    on_event.send(CaptureEvent::Graph { update: &update })
                                {
                                    error!("[TAURI] Erreur envoi GraphUpdate: {}", e);
                                    break; // évite spammer d’erreurs si le canal est cassé
                                }
                            }
                        }
                    };

                    // Rendre le buffer au pool après traitement
                    buffer_pool.put(pkt);
                }

                Ok(CaptureMessage::Stats(new_stats)) => {
                    // new_stats: pcap::Stat
                    if let Err(e) = StatsPayload::maybe_send(
                        &mut stats, // <= ta variable déjà déclarée
                        new_stats, processed,
                        &on_event, // <= passe une référence, pas un move
                    ) {
                        error!("[TAURI] Erreur envoi Stats: {}", e);
                        // on NE fait pas `break` ici, on continue la boucle
                    }
                }
                Err(err) => {
                    error!("Erreur réception canal : {}", err);
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
