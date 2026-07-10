//! Point d'entrée du binaire SONAR : délègue à [`sonar_lib::run`], sauf en
//! mode smoke test (`--startup-smoke`) où l'application vérifie seulement
//! qu'elle démarre puis rend un code de sortie.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), tauri::Error> {
    if sonar_lib::startup_smoke::is_requested() {
        std::process::exit(sonar_lib::startup_smoke::run());
    }

    sonar_lib::run()
}
