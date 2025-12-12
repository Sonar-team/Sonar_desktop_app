use colored::Colorize;
use log::info;
use tauri::AppHandle;

pub mod labels;
pub mod system_info;

/// Log des informations sur le système hôte.
///
/// - Plateforme retournée par `tauri_plugin_os::platform()`
/// - OS + architecture de compilation (`std::env::consts`)
pub fn print_os_infos() {
    let platform = tauri_plugin_os::platform();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    info!("Host platform      : {}", platform);
    info!("Host OS / Arch     : {} / {}", os, arch);
}

/// Affiche une bannière ASCII colorée (utilisée au démarrage dans les logs).
pub fn print_banner() -> String {
    // ASCII art banner
    let banner = r"
    _________                           
   /   _____/ ____   ____ _____ _______ 
   \_____  \ /  _ \ /    \\__  \\_  __ \
   /        (  <_> )   |  \/ __ \|  | \/
  /_______  /\____/|___|  (____  /__|   
          \/            \/     \/          
   ";

    // La bannière est colorée en vert avant d'être retournée.
    banner.green().to_string()
}

/// Log des informations de l'application SONAR.
///
/// - Nom / version (issus de `tauri.conf.json` / `Cargo.toml`)
/// - Identifiant Tauri (`identifier`)
/// - Licence (depuis `Cargo.toml` -> `CARGO_PKG_LICENSE`)
/// - Homepage / repo si défini (`CARGO_PKG_HOMEPAGE`)
/// - Mode build (debug / release)
pub fn log_sonar_version(app: &AppHandle) {
    let pkg = app.package_info();
    let cfg = app.config();

    // Ces variables viennent de [package] dans ton Cargo.toml
    // Assure-toi d’avoir bien:
    // license = "MIT"
    // homepage = "https://github.com/ton-org/sonar" (par exemple)
    let license: &str = env!("CARGO_PKG_LICENSE");
    let homepage: &str = option_env!("CARGO_PKG_HOMEPAGE").unwrap_or("");

    let build_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    info!("App name           : {}", pkg.name);
    info!("App identifier     : {}", cfg.identifier);
    info!("App version        : {}", pkg.version);
    info!("Build profile      : {}", build_profile);
    info!("License            : {}", license);
    if !homepage.is_empty() {
        info!("Homepage / repo    : {}", homepage);
    }
}
