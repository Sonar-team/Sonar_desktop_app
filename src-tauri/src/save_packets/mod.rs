use crate::{sniff::capture_packet::layer_2_infos::PacketInfos, tauri_state::SonarState};
use csv::Writer;
use rust_xlsxwriter::*;
use serde::Serialize;
use tauri::State;
use thiserror::Error;

#[derive(Debug, Error, serde::Serialize)]
pub enum MyError {
    #[error("IO Error: {0}")]
    IoError(String),

    #[error("CSV Error: {0}")]
    CsvError(String),

    #[error("UTF-8 Conversion Error: {0}")]
    Utf8Error(String),

    #[error("XLSX Error: {0}")]
    XlsxError(String),
}

#[derive(Serialize)]
struct PacketInfosCsv {
    mac_address_source: String,
    mac_address_destination: String,
    interface: String,
    l_3_protocol: String, // Corresponding field for CSV serialization
    ip_source: Option<String>,
    ip_destination: Option<String>,
    l_4_protocol: Option<String>,
    port_source: Option<String>,
    port_destination: Option<String>,
    count: u32,
}


impl PacketInfosCsv {
    fn from_packet_infos(packet: &PacketInfos, count: u32) -> Self {
        PacketInfosCsv {
            mac_address_source: packet.mac_address_source.clone(),
            mac_address_destination: packet.mac_address_destination.clone(),
            interface: packet.interface.clone(),
            l_3_protocol: packet.l_3_protocol.clone(), // Populate from PacketInfos
            ip_source: packet.layer_3_infos.ip_source.clone(),
            ip_destination: packet.layer_3_infos.ip_destination.clone(),
            l_4_protocol: packet.layer_3_infos.l_4_protocol.clone(),
            port_source: packet.layer_3_infos.layer_4_infos.port_source.clone(),
            port_destination: packet.layer_3_infos.layer_4_infos.port_destination.clone(),
            count,
        }
    }
}


#[derive(Serialize)]
struct PacketData<'a> {
    packet: &'a PacketInfos,
    count: u32,
}

pub fn cmd_save_packets_to_csv(file_path: String, state: State<SonarState>) -> Result<(), MyError> {
    // Lock the state to access the data
    let data = state.0.lock().unwrap();

    // Create a CSV writer
    let mut wtr = Writer::from_path(file_path).map_err(|e| MyError::IoError(e.to_string()))?;

    // Serialize the entire vector to the CSV
    for (packet, count) in data.iter() {
        let packet_csv = PacketInfosCsv::from_packet_infos(packet, *count);
        wtr.serialize(packet_csv)
            .map_err(|e| MyError::CsvError(e.to_string()))?;
    }

    // Flush to ensure all data is written to the file
    wtr.flush().map_err(|e| MyError::IoError(e.to_string()))?;

    Ok(())
}

pub fn cmd_save_packets_to_excel(file_path: String, state: State<SonarState>) -> Result<(), MyError> {
    // Lock the state to access the data
    let data = state.0.lock().unwrap();

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
        "IP Destination", 
        "L4 Protocol", 
        "Source Port", 
        "Destination Port", 
        "Count"
    ];

    for (i, header) in headers.iter().enumerate() {
        sheet.write_string(0, i as u16, header.to_string())
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
    }

    // Serialize the entire vector to the Excel sheet
    for (i, (packet, count)) in data.iter().enumerate() {
        let packet_csv = PacketInfosCsv::from_packet_infos(packet, *count);
    
        // Écriture des champs dans chaque colonne
        sheet.write_string(i as u32 + 1, 0, &packet_csv.mac_address_source)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
        sheet.write_string(i as u32 + 1, 1, &packet_csv.mac_address_destination)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
        sheet.write_string(i as u32 + 1, 2, &packet_csv.interface)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
        sheet.write_string(i as u32 + 1, 3, &packet_csv.l_3_protocol)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;

        // Les champs optionnels doivent être gérés pour éviter les valeurs null
        if let Some(ip_src) = &packet_csv.ip_source {
            sheet.write_string(i as u32 + 1, 4, ip_src)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(ip_dst) = &packet_csv.ip_destination {
            sheet.write_string(i as u32 + 1, 5, ip_dst)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(l4_proto) = &packet_csv.l_4_protocol {
            sheet.write_string(i as u32 + 1, 6, l4_proto)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(port_src) = &packet_csv.port_source {
            sheet.write_string(i as u32 + 1, 7, port_src)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
        if let Some(port_dst) = &packet_csv.port_destination {
            sheet.write_string(i as u32 + 1, 8, port_dst)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }
    
        // Écriture du champ 'count'
        sheet.write_number(i as u32 + 1, 9, packet_csv.count as f64)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;
    }

    // Close the workbook
    workbook.save(file_path)
        .map_err(|e| MyError::XlsxError(e.to_string()))?;

    Ok(())
}