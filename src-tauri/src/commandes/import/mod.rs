use log::{error, info};
use packet_parser::PacketFlow;
use pcap::Capture;
use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use crate::{
    errors::{CaptureStateError, import::PcapImportError},
    events::CaptureEvent, // (si tu as un module séparé, sinon garde l'enum ci-dessous)
    state::{
        capture::capture_handle::messages::capture::PacketMinimal, flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};

// Compte les paquets d'un PCAP (premier passage)
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
    pcaps: Vec<String>,
    on_event: Channel<CaptureEvent>,
) -> Result<(), CaptureStateError> {
    info!("Liste des fichiers pcap : {:?}", pcaps);

    let mut total_count: u32 = 0;
    let mut matrice_guard = matrice.lock().unwrap();
    let mut graph_guard = graph.lock().unwrap();

    for file_path in pcaps {
        total_count += handle_pcap_file(
            &file_path,
            &mut matrice_guard,
            &mut graph_guard,
            &on_event,
        )?;
        if let Err(e) = on_event.send(CaptureEvent::Finished {
            file_name: &file_path,
            packet_total_count: matrice_guard.matrix.len(),
        }) {
            error!("Erreur lors de l'envoi de Finished: {:?}", e);
        }
    }

    info!("Nombre total de paquets lus: {}", total_count);
    info!(
        "Nombre total de lignes cree: {}",
        &matrice_guard.matrix.len()
    );

    Ok(())
}

fn handle_pcap_file(
    file_path: &str,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
    on_event: &Channel<CaptureEvent>,
) -> Result<u32, CaptureStateError> {
    // 1) Compter d'abord
    let total = count_packets_in_pcap(file_path)?;
    if let Err(e) = on_event.send(CaptureEvent::Started {
        device: file_path,
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    };

    // 2) ROUVRIR le pcap pour le vrai traitement
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
            // On own le flow dans le record pour réutiliser la même instance partout
            let packet = PacketMinimal {
                ts_sec: packet.header.ts.tv_sec,
                ts_usec: packet.header.ts.tv_usec,
                caplen: packet.header.caplen,
                len: packet.header.len,
                flow,
            };

            let matrice_count = matrice.update_flow(&packet.to_owned_packet());
            graph.add_packet_flow(&packet.flow.to_owned());

            // (option) n’envoie pas trop souvent ; ici toutes les 1000 itérations
            if (packet_count.is_multiple_of(1000) || packet_count == total)
                && let Err(e) = on_event.send(CaptureEvent::Stats {
                    received: packet_count as u32,
                    dropped: 0,
                    if_dropped: 0,
                    processed: matrice_count as u32,
                })
            {
                error!("Erreur lors de l'envoi de Stats: {:?}", e);
            }
        }
    }

    if let Err(e) = on_event.send(CaptureEvent::Finished {
        file_name: file_path,
        packet_total_count: total,
    }) {
        error!("Erreur lors de l'envoi de Finished: {:?}", e);
    };
    Ok(packet_count as u32)
}
