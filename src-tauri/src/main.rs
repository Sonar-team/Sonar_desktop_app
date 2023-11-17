// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, time::Duration};

use sonar_desktop_app::print_banner;
use sonar_lib::{capture_packet::get_interfaces, scan_until_interrupt};
use tauri::Manager;

extern crate sonar_desktop_app;

fn main() {
    println!("{}", print_banner());
    tauri::Builder::default()
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            let mut count = 0;
    
            // Spawn a new thread to emit events
            thread::spawn(move || {
            loop {
                // Emit an event with the incremented count
                main_window.emit("counter", count).unwrap();
    
                // Increment the count
                count += 1;
    
                // Wait for a second
                thread::sleep(Duration::from_secs(1));
            }
            });
    
            Ok(())
        })
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