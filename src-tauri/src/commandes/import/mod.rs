use log::{error, info};
use packet_parser::PacketFlow;
use pcap::Capture;
use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError},
    events::CaptureEvent,
    state::{
        capture::capture_handle::messages::capture::PacketMinimal,
        flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};

fn count_packets_in_pcap(file_path: &str) -> Result<usize, CaptureStateError> {
    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;

    let mut count: usize = 0;
    while cap.next_packet().is_ok() {
        count += 1;
    }
    Ok(count)
}

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    pcap_paths: Vec<String>,
    on_event: Channel<CaptureEvent<'_>>,
) -> Result<(), CaptureStateError> {
    info!(
        "[convert_from_pcap_list] COMMAND CALLED avec pcap_paths = {:?}",
        pcap_paths
    );

    // started
    if let Err(e) = on_event.send(CaptureEvent::Started {
        device: "",
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
        snaplen: 65536,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    };

    let mut matrice_guard = matrice.lock().unwrap();
    let mut graph_guard = graph.lock().unwrap();
    matrice_guard.clear();
    graph_guard.clear();

    info!("[convert_from_pcap_list] Matrice & GraphData reset");

    for pcap_path in &pcap_paths {
        info!("[convert_from_pcap_list] Traitement de {}", pcap_path);
        handle_pcap_file(pcap_path, &mut matrice_guard, &mut graph_guard, &on_event)?;
    }

    info!("[convert_from_pcap_list] FIN traitement liste PCAP");

    // 🔥 snapshot complet envoyé sur le channel
    let snapshot: GraphData = graph_guard.get_all_graph_data(); // doit renvoyer un GraphData possédé

    if let Err(e) = on_event.send(CaptureEvent::GraphSnapshot {
        graph_data: &snapshot,
    }) {
        error!("Erreur lors de l'envoi de GraphSnapshot: {:?}", e);
    }

    Ok(())
}

fn handle_pcap_file(
    file_path: &str,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent<'_>>,
) -> Result<(), CaptureStateError> {
    let total = count_packets_in_pcap(file_path)?;
    info!("[handle_pcap_file] {} : {} paquets détectés", file_path, total);

    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;

    let mut packet_count: usize = 0;

    while let Ok(packet) = cap.next_packet() {
        packet_count += 1;

        if let Ok(flow) = PacketFlow::try_from(packet.data) {
            let packet_min = PacketMinimal {
                ts_sec: packet.header.ts.tv_sec,
                ts_usec: packet.header.ts.tv_usec,
                caplen: packet.header.caplen,
                len: packet.header.len,
                flow,
            };

            matrice.update_flow(&packet_min.to_owned_packet());
            let matrix_count = matrice.matrix.len();
            info!(
                "[handle_pcap_file] {} : paquet {}/{} ; lignes matrice = {}",
                file_path,
                packet_count,
                total,
                matrix_count
            );

            graph.add_packet_flow(&packet_min.flow.to_owned());

            // Stats périodiques (optionnel)
            if (packet_count.is_multiple_of(1000) || packet_count == total)
                && let Err(e) = on_event.send(CaptureEvent::Stats {
                    received: packet_count as u32,
                    dropped: 0,
                    if_dropped: 0,
                    processed: matrix_count as u32,
                })
            {
                error!("Erreur lors de l'envoi de Stats: {:?}", e);
            }

            // si tu veux envoyer des Packet individuellement (live)
            // if let Err(e) = on_event.send(CaptureEvent::Packet { packet: &packet_min }) { ... }
        }
    }

    if let Err(e) = on_event.send(CaptureEvent::Finished {
        file_name: file_path,
        packet_total_count: total,
        matrix_total_count: matrice.matrix.len(),
    }) {
        error!("Erreur lors de l'envoi de Finished: {:?}", e);
    };
    info!("[handle_pcap_file] Finised with {} paquets lu, {} lignes matrice", total, matrice.matrix.len());
    Ok(())
}
