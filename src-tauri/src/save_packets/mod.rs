use std::sync::Mutex;

use crate::{
    sniff::capture_packet::layer_2_infos::{layer_3_infos::ip_type::IpType, PacketInfos},
    tauri_state::SonarState,
};
use csv::Writer;
use rust_xlsxwriter::*;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use thiserror::Error;

/// Enum représentant les différentes erreurs pouvant survenir lors de l'écriture de paquets vers un fichier CSV ou Excel.
#[derive(Debug, Error, serde::Serialize)]
pub enum MyError {
    /// Erreur d'entrée/sortie avec un message explicatif.
    #[error("Erreur d'E/S : {0}")]
    IoError(String),

    /// Erreur lors de la manipulation de fichiers CSV avec un message explicatif.
    #[error("Erreur CSV : {0}")]
    CsvError(String),

    /// Erreur de conversion UTF-8 avec un message explicatif.
    #[error("Erreur de conversion UTF-8 : {0}")]
    Utf8Error(String),

    /// Erreur lors de la manipulation de fichiers Excel avec un message explicatif.
    #[error("Erreur Excel : {0}")]
    XlsxError(String),
}

/// Structure représentant les informations des paquets à sérialiser vers un fichier CSV.
#[derive(Serialize)]
struct PacketInfosFlaten {
    /// Adresse MAC source du paquet.
    mac_address_source: String,
    /// Adresse MAC destination du paquet.
    mac_address_destination: String,
    /// Interface du paquet.
    interface: String,
    /// Protocole de la couche 3 du paquet.
    l_3_protocol: String,
    /// Adresse IP source du paquet (optionnel).
    ip_source: Option<String>,
    ip_source_type: Option<IpType>,
    /// Adresse IP destination du paquet (optionnel).
    ip_destination: Option<String>,
    ip_destination_type: Option<IpType>,
    /// Protocole de la couche 4 du paquet (optionnel).
    l_4_protocol: Option<String>,
    /// Port source du paquet (optionnel).
    port_source: Option<String>,
    /// Port destination du paquet (optionnel).
    port_destination: Option<String>,
    /// Taille du paquet.
    l_7_protocol: Option<String>,
    packet_size: usize,
    /// Nombre de fois que ce paquet a été rencontré.
    count: u32,
}

impl PacketInfosFlaten {
    /// Convertit les informations du paquet en une structure `PacketInfosFlaten`.
    fn from_packet_infos(packet: &PacketInfos, count: u32) -> Self {
        PacketInfosFlaten {
            mac_address_source: packet.mac_address_source.clone(),
            mac_address_destination: packet.mac_address_destination.clone(),
            interface: packet.interface.clone(),
            l_3_protocol: packet.l_3_protocol.clone(), // Populate from PacketInfos
            ip_source: packet.layer_3_infos.ip_source.clone(),
            ip_source_type: packet.layer_3_infos.ip_source_type.clone(),
            ip_destination: packet.layer_3_infos.ip_destination.clone(),
            ip_destination_type: packet.layer_3_infos.ip_destination_type.clone(),
            l_4_protocol: packet.layer_3_infos.l_4_protocol.clone(),
            port_source: packet.layer_3_infos.layer_4_infos.port_source.clone(),
            port_destination: packet.layer_3_infos.layer_4_infos.port_destination.clone(),
            l_7_protocol: packet.layer_3_infos.layer_4_infos.l_7_protocol.clone(),
            packet_size: packet.packet_size,
            count,
        }
    }
}


