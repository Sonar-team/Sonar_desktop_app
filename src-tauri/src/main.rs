// Prevents additional console window on Windows in release, DO NOT REMOVE!!
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use log::info;

use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use sonar_desktop_app::{
    cli::print_banner,
    get_hostname::hostname_to_s,
    get_interfaces::get_interfaces,
    sniff::{capture_packet::layer_2_infos::PacketInfos, scan_until_interrupt},
    tauri_state::{MyError, SonarState},
};
use tauri::{generate_handler, AppHandle, InvokeError, Manager, State};
// use tauri_plugin_log::LogTarget;

use resvg::tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree};

extern crate sonar_desktop_app;

fn main() {
    println!("{}", print_banner());

    tauri::Builder::default()
        .manage(SonarState::new())
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                std::process::exit(0);
            }
        })
        .setup(move |app| {
            let app_handle = app.handle();
            // Event listener for before-quit
            app_handle.listen_global("tauri://before-quit", move |_| {
                info!("Quit event received");
            });

            Ok(())
        })
        .invoke_handler(generate_handler![
            get_interfaces_tab,
            get_selected_interface,
            save_packets_to_csv,
            save_packets_to_excel,
            get_matrice,
            get_graph_state,
            write_file,
            write_file_as_png,
            toggle_ipv6_filter,
            toggle_pause,
            get_hostname_to_string,
            reset,
            convert_from_pcap_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(async, rename_all = "snake_case")]
fn get_interfaces_tab() -> Vec<String> {
    info!("demande des Interfaces réseaux");
    get_interfaces()
}

#[tauri::command(async, rename_all = "snake_case")]
fn get_hostname_to_string() -> String {
    info!("demande du hostname");
    hostname_to_s()
}

#[tauri::command(async, rename_all = "snake_case")]
fn get_selected_interface(app: AppHandle, interface_name: String) {
    info!("Interface sélectionée: {}", interface_name);
    scan_until_interrupt(app, &interface_name);
}

#[tauri::command(async, rename_all = "snake_case")]
fn save_packets_to_csv(
    file_path: String,
    state: State<'_, Arc<Mutex<SonarState>>>,
) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du CSV: {}", &file_path);
    let locked_state = state.lock().unwrap();
    locked_state.cmd_save_packets_to_csv(file_path)
}

#[tauri::command(async, rename_all = "snake_case")]
fn save_packets_to_excel(
    file_path: String,
    state: State<'_, Arc<Mutex<SonarState>>>,
) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du Excel: {}", &file_path);
    let locked_state = state.lock().unwrap();

    locked_state.cmd_save_packets_to_excel(file_path)
}

#[tauri::command(async)]
fn get_matrice(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<String, String> {
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
            println!("Error: {}", e); // Utilisez log::error pour les erreurs
            Err(e)
        }
    }
}

#[tauri::command(async)]
fn get_graph_state(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<String, String> {
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;

    locked_state.get_graph_data()
}

#[tauri::command(async)]
fn write_file(path: String, contents: String) -> Result<(), String> {
    info!("Chemin d'enregistrement du SVG: {}", &path);
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn write_file_as_png(path: String, contents: String) -> Result<(), String> {
    // Parse the SVG contents
    let opt = Options::default();
    let rtree = Tree::from_str(&contents, &opt).map_err(|e| e.to_string())?;

    // Create a pixmap with the dimensions of the SVG
    let pixmap_size = rtree.size();
    let mut pixmap = Pixmap::new(pixmap_size.width() as u32, pixmap_size.height() as u32)
        .ok_or("Failed to create pixmap")?;

    // Render the SVG onto the pixmap
    resvg::render(&rtree, Transform::identity(), &mut pixmap.as_mut());

    // Save the rendered image as a PNG file
    pixmap.save_png(&path).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command(async)]
fn toggle_ipv6_filter(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;

    locked_state.toggle_filter_ipv6();
    info!("etat du filtre {:?}", locked_state.filter_ipv6);
    Ok(())
}

#[tauri::command(async)]
fn toggle_pause(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    locked_state.toggle_actif();
    println!("etat actif");
    info!("etat du filtre {:?}", locked_state.actif);
    Ok(())
}

#[tauri::command(async)]
fn reset(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let mut locked_state = state
        .lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    locked_state.reset();
    Ok(())
}

use thiserror::Error;

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

#[tauri::command(async)]
fn convert_from_pcap_list(
    state: State<'_, Arc<Mutex<SonarState>>>,
    pcaps: Vec<String>,
) -> Result<u32, PcapProcessingError> {
    println!("Liste des fichiers pcap : {:?}", pcaps);

    let mut total_count = 0;

    for file_path in pcaps {
        // Ajoute le nombre de paquets lus pour chaque fichier `.pcap`
        total_count += handle_pcap_file(&file_path, &state)?;
    }

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

        // print_packet_in_hex(&packet.data);
        if let Some(ethernet_packet) = EthernetPacket::new(&packet.data) {
            // Créez une instance de PacketInfos pour le paquet actuel
            let packet_info = PacketInfos::new(&file_path.to_string(), &ethernet_packet);

            // Mettre à jour l'état de SonarState avec ce paquet
            let mut sonar_state = state.lock().unwrap();
            sonar_state.update_matrice_with_packet(packet_info);
        }
    }

    Ok(packet_count) // Retourne le nombre de paquets lus pour ce fichier
}

// // Fonction pour afficher un paquet en hexadécimal
// fn print_packet_in_hex(data: &[u8]) {
//     for byte in data {
//         print!("{:02X} ", byte);
//     }
//     println!();
// }
