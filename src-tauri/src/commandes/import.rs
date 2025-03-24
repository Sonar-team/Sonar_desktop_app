use crate::tauri_state::SonarState;
use log::info;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use std::sync::{Arc, Mutex};
use tauri::{ipc::InvokeError, State};

#[tauri::command(async)]
pub fn convert_from_pcap_list(
    state: State<'_, Arc<Mutex<SonarState>>>,
    pcaps: Vec<String>,
) -> Result<u32, PcapProcessingError> {
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
    state: &State<'_, Arc<Mutex<SonarState>>>,
) -> Result<u32, PcapProcessingError> {
    let mut cap = Capture::from_file(file_path)
        .map_err(|e| PcapProcessingError::OpenFileError(file_path.to_string(), e.to_string()))?;

    let mut packet_count = 0;

    // Itérer sur les paquets, les afficher en hexadécimal et mettre à jour la matrice
    while let Ok(packet) = cap.next_packet() {
        packet_count += 1; // Incrémente le compteur pour chaque paquet
        println!("Paquet {}:", packet_count);
        // print_packet_in_hex(&packet.data);
        if let Some(ethernet_packet) = EthernetPacket::new(&packet.data) {
            // Créez une instance de PacketInfos pour le paquet actuel
            let packet_info = PacketInfos::new(&file_path.to_string(), &ethernet_packet);

            // Mettre à jour l'état de SonarState avec ce paquet
            let mut sonar_state = state.lock().unwrap();
            sonar_state.update_matrice_with_packet(packet_info);
            println!("Matrice size: {}",sonar_state.matrice.len());
        }
    }

    Ok(packet_count) // Retourne le nombre de paquets lus pour ce fichier
}

use thiserror::Error;

use super::sniff::capture_packet::layer_2_infos::PacketInfos;

#[derive(Error, Debug)]
pub enum PcapProcessingError {
    #[error("Failed to open pcap file {0}: {1}")]
    OpenFileError(String, String),
}

// Implémentation de `Into<InvokeError>` pour que `PcapProcessingError` soit compatible avec tauri::command
impl From<PcapProcessingError> for InvokeError {
    fn from(error: PcapProcessingError) -> Self {
        InvokeError::from(error.to_string())
    }
}
