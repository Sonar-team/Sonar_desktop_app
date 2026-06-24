use chrono::Local;
use commandes::{
    net_capture::{config_capture, get_config_capture, start_capture, stop_capture},
    net_interface::get_devices_list,
};
use log::{error, info};

use std::sync::{Arc, Mutex};
use tauri::{Manager, menu::MenuBuilder};
use tauri_plugin_cli::CliExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::{
    commandes::{
        export::{csv::export_csv, logs::export_logs},
        flow_matrix::{add_label, get_label_list},
        import::{
            convert_from_pcap_list, import_label_file, is_matrix_empty, clear_label_store, get_label_rows
        },
        net_capture::{reset_capture, set_filter, start_capture_core},
    },
    setup::{
        about::about_message, labels::read_labels, log_host_and_app_snapshot, print_banner,
        system_info::start_cpu_monitor,
    },
    state::{
        capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData,
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

/// Main entry point for the application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    let now = Local::now();
    let filename = format!(
        "DR_SONAR_{}_{}",
        now.format("%Y-%m-%d"),
        now.format("%H-%M-%S")
    );

    let ctrl_c_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyC);

    let exit_code = 0;

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_cli::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    println!("{:?}", shortcut);
                    if shortcut == &ctrl_c_shortcut {
                        match event.state() {
                            ShortcutState::Pressed => {
                                println!("Ctrl-C Pressed!");
                                app.exit(exit_code);
                            }
                            ShortcutState::Released => {
                                println!("Ctrl-C Released!");
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
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
        .manage(Arc::new(Mutex::new(CaptureState::new())))
        .manage(Arc::new(Mutex::new(FlowMatrix::new())))
        .manage(Arc::new(Mutex::new(GraphData::new())))
        .manage(Arc::new(Mutex::new(PcInfoLabel::new())))
        .manage(Arc::new(Mutex::new(LabelStore::new())))
        .on_menu_event(|app, event| {
            if event.id() == "apropos" {
                app.dialog()
                    .message(about_message())
                    .title("A propos")
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
        })
        .setup({
            move |app| {
                info!("{}", print_banner());
                log_host_and_app_snapshot(app.app_handle());
                info!("Reading labels...");
                read_labels(app.handle())?;

                // CLI
                let Ok(cli_matches) = app.cli().matches() else {
                    println!("Une erreur est survenue lors de l'analyse des arguments");
                    return Ok(());
                };

                let headless_enabled = cli_matches
                    .args
                    .get("headless")
                    .map(|a| a.occurrences > 0)
                    .unwrap_or(false);

                println!("headless_enabled = {}", headless_enabled);
                println!("args: {:?}", cli_matches);
                app.global_shortcut().register(ctrl_c_shortcut)?;

                // handle the capture state here
                if !headless_enabled {
                    let _ = start_cpu_monitor(app.handle().clone());

                    let menu = MenuBuilder::new(app)
                        .text("fichier", "Fichier")
                        .text("apropos", "A propos")
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
                    //println!("labels: {:#?}", labels);
                    //setup::labels::add_labels_to_file(app.handle(), labels.clone())?;
                    //read_labels(app.handle())?;
                    //setup::labels::update_labels_in_state(app.handle())?;
                } else {
                    let capture_state = app.state::<Arc<Mutex<CaptureState>>>();
                    let config = get_config_capture(capture_state.clone());
                    println!("config: {:#?}", config);
                    start_capture_core(capture_state, app.handle().clone())?;
                }

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
            reset_capture,
            export_logs,
            convert_from_pcap_list,
            add_label,
            get_label_list,
            set_filter,
            get_label_rows,
            import_label_file,
            clear_label_store,
            is_matrix_empty
        ])
        .run(tauri::generate_context!())
}
