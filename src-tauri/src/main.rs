// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sonar_lib::{capture_packet::get_interfaces, scan_until_interrupt};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_interfaces_tab,
            print_selected_interface,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command(rename_all = "snake_case")]
fn get_interfaces_tab() -> Vec<String> {
    
    get_interfaces()
}

#[tauri::command(rename_all = "snake_case")]
fn print_selected_interface(interface_name: String) {
    println!("You have selected the interface: {}", interface_name);
    scan_until_interrupt(&interface_name);
    
}