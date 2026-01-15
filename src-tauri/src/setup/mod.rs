use colored::Colorize;
use log::info;
use tauri::AppHandle;

pub mod labels;
pub mod system_info;

pub fn log_host_and_app_snapshot(app: &AppHandle) {
    // ---------- OS / Host (tauri_plugin_os) ----------
    // Note: la lib `tauri_plugin_os` expose des helpers côté Rust.
    let platform = tauri_plugin_os::platform(); // std::env::consts::OS
    let family = tauri_plugin_os::family(); // std::env::consts::FAMILY
    let arch = tauri_plugin_os::arch(); // std::env::consts::ARCH
    let exe_ext = tauri_plugin_os::exe_extension(); // std::env::consts::EXE_EXTENSION
    let os_type = tauri_plugin_os::type_().to_string(); // linux/windows/macos/ios/android
    let os_version = tauri_plugin_os::version().to_string();
    let locale = tauri_plugin_os::locale().unwrap_or_else(|| "unknown".to_string());
    let hostname = tauri_plugin_os::hostname();

    info!("--- Host / OS ---");
    info!("Host platform      : {}", platform);
    info!("Host family        : {}", family);
    info!("Host type          : {}", os_type);
    info!("Host version       : {}", os_version);
    info!("Host arch          : {}", arch);
    info!(
        "Exe extension      : {}",
        if exe_ext.is_empty() {
            "(none)"
        } else {
            exe_ext
        }
    );
    info!("Locale             : {}", locale);
    info!("Hostname           : {}", hostname);

    // Optionnel mais utile pour debug d’exécution
    if let Ok(cwd) = std::env::current_dir() {
        info!("Current dir        : {}", cwd.display());
    }
    if let Ok(exe) = std::env::current_exe() {
        info!("Current exe        : {}", exe.display());
    }

    // ---------- Application (Tauri + Cargo metadata) ----------
    let pkg = app.package_info();
    let cfg = app.config();

    // Ces env! sont injectées par Cargo au build-time.
    // Attention: env! panique si la variable n’existe pas. option_env! = safe.
    let license: &str = env!("CARGO_PKG_LICENSE");
    let authors: &str = env!("CARGO_PKG_AUTHORS");
    let description: &str = option_env!("CARGO_PKG_DESCRIPTION").unwrap_or("");
    let repository: &str = option_env!("CARGO_PKG_REPOSITORY").unwrap_or("");
    let homepage: &str = option_env!("CARGO_PKG_HOMEPAGE").unwrap_or("");
    let rust_version: &str = option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("");
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    // Variables utiles si présentes (selon environnement CI)
    let git_sha = option_env!("GIT_SHA").unwrap_or(""); // à injecter toi-même dans la CI
    let build_time = option_env!("BUILD_TIME").unwrap_or(""); // idem

    info!("--- Application ---");
    info!("App name           : {}", pkg.name);
    info!("App identifier     : {}", cfg.identifier);
    info!("App version        : {}", pkg.version);
    info!("Build profile      : {}", profile);
    info!("License            : {}", license);
    info!("Authors            : {}", authors);

    if !description.is_empty() {
        info!("Description        : {}", description);
    }
    if !rust_version.is_empty() {
        info!("Rust version req   : {}", rust_version);
    }
    if !homepage.is_empty() {
        info!("Homepage           : {}", homepage);
    }
    if !repository.is_empty() {
        info!("Repository         : {}", repository);
    }
    if !git_sha.is_empty() {
        info!("Git SHA            : {}", git_sha);
    }
    if !build_time.is_empty() {
        info!("Build time         : {}", build_time);
    }

    // ---------- Target / compilation (très utile en cross-build) ----------
    // Beaucoup sont disponibles via env! et cfg!.
    // TARGET n'est pas toujours exposée automatiquement => on la met en option.
    let target = option_env!("TARGET").unwrap_or("unknown"); // selon CI/cargo
    let target_os = std::env::consts::OS;
    let target_arch = std::env::consts::ARCH;

    info!("--- Build / Target ---");
    info!("Target triple      : {}", target);
    info!("Target OS / Arch   : {} / {}", target_os, target_arch);
    info!("Debug assertions   : {}", cfg!(debug_assertions));
    info!(
        "Panic strategy     : {}",
        if cfg!(panic = "abort") {
            "abort"
        } else {
            "unwind"
        }
    );

    // ---------- Platform specific (optionnel) ----------
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let uid = unsafe { libc::geteuid() };
        let gid = unsafe { libc::getegid() };
        info!("--- Unix details ---");
        info!("EUID / EGID        : {} / {}", uid, gid);

        if let Ok(meta) = std::fs::metadata("/") {
            info!("Root dev/inode     : {}/{}", meta.dev(), meta.ino());
        }
    }

    #[cfg(windows)]
    {
        info!("--- Windows details ---");
        // Sur Windows, on peut aller plus loin via winapi/windows-rs,
        // mais je n’ajoute rien ici pour éviter d’alourdir tes deps.
    }
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