/// Fonction pour enregistrer les paquets vers un fichier CSV.
///
/// # Arguments
///
/// * `file_path` - Chemin du fichier CSV.
/// * `state` - État contenant les données des paquets.
///
/// # Exemple
///
/// ```rust
/// cmd_save_packets_to_csv(String::from("paquets.csv"), state);
/// ```
pub fn cmd_save_packets_to_csv(file_path: String, app: AppHandle) -> Result<(), MyError> {
    // Lock the state to access the data
    let state = app.state::<Mutex<SonarState>>(); // Acquire a lock
    let state_guard = state.lock().unwrap();
    let data = state_guard.get_matrice();

    // Create a CSV writer
    let mut wtr = Writer::from_path(file_path).map_err(|e| MyError::IoError(e.to_string()))?;

    // Serialize the entire vector to the CSV
    for (packet, count) in data.iter() {
        let packet_csv = PacketInfosFlaten::from_packet_infos(packet, *count);
        wtr.serialize(packet_csv)
            .map_err(|e| MyError::CsvError(e.to_string()))?;
    }

    // Flush to ensure all data is written to the file
    wtr.flush().map_err(|e| MyError::IoError(e.to_string()))?;

    Ok(())
}

/// Fonction pour enregistrer les paquets vers un fichier Excel.
///
/// # Arguments
///
/// * `file_path` - Chemin du fichier Excel.
/// * `state` - État contenant les données des paquets.
///
/// # Exemple
///
/// ```rust
/// cmd_save_packets_to_excel(String::from("paquets.xlsx"), state);
/// ```
pub fn cmd_save_packets_to_excel(file_path: String, app: AppHandle) -> Result<(), MyError> {
    // Lock the state to access the data
    let state = app.state::<Mutex<SonarState>>(); // Acquire a lock
    let state_guard = state.lock().unwrap();
    let data = state_guard.get_matrice();

    // Create an Excel workbook
    let mut workbook = Workbook::new();

    // Add a worksheet
    let sheet = workbook.add_worksheet();

    // Write header
    let headers = [
        "MAC Source",
        "MAC Destination",
        "Interface",
        "L3 Protocol",
        "IP Source",
        "IP Source Type",
        "IP Destination",
        "IP Destination Type",
        "L4 Protocol",
        "Source Port",
        "Destination Port",
        "Taille des packets",
        "Count",
    ];

    for (i, header) in headers.iter().enumerate() {
        sheet
            .write_string(0, i as u16, header.to_string())
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
    }

    // Serialize the entire vector to the Excel sheet
    for (i, (packet, count)) in data.iter().enumerate() {
        let packet_csv = PacketInfosFlaten::from_packet_infos(packet, *count);

        // Écriture des champs dans chaque colonne
        sheet
            .write_string(i as u32 + 1, 0, &packet_csv.mac_address_source)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
        sheet
            .write_string(i as u32 + 1, 1, &packet_csv.mac_address_destination)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
        sheet
            .write_string(i as u32 + 1, 2, &packet_csv.interface)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
        sheet
            .write_string(i as u32 + 1, 3, &packet_csv.l_3_protocol)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;

        // Les champs optionnels doivent être gérés pour éviter les valeurs null
        if let Some(ip_src) = &packet_csv.ip_source {
            sheet
                .write_string(i as u32 + 1, 4, ip_src)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(ip_src_type) = &packet_csv.ip_source_type {
            sheet
                .write_string(i as u32 + 1, 5, ip_src_type.to_string())
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(ip_dst) = &packet_csv.ip_destination {
            sheet
                .write_string(i as u32 + 1, 6, ip_dst)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(ip_dst_type) = &packet_csv.ip_destination_type {
            sheet
                .write_string(i as u32 + 1, 7, ip_dst_type.to_string())
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(l4_proto) = &packet_csv.l_4_protocol {
            sheet
                .write_string(i as u32 + 1, 8, l4_proto)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(port_src) = &packet_csv.port_source {
            sheet
                .write_string(i as u32 + 1, 9, port_src)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(port_dst) = &packet_csv.port_destination {
            sheet
                .write_string(i as u32 + 1, 10, port_dst)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }

        // Écriture du champ 'size'
        sheet
            .write_number(i as u32 + 1, 11, packet_csv.packet_size as f64)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;

        // Écriture du champ 'count'
        sheet
            .write_number(i as u32 + 1, 12, packet_csv.count as f64)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
    }

    // Close the workbook
    workbook
        .save(file_path)
        .map_err(|e| MyError::XlsxError(e.to_string()))?;

    Ok(())
}
