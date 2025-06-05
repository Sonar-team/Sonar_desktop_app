pub mod get_interfaces;
use std::sync::{Arc, Mutex};

pub use get_interfaces::get_interfaces_tab; // Réexporte la fonction

pub mod get_hostname_to_string;
pub use get_hostname_to_string::get_hostname_to_string;
use log::{error, info};
use tauri::State;

use crate::tauri_state::matrice::{PacketInfoEntry, SonarState};

pub mod export;
pub mod get_graph_data;
pub mod import;
pub mod net_capture;

#[tauri::command(async)]
pub fn get_matrice(state: State<'_, Arc<Mutex<SonarState>>>, header_value: Option<String>) -> Result<String, String> {
    //println!("  getmarice");
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;

    match locked_state.get_matrice_data() {
        Ok(data) => {
            // Parse the JSON string back into Vec<PacketInfoEntry>
            let entries: Vec<PacketInfoEntry> = match serde_json::from_str(&data) {
                Ok(entries) => entries,
                Err(e) => return Err(format!("Failed to parse JSON: {}", e)),
            };

            // Sort the vector based on the header_value if provided
            let mut sorted_entries = entries;
            if let Some(header) = header_value {
                sorted_entries.sort_by(|a, b| {
                    match header.as_str() {
                        "mac_address_source" => a.infos.mac_address_source.cmp(&b.infos.mac_address_source),
                        "mac_address_destination" => a.infos.mac_address_destination.cmp(&b.infos.mac_address_destination),
                        "l_3_protocol" => a.infos.l_3_protocol.cmp(&b.infos.l_3_protocol),
                        "ip_source" => match (a.infos.layer_3_infos.ip_source.as_ref(), b.infos.layer_3_infos.ip_source.as_ref()) {
                            (Some(a_ip), Some(b_ip)) => a_ip.cmp(b_ip),
                            (None, None) => std::cmp::Ordering::Equal,
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                        },
                        "ip_destination" => match (a.infos.layer_3_infos.ip_destination.as_ref(), b.infos.layer_3_infos.ip_destination.as_ref()) {
                            (Some(a_ip), Some(b_ip)) => a_ip.cmp(b_ip),
                            (None, None) => std::cmp::Ordering::Equal,
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                        },
                        _ => std::cmp::Ordering::Equal, // Default case if header value is not recognized
                    }
                });
            } // If no header_value is provided, the entries remain unsorted

            // Serialize back to JSON
            match serde_json::to_string(&sorted_entries) {
                Ok(sorted_json) => Ok(sorted_json),
                Err(e) => Err(format!("Failed to serialize sorted data: {}", e)),
            }
        }
        Err(e) => {
            error!("Error: {}", e);
            Err(e)
        }
    }
}

#[tauri::command(async)]
pub fn get_graph_state(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<String, String> {
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;

    locked_state.get_graph_data()
}

#[tauri::command(async)]
pub fn reset(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    info!("reset");
    let mut locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    locked_state.reset();
    Ok(())
}
