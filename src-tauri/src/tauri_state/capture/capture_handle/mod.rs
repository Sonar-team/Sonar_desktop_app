use capture_message::{CaptureMessage, ChannelCapacityPayload, Codec, PacketFlow, StatsPayload};
use crossbeam::channel::{bounded, Receiver, Sender};
use layer_2_infos::PacketInfos;
use log::{debug, error, info, warn};

use pcap::{Device, PacketCodec};
use pnet::packet::ethernet::EthernetPacket;
use std::{
    collections::VecDeque, sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    }, thread, time::{Duration, Instant}
};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{errors::capture_error::CaptureError, tauri_state::{capture::capture_handle::capture_message::format_timestamp, matrice::SonarState}};
pub mod capture_message;
pub mod layer_2_infos;
pub mod setup;
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

    pub fn start(&self, config: (String, i32, i32), app: AppHandle) -> Result<(), CaptureError> {
        debug!("Démarrage de la capture sur l'interface {} avec un buffer de {} octets et une taille de channel de {}", config.0, config.1, config.2);

        let stop_flag = self.stop_flag.clone();

        let iface_name = config.0.clone();
        let device = Device::list()
            .map_err(CaptureError::DeviceListError)?
            .into_iter()
            .find(|d| d.name == iface_name)
            .ok_or_else(|| CaptureError::InterfaceNotFound(iface_name.clone()))?;

        info!("Interface trouvée : {}", device.name);

        let cap = setup::setup_capture(device, config.1)?;
        let (tx, rx): (Sender<CaptureMessage>, Receiver<CaptureMessage>) =
            bounded(config.2 as usize);

        // Thread de traitement
        let app_processing = app.clone();
        let interface_name = config.0.clone();
        thread::spawn(move || {
            debug!("Démarrage du thread de traitement");
            let mut processed = 0;
            let mut last_stats = None;
            let seuil_alerte = (config.2 as f32 * 0.9).floor() as usize;
            let mut last_update = Instant::now();
            let mut last_len = 0usize;
            let mut last_packets: VecDeque<PacketFlow> = VecDeque::with_capacity(5);

            loop {
                match rx.recv() {
                    Ok(msg) => match msg {
                        CaptureMessage::Packet(pkt) => {
                            processed += 1;

                            if last_packets.len() == 5 {
                                last_packets.pop_back(); // supprime le plus ancien
                            }

                            let packet = EthernetPacket::new(&pkt.data).unwrap();
                            
                            let packet_info = PacketFlow {
                                ts_sec: pkt.header.ts.tv_sec,
                                ts_usec: pkt.header.ts.tv_usec,
                                caplen: pkt.header.caplen,
                                len: pkt.header.len,
                                flow: PacketInfos::new(&interface_name, &packet),
                                formatted_time: format_timestamp(pkt.header.ts.tv_sec, pkt.header.ts.tv_usec),
                            };
                            let state: State<Arc<Mutex<SonarState>>> = app_processing.state::<Arc<Mutex<SonarState>>>();

                            if let Ok(mut locked_state) = state.lock() {
                                locked_state.update_matrice_with_packet(&packet_info.flow);
                                last_packets.push_front(packet_info); // ajoute le nouveau en tête
                                if let Err(e) = app_processing.emit("frame", &last_packets) {
                                    error!("[TAURI] Échec de l'émission 'frame' : {}", e);
                                }
                                if let Err(e) = app_processing.emit("matrice_len", &locked_state.get_matrice_len()) {
                                    error!("[TAURI] Échec de l'émission 'matrice_len' : {}", e);
                                }
                            } else {
                                error!("Échec du verrouillage du state SonarState");
                            };
                        }
                        CaptureMessage::Stats(stats) => {
                            let current = (stats.received, stats.dropped, stats.if_dropped);
                            if last_stats != Some(current) {
                                last_stats = Some(current);
                                if let Err(e) = app_processing.emit(
                                    "stats",
                                    StatsPayload {
                                        received: current.0,
                                        dropped: current.1,
                                        if_dropped: current.2,
                                        processed,
                                    },
                                ) {
                                    error!("[TAURI] Échec de l'émission 'stats' : {}", e);
                                }
                            }
                        }
                    },
                    Err(err) => {
                        error!("Erreur réception canal : {}", err);
                        break;
                    }
                }

                let current_len = rx.len();
                if last_len != current_len || last_update.elapsed() >= Duration::from_millis(50) {
                    last_update = Instant::now();
                    last_len = current_len;

                    let backpressure = current_len >= seuil_alerte;

                    if let Err(e) = app_processing.emit(
                        "channel",
                        ChannelCapacityPayload {
                            channel_size: config.2 as usize,
                            current_size: current_len,
                            backpressure,
                        },
                    ) {
                        error!("[TAURI] Échec de l'émission 'channel' : {}", e);
                    }

                    if backpressure {
                        warn!(
                            "[BACKPRESSURE] Canal rempli à {}/{} ({}%)",
                            current_len,
                            config.2 as usize,
                            (current_len * 100) / config.2 as usize
                        );
                    }
                }
            }
        });

        let app_cpature = app.clone();
        // ✅ Thread de capture ici
        let stop_flag_capture = stop_flag.clone();

        // let app_capture = app.clone();
        thread::spawn(move || {
            debug!("Démarrage du thread de capture");

            let mut cap = cap;
            let mut codec = Codec;
            let mut backoff = 1;

            while !stop_flag_capture.load(Ordering::Relaxed) {
                match cap.stats() {
                    Ok(stats) => {
                        let _ = tx.try_send(CaptureMessage::Stats(stats));
                    }
                    Err(e) => {
                        error!("[ERREUR] Impossible de récupérer les stats : {:?}", e);
                    }
                }

                match cap.next_packet() {
                    Ok(packet) => {
                        let owned = codec.decode(packet);
                        if let Err(err) = tx.try_send(CaptureMessage::Packet(owned)) {
                            error!("Erreur try_send paquet: {}", err);
                            if let Err(e) = app_cpature.emit(
                                "channel",
                                ChannelCapacityPayload {
                                    channel_size: config.2 as usize,
                                    current_size: tx.len(),
                                    backpressure: true,
                                },
                            ) {
                                error!("[TAURI] Échec de l'émission 'channel' : {}", e);
                            }
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
                        error!("Erreur de capture: {:?}", e);
                        break;
                    }
                }
            }
            debug!("Thread de capture terminé.");
        });

        Ok(())
    }

    pub fn stop(&self, app_processing: AppHandle) {
        info!("Arrêt de la capture demandé");
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Err(e) = app_processing.emit(
            "stats",
            StatsPayload {
                received: 0,
                dropped: 0,
                if_dropped: 0,
                processed: 0,
            },
        ) {
            error!("[TAURI] Échec de l'émission 'stats' : {}", e);
        }
    }
}
