// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use log::info;

use sonar_desktop_app::{
    cli::print_banner,
    get_interfaces::get_interfaces,
    get_matrice::{get_graph_data::get_graph_data, get_matrice_data::get_matrice_data},
    save_packets::{cmd_save_packets_to_csv, cmd_save_packets_to_excel, MyError},
    sniff::scan_until_interrupt,
    tauri_state::SonarState,
};
use tauri::{Manager, State};
use tauri_plugin_log::LogTarget;

extern crate sonar_desktop_app;

fn main() {
    println!("{}", print_banner());

    // #[cfg(debug_assertions)] // only enable instrumentation in development builds
    // let devtools = devtools::init();

    let builder = tauri::Builder::default();

    // #[cfg(debug_assertions)]
    // let builder = builder.plugin(devtools);

    builder
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                std::process::exit(0);
            }
        })
        .manage(SonarState(Arc::new(Mutex::new(Vec::new()))))
        .invoke_handler(tauri::generate_handler![
            get_interfaces_tab,
            get_selected_interface,
            save_packets_to_csv,
            save_packets_to_excel,
            get_hash_map_state,
            get_graph_state,
            write_file
        ])
        .setup(move |app| {
            let app_handle = app.handle();

            // Event listener for before-quit
            app_handle.listen_global("tauri://before-quit", move |_| {
                info!("Quit event received");
            });

            Ok(())
        })
        //.plugin(devtools::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(rename_all = "snake_case")]
fn get_interfaces_tab() -> Vec<String> {
    get_interfaces()
}

#[tauri::command(async, rename_all = "snake_case")]
fn get_selected_interface(
    window: tauri::Window,
    interface_name: String,
    state: tauri::State<SonarState>,
) {
    let app = window.app_handle();
    info!("Interface sélectionée: {}", interface_name);
    scan_until_interrupt(app, &interface_name, state);
}

#[tauri::command(async, rename_all = "snake_case")]
fn save_packets_to_csv(file_path: String, state: State<SonarState>) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du CSV: {}", &file_path);
    cmd_save_packets_to_csv(file_path, state)
}

#[tauri::command(async, rename_all = "snake_case")]
fn save_packets_to_excel(file_path: String, state: State<SonarState>) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du Excel: {}", &file_path);
    cmd_save_packets_to_excel(file_path, state)
}

#[tauri::command]
fn get_hash_map_state(shared_hash_map: State<SonarState>) -> Result<String, String> {
    match get_matrice_data(shared_hash_map) {
        Ok(data) => {
            println!("Data: {}", data); // Utilisez log::info si vous avez configuré un logger
            Ok(data)
        }
        Err(e) => {
            println!("Error: {}", e); // Utilisez log::error pour les erreurs
            Err(e)
        }
    }
}


#[tauri::command]
fn get_graph_state(shared_hash_map: State<SonarState>) -> Result<String, String> {
    get_graph_data(shared_hash_map)
}

#[tauri::command]
fn write_file(path: String, contents: String) -> Result<(), String> {
    info!("Chemin d'enregistrement du VSG: {}", &path);
    std::fs::write(path, contents).map_err(|e| e.to_string())
}
