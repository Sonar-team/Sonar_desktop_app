use commandes::{
    net_capture::{config_capture, get_config_capture, start_capture, stop_capture},
    net_interface::get_devices_list,
};
use log::info;

use std::sync::{Arc, Mutex};
use tauri::menu::MenuBuilder;

use crate::{
    commandes::{
        export::{csv::export_csv, logs::export_logs}, flow_matrix::add_label, import::convert_from_pcap_list, net_capture::{reset_capture, set_filter}
    },
    setup::{get_os, print_banner, system_info::start_cpu_monitor},
    state::{capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData},
};

mod commandes;
mod errors;
mod events;
mod setup;
mod state;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        // Plugins
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // Etats partager dans l'application
        .manage(Arc::new(Mutex::new(CaptureState::new())))
        .manage(Arc::new(Mutex::new(FlowMatrix::new())))
        .manage(Arc::new(Mutex::new(GraphData::new())))
        // Menu
        .setup(|app| {
            info!("{}", print_banner());
            get_os();
            start_cpu_monitor(app.handle().clone());
            let menu = MenuBuilder::new(app)
                .text("fichier", "Fichier")
                .text("apropos", "A propos")
                .text("fermer", "Fermer")
                .build()?;

            app.set_menu(menu)?;

            Ok(())
        })
        // Gestion des appels depuis le frontend
        .invoke_handler(tauri::generate_handler![
            get_devices_list,
            start_capture,
            stop_capture,
            config_capture,
            get_config_capture,
            export_csv,
            reset_capture,
            export_logs,
            convert_from_pcap_list,
            add_label,
            set_filter
        ])
        // Lancement de l'application
        .run(tauri::generate_context!())
}
