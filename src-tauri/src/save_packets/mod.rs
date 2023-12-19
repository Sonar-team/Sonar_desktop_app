use std::fs::File;
use crate::{tauri_state::SonarState, capture_packet::layer_2_infos::PacketInfos};
use csv::Writer;
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
}

#[derive(Serialize)]
struct PacketInfosCsv {
    mac_address_source: String,
    mac_address_destination: String,
    interface: String,
    ip_source: Option<String>,
    ip_destination: Option<String>,
    l_4_protocol: Option<String>,
    port_source: Option<String>, // Assuming port is of type u16
    port_destination: Option<String>, // Assuming port is of type u16
    count: u32,
}

// Assuming you have a function to convert PacketInfos to PacketInfosCsv
impl PacketInfosCsv {
    fn from_packet_infos(packet: &PacketInfos, count: u32) -> Self {
        PacketInfosCsv {
            mac_address_source: packet.mac_address_source.clone(),
            mac_address_destination: packet.mac_address_destination.clone(),
            interface: packet.interface.clone(),
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
    let mut wtr = Writer::from_path(file_path)
        .map_err(|e| MyError::IoError(e.to_string()))?;

    // Serialize the entire vector to the CSV
    for (packet, count) in data.iter() {
        let packet_csv = PacketInfosCsv::from_packet_infos(packet, *count);
        wtr.serialize(packet_csv)
            .map_err(|e| MyError::CsvError(e.to_string()))?;
    }

    // Flush to ensure all data is written to the file
    wtr.flush()
        .map_err(|e| MyError::IoError(e.to_string()))?;

    Ok(())
}