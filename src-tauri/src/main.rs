// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;

use sonar_desktop_app::{print_banner, scan_until_interrupt, capture_packet::get_interfaces};
use tauri::Manager;

extern crate sonar_desktop_app;

fn main() {
    println!("{}", print_banner());
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_interfaces_tab,
            get_selected_interface,
            save_to_csv,
            save_file_from_frontend
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(rename_all = "snake_case")]
fn get_interfaces_tab() -> Vec<String> {
    get_interfaces()
}

#[tauri::command(rename_all = "snake_case")]
fn get_selected_interface(window: tauri::Window, interface_name: String) {
    let app = window.app_handle();
    println!("You have selected the interface: {}", interface_name);
    thread::spawn(move || {
        let _ = scan_until_interrupt(app, "oui.csv", &interface_name);
    });
}

#[tauri::command(rename_all = "snake_case")]
fn save_to_csv() {
    println!("save to csv");
}

use tauri::api::dialog::FileDialogBuilder;
use std::fs::File;
use std::path::PathBuf;

#[tauri::command]
fn save_file_from_frontend() {
    FileDialogBuilder::new()
        .set_title("Enregistrer le fichier")
        .add_filter("Texte", &["txt", "md"])
        .save_file(move |file_path: Option<PathBuf>| {
            if let Some(path) = file_path {
                match File::create(path) {
                    Ok(mut _file) => {
                        // Ici, vous pouvez éventuellement écrire dans le fichier
                        // ou faire d'autres opérations
                    }
                    Err(e) => {
                        // Gérer l'erreur de création du fichier
                        println!("Erreur lors de la création du fichier : {}", e);
                    }
                }
            }
        });
}

