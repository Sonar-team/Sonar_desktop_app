use crossbeam::channel::Receiver;
use std::{sync::Arc, thread, time::{Duration, Instant}};
use tauri::AppHandle;
use log::{debug, error};

use packet_parser::PacketFlow;
use crate::{
    events::emit_event,
    tauri_state::capture::capture_handle::{
        messages::{
            capture::PacketMinimal, 
            channel::ChannelCapacityPayload, 
            stats::{
                StatTriple, 
                StatsPayload}, 
            CaptureMessage
        },
        threads::packet_buffer::PacketBufferPool
    },
};

pub fn spawn_processing_thread(
    rx: Receiver<CaptureMessage>,
    channel_capacity: i32,
    app: AppHandle,
    buffer_pool: Arc<PacketBufferPool>,
) {
    thread::spawn(move || {
        debug!("Démarrage du thread de traitement");

        let mut processed = 0;
        let mut last_stats = StatTriple::default();
        let mut last_channel = ChannelCapacityPayload::default();
        let mut last_update = Instant::now();
        let mut last_len = 0usize;
        static TEMPO: Duration = Duration::from_millis(40);
        let mut last_emit_frame = Instant::now();

        loop {
            let start_total = Instant::now();  // ⏱️ Chrono global

            match rx.recv() {
                Ok(CaptureMessage::Packet(pkt_arc)) => {
                    let start_lock = Instant::now();

                    if let Ok(buffer) = pkt_arc.lock() {
                        let elapsed_lock = start_lock.elapsed();
                        debug!("⏱️ Lock buffer : {} µs", elapsed_lock.as_micros());

                        let start_parse = Instant::now();
                        let flow = PacketFlow::try_from(buffer.data.as_ref()).ok();
                        // debug!("⏱️ Parse PacketFlow : {:?}", flow);
                        let elapsed_parse = start_parse.elapsed();
                        debug!("⏱️ Parse PacketFlow : {} µs", elapsed_parse.as_micros());

                        let start_build = Instant::now();
                        let record = PacketMinimal {
                            ts_sec: buffer.header.ts.tv_sec,
                            ts_usec: buffer.header.ts.tv_usec as i32,
                            caplen: buffer.header.caplen,
                            len: buffer.header.len,
                            flow: flow,
                        };
                        let elapsed_build = start_build.elapsed();
                        debug!("⏱️ Build PacketMinimal : {} µs", elapsed_build.as_micros());

                        processed += 1;

                        if last_emit_frame.elapsed() >= TEMPO {
                            let start_emit = Instant::now();
                            last_emit_frame = Instant::now();

                            emit_event(&app, "packet", &record);

                            let elapsed_emit = start_emit.elapsed();
                            debug!("⏱️ Emit parsed packet vers frontend : {} µs", elapsed_emit.as_micros());
                        }
                    }

                    buffer_pool.put(pkt_arc);

                    let elapsed_total = start_total.elapsed();
                    debug!("⏱️ Total traitement packet : {} µs", elapsed_total.as_micros());
                }

                Ok(CaptureMessage::Stats(stats)) => {
                    if let Err(e) = StatsPayload::from_stat_and_send(
                        &mut last_stats,
                        stats,
                        processed,
                        &app,
                    ) {
                        error!("[TAURI] Erreur stats : {}", e);
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

                let start_send = Instant::now();
                if let Err(e) = ChannelCapacityPayload::send_if_changed(
                    &mut last_channel,
                    current_len,
                    channel_capacity as usize,
                    &app,
                ) {
                    error!("[TAURI] Erreur émission canal : {}", e);
                }
                let elapsed_send = start_send.elapsed();
                debug!("⏱️ Temps de send les stats vers frontend : {} µs", elapsed_send.as_micros());
            }
        }
    });
}
