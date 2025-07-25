
use capture_message::{CaptureMessage, ChannelCapacityPayload, Codec, PacketFlow, StatsPayload};
use crossbeam::channel::{bounded, Receiver, Sender};
use layer_2_infos::PacketInfos;
use log::{debug, error, info, warn};

use pcap::{Device, PacketCodec};
use pnet::packet::ethernet::EthernetPacket;
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    errors::capture_error::CaptureError,
    tauri_state::{capture::capture_handle::capture_message::format_timestamp, matrice::SonarState},
};

pub mod capture_message;
pub mod layer_2_infos;
pub mod setup;

pub struct CaptureHandle {
    stop_flag: Arc<AtomicBool>,
}

impl CaptureHandle {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self, config: (String, i32, i32), app: AppHandle) -> Result<(), CaptureError> {
        debug!(
            "Démarrage de la capture sur l'interface {} avec un buffer de {} octets et une taille de channel de {}",
            config.0, config.1, config.2
        );

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

        let app_processing = app.clone();
        let interface_name = config.0.clone();
        thread::spawn(move || {
            debug!("Démarrage du thread de traitement");

            let mut processed = 0;
            let mut last_stats = None;
            let seuil_alerte = (config.2 as f32 * 0.9).floor() as usize;
            let mut last_packets: VecDeque<PacketFlow> = VecDeque::with_capacity(5);
            let mut last_emit_frame = Instant::now();
            let mut last_emit_stats = Instant::now();
            let mut last_emit_channel = Instant::now();
            let mut last_channel_len = 0usize;

            static ONE_SECOND: Duration = Duration::from_secs(2);

            loop {
                match rx.recv() {
                    
                    Ok(msg) => match msg {
                        CaptureMessage::Packet(pkt) => {
                            processed += 1;
                            if last_packets.len() == 5 {
                                last_packets.pop_back();
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
                                last_packets.push_front(packet_info);

                                if last_emit_frame.elapsed() >= ONE_SECOND {
                                    last_emit_frame = Instant::now();
                                    let _ = app_processing.emit("frame", &last_packets);
                                    let _ = app_processing.emit("matrice_len", &locked_state.get_matrice_len());
                                }
                            };
                        }
                        CaptureMessage::Stats(stats) => {
                            let current = (stats.received, stats.dropped, stats.if_dropped);
                            let should_emit = last_stats != Some(current)
                                && last_emit_stats.elapsed() >= ONE_SECOND;

                            if should_emit {
                                last_stats = Some(current);
                                last_emit_stats = Instant::now();

                                let _ = app_processing.emit(
                                    "stats",
                                    StatsPayload {
                                        received: current.0,
                                        dropped: current.1,
                                        if_dropped: current.2,
                                        processed,
                                    },
                                );
                            }
                        }
                    },
                    Err(err) => {
                        error!("Erreur réception canal : {}", err);
                        break;
                    }
                }

                let current_len = rx.len();
                let backpressure = current_len >= seuil_alerte;

                if current_len != last_channel_len || last_emit_channel.elapsed() >= ONE_SECOND {
                    last_channel_len = current_len;
                    last_emit_channel = Instant::now();

                    let _ = app_processing.emit(
                        "channel",
                        ChannelCapacityPayload {
                            channel_size: config.2 as usize,
                            current_size: current_len,
                            backpressure,
                        },
                    );

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

        let app_capture = app.clone();
        let stop_flag_capture = stop_flag.clone();
        thread::spawn(move || {
            debug!("Démarrage du thread de capture");

            let mut cap = cap;
            let mut codec = Codec;
            let mut backoff = 1;

            while !stop_flag_capture.load(Ordering::Relaxed) {
                if let Ok(stats) = cap.stats() {
                    let _ = tx.try_send(CaptureMessage::Stats(stats));
                }

                match cap.next_packet() {
                    Ok(packet) => {
                        let owned = codec.decode(packet);
                        if let Err(_err) = tx.try_send(CaptureMessage::Packet(owned)) {
                            let _ = app_capture.emit(
                                "channel",
                                ChannelCapacityPayload {
                                    channel_size: config.2 as usize,
                                    current_size: tx.len(),
                                    backpressure: true,
                                },
                            );
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
        self.stop_flag.store(true, Ordering::Relaxed);
        let _ = app_processing.emit(
            "stats",
            StatsPayload {
                received: 0,
                dropped: 0,
                if_dropped: 0,
                processed: 0,
            },
        );
    }
}