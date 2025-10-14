use std::sync::{Arc, Mutex};

use log::info;
use packet_parser::PacketFlow;
use pcap::Capture;
use tauri::State;

use crate::{
    errors::{import::PcapImportError, CaptureStateError},
    state::{capture::capture_handle::messages::capture::PacketMinimal, flow_matrix::FlowMatrix},
};

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    state: State<'_, Arc<Mutex<FlowMatrix>>>,
    pcaps: Vec<String>,
) -> Result<u32, CaptureStateError> {
    info!("Liste des fichiers pcap : {:?}", pcaps);

    let mut total_count = 0;

    for file_path in pcaps {
        // Ajoute le nombre de paquets lus pour chaque fichier `.pcap`
        total_count += handle_pcap_file(&file_path, &state)?;
    }
    println!("Nombre total de paquets lus: {}", total_count);
    Ok(total_count)
}

fn handle_pcap_file(
    file_path: &str,
    state: &State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<u32, CaptureStateError> {
    let mut cap = Capture::from_file(file_path).map_err(|e| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            file_path.to_string(),
            e.to_string(),
        ))
    })?;

    let mut packet_count = 0;

    while let Ok(packet) = cap.next_packet() {
        packet_count += 1; // Incrémente le compteur pour chaque paquet
        if let Ok(flow) = PacketFlow::try_from(packet.data.as_ref()) {
            let record = PacketMinimal {
                ts_sec: packet.header.ts.tv_sec,
                ts_usec: packet.header.ts.tv_usec,
                caplen: packet.header.caplen,
                len: packet.header.len,
                flow,
            };
            state.lock().unwrap().update_flow(&record.to_owned_packet());
        }
    }
    Ok(packet_count)
}
