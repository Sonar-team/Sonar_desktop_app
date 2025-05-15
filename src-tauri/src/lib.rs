mod commandes;
use std::sync::{Arc, Mutex};

use colored::Colorize;
use commandes::{
    export::{
        logs::export_logs, save_packets_to_csv, save_packets_to_excel, write_file, write_file_as_png, write_png_file
    },
    get_graph_state, get_hostname_to_string, get_interfaces_tab, get_matrice,
    import::convert_from_pcap_list,
    net_capture::{config_capture, get_config_capture, start_capture, stop_capture},
    reset,
};
mod errors;
mod tauri_state;
mod utils;
use log::info;
use tauri_state::{capture::CaptureState, matrice::SonarState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        // liste des plugins
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll) // Empêche la suppression des logs
                .max_file_size(500_000) // Définit une taille maximale de fichier
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .level(log::LevelFilter::Info)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("SSF_sonar".to_string()),
                    },
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // State
        .manage(SonarState::new())
        .manage(Arc::new(Mutex::new(CaptureState::new())))
        // Actions au lancement
        .setup(|_app| {
            info!("{}", print_banner());
            get_os();
            Ok(())
        })
        // Commandes
        .invoke_handler(tauri::generate_handler![
            get_interfaces_tab,
            start_capture,
            stop_capture,
            config_capture,
            get_config_capture,
            save_packets_to_csv,
            save_packets_to_excel,
            get_matrice,
            get_graph_state,
            write_file,
            write_file_as_png,
            get_hostname_to_string,
            reset,
            convert_from_pcap_list,
            write_png_file,
            export_logs
        ])
        // Exécuter l'application
        .run(tauri::generate_context!())
}

fn get_os() {
    let platform = tauri_plugin_os::platform();
    info!("Platform: {}", platform);
}

fn print_banner() -> String {
    // ASCII art banner
    let banner = r"
    _________                           
   /   _____/ ____   ____ _____ _______ 
   \_____  \ /  _ \ /    \\__  \\_  __ \
   /        (  <_> )   |  \/ __ \|  | \/
  /_______  /\____/|___|  (____  /__|   
          \/            \/     \/          
   ";

    // La bannière est colorée en vert avant d'être retournée.
    banner.green().to_string()
}
