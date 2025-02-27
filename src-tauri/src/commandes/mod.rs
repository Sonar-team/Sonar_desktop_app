pub mod get_interfaces;
use std::sync::{Arc, Mutex};

pub use get_interfaces::get_interfaces_tab; // Réexporte la fonction

pub mod get_hostname_to_string;
pub use get_hostname_to_string::get_hostname_to_string;
use log::{error, info};
use tauri::State;

use crate::tauri_state::SonarState;

pub mod sniff;

pub mod get_graph_data;

pub mod export;
pub mod import;

#[tauri::command(async)]
pub fn get_matrice(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<String, String> {
    //println!("  getmarice");
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;

    match locked_state.get_matrice_data() {
        Ok(data) => {
            //println!("Data: {}", data); // Utilisez log::info si vous avez configuré un logger
            Ok(data)
        }
        Err(e) => {
            error!("Error: {}", e); // Utilisez log::error pour les erreurs
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
pub fn toggle_ipv6_filter(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;

    locked_state.toggle_filter_ipv6();
    info!("etat du filtre {:?}", locked_state.filter_ipv6);
    Ok(())
}

#[tauri::command(async)]
pub fn toggle_pause(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    locked_state.toggle_actif();
    println!("etat actif");
    info!("etat du filtre {:?}", locked_state.actif);
    Ok(())
}

#[tauri::command(async)]
pub fn reset(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let mut locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    locked_state.reset();
    Ok(())
}
