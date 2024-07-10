// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use log::info;

use sonar_desktop_app::{
    cli::print_banner,
    get_hostname::hostname_to_s,
    get_interfaces::get_interfaces,
    sniff::scan_until_interrupt,
    tauri_state::{MyError, SonarState},
};
use tauri::{generate_handler, AppHandle, Manager, State};
// use tauri_plugin_log::LogTarget;

use resvg::tiny_skia::{Pixmap, Transform};
use usvg::{Tree, Options};

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
fn save_packets_to_csv(file_path: String, state: State<'_,Arc<Mutex<SonarState>>>) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du CSV: {}", &file_path);
    let locked_state = state.lock().unwrap();
    locked_state.cmd_save_packets_to_csv(file_path)
    
}

#[tauri::command(async, rename_all = "snake_case")]
fn save_packets_to_excel(file_path: String, state: State<'_,Arc<Mutex<SonarState>>>) -> Result<(), MyError> {
    info!("Chemin d'enregistrement du Excel: {}", &file_path);
    let locked_state = state.lock().unwrap();

    locked_state.cmd_save_packets_to_excel(file_path)
    
}

#[tauri::command(async)]
fn get_matrice(state: State<'_,Arc<Mutex<SonarState>>>) -> Result<String, String> {
    //println!("  getmarice");
    let locked_state = state.lock().map_err(|_| "Failed to lock state".to_string())?;

    match locked_state.get_matrice_data() {
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

#[tauri::command(async)]
fn get_graph_state(state: State<'_,Arc<Mutex<SonarState>>>) -> Result<String, String> {
    let locked_state = state.lock().map_err(|_| "Failed to lock state".to_string())?;

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
    let locked_state = state.lock().map_err(|_| "Failed to lock state".to_string())?;

    locked_state.toggle_filter_ipv6();
    info!("etat du filtre {:?}", locked_state.filter_ipv6);
    Ok(())
}

#[tauri::command(async)]
fn toggle_pause(state: State<'_, Arc<Mutex<SonarState>>>) -> Result<(), String> {
    let locked_state = state.lock().map_err(|_| "Failed to lock state".to_string())?;
    locked_state.toggle_actif();
    println!("etat actif");
    info!("etat du filtre {:?}", locked_state.actif);
    Ok(())
}
