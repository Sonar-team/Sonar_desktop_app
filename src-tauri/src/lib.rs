//! Bibliothèque principale de SONAR : construit l'application Tauri
//! (plugins, état partagé, menu, fenêtre) et enregistre toutes les
//! commandes exposées au frontend.
//!
//! L'usage sans interface graphique n'existe pas ici : c'est le rôle de
//! `sonar-cli` (workspace `sonar-rust/`), cf. VISION.md « Deux formes,
//! un seul cœur ».

use chrono::Local;
use commandes::{
    net_capture::{config_capture, get_config_capture, start_capture, stop_capture},
    net_interface::get_devices_list,
};
use log::{error, info};

use std::sync::{Arc, Mutex};
use tauri::{
    Manager,
    menu::{MenuBuilder, SubmenuBuilder},
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::{
    commandes::{
        export::{csv::export_csv, labels::export_label_file, logs::export_logs},
        flow_matrix::{
            add_label, delete_label, get_label_list, get_labels_with_status, get_matrix_labels,
            update_label,
        },
        import::{
            LabelConflictStore, cancel_import, clear_label_store, convert_from_pcap_list,
            get_label_conflicts, get_label_rows, import_label_file, import_matrix_file,
            import_matrix_files, is_matrix_empty, resolve_label_conflicts,
        },
        net_capture::{reset_capture, set_filter},
        project::{get_recovery_offer, is_session_dirty, open_project, save_project},
    },
    setup::{
        about::{about_message, changelog_message},
        labels::read_labels,
        log_host_and_app_snapshot, print_banner,
        system_info::start_cpu_monitor,
    },
    state::{
        capture::{CaptureState, capture_config::CaptureConfig},
        flow_matrix::FlowMatrix,
        graph::GraphData,
        labels_list::{LabelStore, PcInfoLabel},
    },
};

mod commandes;
mod dto;
mod errors;
mod events;
mod setup;
pub mod startup_smoke;
mod state;
mod utils;

// Exposé pour le benchmark `examples/pool_bench.rs`.
pub use state::capture::capture_handle::threads::packet_buffer;

/// Gère les clics du menu natif (À propos, Changelog, Fermer).
fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    if event.id() == "version" {
        app.dialog()
            .message(about_message())
            .title("À propos de SONAR")
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    } else if event.id() == "changelog" {
        app.dialog()
            .message(changelog_message())
            .title("Changelog")
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::Ok)
            .show(|_| {});
    } else if event.id() == "fermer" {
        if let Some(window) = app.get_webview_window("main") {
            if let Err(close_error) = window.close() {
                error!("Failed to close main window: {close_error}");
            }
        } else {
            app.exit(0);
        }
    }
}

/// Recharge la configuration de capture persistée au démarrage, si présente.
fn restore_persisted_capture_config(app: &tauri::AppHandle) {
    let capture_state = app.state::<Arc<Mutex<CaptureState>>>();
    match CaptureConfig::load_persisted(app) {
        Ok(Some(config)) => match capture_state.lock() {
            Ok(mut state) => {
                info!("Configuration capture chargée depuis le disque");
                state.config = config;
            }
            Err(err) => error!("Impossible de charger la configuration capture: {err}"),
        },
        Ok(None) => {}
        Err(err) => error!("Configuration capture persistée ignorée: {err}"),
    }
}

/// Point d'entrée de l'application : configure les plugins, l'état partagé,
/// le menu et la fenêtre, puis démarre la boucle Tauri.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    let now = Local::now();
    let filename = format!(
        "DR_SONAR_{}_{}",
        now.format("%Y-%m-%d"),
        now.format("%H-%M-%S")
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // Une session garde au plus 5 fichiers rotatés de 500 Ko ;
                // l'accumulation entre sessions est bornée par
                // `setup::logs::prune_old_logs` au démarrage (#147).
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(5))
                .max_file_size(500_000)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some(filename),
                    },
                ))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        // Récents et préférences UI uniquement : le plugin écrit en
        // `fs::write` direct (non atomique, vérifié dans la source vendorée
        // 2.4.4) — la config de capture reste sur notre persistance atomique.
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(Arc::new(Mutex::new(CaptureState::new())))
        .manage(Arc::new(Mutex::new(FlowMatrix::new())))
        .manage(Arc::new(Mutex::new(GraphData::new())))
        .manage(Arc::new(Mutex::new(PcInfoLabel::new())))
        .manage(Arc::new(Mutex::new(LabelStore::new())))
        .manage(Arc::new(Mutex::new(LabelConflictStore::default())))
        .on_menu_event(handle_menu_event)
        .setup({
            move |app| {
                info!("{}", print_banner());
                log_host_and_app_snapshot(app.app_handle());
                setup::logs::prune_old_logs(app.app_handle());
                info!("Reading labels...");
                read_labels(app.handle())?;

                restore_persisted_capture_config(app.handle());

                // Persistance de session (#159) : offre de récupération
                // figée AVANT la pose de la sentinelle, puis autosave.
                let recovery_offer = commandes::project::init_session_persistence(app.handle());
                app.manage(recovery_offer);

                let _ = start_cpu_monitor(app.handle().clone());

                let apropos = SubmenuBuilder::new(app, "À propos")
                    .text("version", "À propos de SONAR")
                    .text("changelog", "Changelog")
                    .build()?;

                let menu = MenuBuilder::new(app)
                    .text("fichier", "Fichier")
                    .item(&apropos)
                    .text("fermer", "Fermer")
                    .build()?;

                app.set_menu(menu)?;

                tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("SONAR")
                .inner_size(1800.0, 950.0)
                .build()?;

                let interfaces = setup::system_info::get_interfaces();
                setup::labels::create_labels_from_network_interfaces(interfaces, app.handle())?;

                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_devices_list,
            start_capture,
            stop_capture,
            config_capture,
            get_config_capture,
            export_csv,
            export_label_file,
            reset_capture,
            export_logs,
            convert_from_pcap_list,
            cancel_import,
            add_label,
            update_label,
            delete_label,
            get_labels_with_status,
            get_label_list,
            get_matrix_labels,
            set_filter,
            get_label_rows,
            get_label_conflicts,
            resolve_label_conflicts,
            import_label_file,
            import_matrix_file,
            import_matrix_files,
            clear_label_store,
            is_matrix_empty,
            save_project,
            open_project,
            is_session_dirty,
            get_recovery_offer
        ])
        .build(tauri::generate_context!())?
        .run(|app_handle, event| {
            // Fermeture propre : sentinelle retirée, le prochain démarrage
            // ne proposera pas de récupération (#159). Un crash ne passe pas
            // ici — c'est exactement le signal recherché.
            if let tauri::RunEvent::Exit = event {
                commandes::project::remove_session_lock(app_handle);
            }
        });
    Ok(())
}
