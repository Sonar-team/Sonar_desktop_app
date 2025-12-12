use chrono::Local;
use commandes::{
    net_capture::{config_capture, get_config_capture, start_capture, stop_capture},
    net_interface::get_devices_list,
};
use log::info;

use std::sync::{Arc, Mutex};
use tauri::menu::MenuBuilder;

use crate::{
    commandes::{
        export::{csv::export_csv, logs::export_logs},
        flow_matrix::{add_label, get_label_list},
        import::convert_from_pcap_list,
        net_capture::{reset_capture, set_filter},
    },
    setup::{
        labels::read_labels, log_sonar_version, print_banner, print_os_infos,
        system_info::start_cpu_monitor,
    },
    state::{capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData},
};

mod commandes;
mod dto;
mod errors;
mod events;
mod setup;
mod state;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    // ▶️ Génération de la date/heure dynamique
    let now = Local::now();
    let filename = format!(
        "DR_SONAR_{}_{}",
        now.format("%Y-%m-%d"),
        now.format("%H-%M-%S"),
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        // Plugins
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .max_file_size(500_000)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some(filename),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // Etats partager dans l'application
        .manage(Arc::new(Mutex::new(CaptureState::new())))
        .manage(Arc::new(Mutex::new(FlowMatrix::new())))
        .manage(Arc::new(Mutex::new(GraphData::new())))
        // Menu
        .setup(|app| {
            info!("{}", print_banner());
            print_os_infos();
            read_labels(app.handle())?;
            log_sonar_version(app.handle());

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
            get_label_list,
            set_filter
        ])
        // Lancement de l'application
        .run(tauri::generate_context!())
}
