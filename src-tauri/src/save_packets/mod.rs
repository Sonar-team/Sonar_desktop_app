use std::fs::File;
use crate::tauri_state::SonarState;
use csv::Writer;
use tauri::State;

pub fn cmd_save_packets_to_csv(file_path: String, state: State<SonarState>) -> Result<String, String> {
    // Access the inner SonarState and then call lock on it
    let packets = state.inner().0.lock().map_err(|e| e.to_string())?;

    let file = File::create(&file_path).map_err(|e| e.to_string())?;
    let mut wtr = Writer::from_writer(file);

    for packet_info in packets.iter() {
        wtr.serialize(packet_info).map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    
    Ok(format!("Data saved to {}", file_path))
}