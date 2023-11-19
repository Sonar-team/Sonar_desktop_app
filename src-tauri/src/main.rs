// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sonar_desktop_app::{print_banner, scan_until_interrupt, capture_packet::get_interfaces};
use tauri::Manager;

extern crate sonar_desktop_app;

fn main() {
    println!("{}", print_banner());
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_interfaces_tab,
            get_selected_interface,
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
    let _ = scan_until_interrupt(app, "oui",&interface_name);
}