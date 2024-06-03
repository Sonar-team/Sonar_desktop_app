// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::File, io::BufWriter, sync::Mutex};

use log::info;

use sonar_desktop_app::{
    cli::print_banner,
    get_hostname::hostname_to_s,
    get_interfaces::get_interfaces,
    get_matrice::{get_graph_data::get_graph_data, get_matrice_data::get_matrice_data},
    save_packets::{cmd_save_packets_to_csv, cmd_save_packets_to_excel, MyError},
    sniff::scan_until_interrupt,
    tauri_state::SonarState,
};
use tauri::{AppHandle, Manager};
use tauri_plugin_log::LogTarget;


use resvg::tiny_skia::{Pixmap, Transform};
use usvg::{Tree, Options};


extern crate sonar_desktop_app;

fn main() {
    println!("{}", print_banner());

    let builder = tauri::Builder::default();

    builder
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event.event() {
                std::process::exit(0);
            }
        })
        .manage(Mutex::new(SonarState::new()))
        .manage(SonarState::new())
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
        ])
        .setup(move |app| {
            let app_handle = app.handle();

            // Event listener for before-quit
            app_handle.listen_global("tauri://before-quit", move |_| {
                info!("Quit event received");
            });
            app_handle.manage(Mutex::new(SonarState::new()));

            Ok(())
        })
        .plugin(devtools::init())
        // .plugin(
        //     tauri_plugin_log::Builder::default()
        //         .targets([LogTarget::LogDir, LogTarget::Stdout, LogTarget::Webview])
        //         .build(),
        // )
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
fn save_packets_to_csv(file_path: String, app: AppHandle) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du CSV: {}", &file_path);
    cmd_save_packets_to_csv(file_path, app)
}

#[tauri::command(async, rename_all = "snake_case")]
fn save_packets_to_excel(file_path: String, app: AppHandle) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du Excel: {}", &file_path);
    cmd_save_packets_to_excel(file_path, app)
}

#[tauri::command(async)]
fn get_matrice(app: AppHandle) -> Result<String, String> {
    match get_matrice_data(app) {
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
fn get_graph_state(app: AppHandle) -> Result<String, String> {
    get_graph_data(app)
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
fn toggle_ipv6_filter(app: AppHandle) {
    let state = app.state::<Mutex<SonarState>>(); // Acquire a lock
    let mut state_guard = state.lock().unwrap();
    state_guard.toggle_filter_ipv6();
    info!("etat du filtre {:?}", state_guard.filter_ipv6);
}

#[tauri::command(async)]
fn toggle_pause(app: AppHandle) {
    let state = app.state::<Mutex<SonarState>>(); // Acquire a lock
    let mut state_guard = state.lock().unwrap();
    state_guard.toggle_actif();
    println!("etat actif");
    info!("etat du filtre {:?}", state_guard.actif);
}
