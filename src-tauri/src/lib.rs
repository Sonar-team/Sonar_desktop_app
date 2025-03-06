mod commandes;
use colored::Colorize;
use commandes::{
    export::{
        save_packets_to_csv, save_packets_to_excel, write_file, write_file_as_png, write_png_file,
    },
    get_graph_state, get_hostname_to_string, get_interfaces_tab, get_matrice,
    import::convert_from_pcap_list,
    reset,
    sniff::get_selected_interface,
    toggle_ipv6_filter, toggle_pause,
};
mod tauri_state;

use log::info;
use tauri_state::SonarState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        // liste des plugins
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // // .level(log::LevelFilter::Error)
                .clear_targets()
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .level(log::LevelFilter::Info)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("sonar".to_string()),
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
        // Actions au lancement
        .setup(|_app| {
            info!("{}", print_banner());
            get_os();
            Ok(())
        })
        // Commandes
        .invoke_handler(tauri::generate_handler![
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
            convert_from_pcap_list,
            write_png_file
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
