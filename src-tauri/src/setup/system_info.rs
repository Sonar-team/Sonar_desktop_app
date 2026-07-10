//! Monitoring système : usage CPU moyen émis périodiquement au frontend et
//! énumération des interfaces réseau locales.

use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tauri::{AppHandle, Emitter};

/// Payload de l'événement `cpu_usage_update` (usage CPU moyen, en %).
#[derive(Debug, Serialize, Clone)]
pub struct SystemInfo {
    pub cpu_usage: f32,
}

/// Démarre la tâche de fond qui émet chaque seconde l'usage CPU moyen au
/// frontend (événement `cpu_usage_update`, affiché dans la barre de statut).
pub fn start_cpu_monitor(app_handle: AppHandle) -> Result<(), tauri::Error> {
    let sys = Arc::new(Mutex::new(System::new_all()));

    tauri::async_runtime::spawn({
        let sys = Arc::clone(&sys);
        async move {
            loop {
                let average = match sys.lock() {
                    Ok(mut sys) => {
                        sys.refresh_cpu_usage();

                        let cpus = sys.cpus();
                        if cpus.is_empty() {
                            0.0
                        } else {
                            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to lock system info mutex: {err}");
                        continue;
                    }
                };

                let info = SystemInfo { cpu_usage: average };
                let _ = app_handle.emit("cpu_usage_update", info);

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    });

    Ok(())
}

/// Interfaces réseau locales (via `netdev`), journalisées au passage.
pub fn get_interfaces() -> Vec<netdev::Interface> {
    let interfaces = netdev::get_interfaces();

    for iface in &interfaces {
        print!("Interface: {}", iface.name);

        // MAC
        if let Some(mac) = iface.mac_addr {
            print!(", Mac: {}", mac);
        } else {
            print!(", Mac: none");
        }

        // IPv4
        for ipv4 in &iface.ipv4 {
            print!(", IPv4: {}", ipv4);
        }

        // IPv6
        for ipv6 in &iface.ipv6 {
            print!(", IPv6: {}", ipv6);
        }
        println!();
    }
    interfaces.clone()
}
