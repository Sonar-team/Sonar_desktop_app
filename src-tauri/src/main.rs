// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), tauri::Error> {
    if sonar_lib::startup_smoke::is_requested() {
        std::process::exit(sonar_lib::startup_smoke::run());
    }

    sonar_lib::run()
}
