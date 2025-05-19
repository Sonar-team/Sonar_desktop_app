use serde::Serialize;
use sysinfo::System;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Clone)]
pub struct SystemInfo {
    pub cpu_usage: f32,
}

pub fn start_cpu_monitor(app_handle: AppHandle) {
    // Partage du System entre les itérations avec Mutex (thread-safe)
    let sys = Arc::new(Mutex::new(System::new_all()));

    tauri::async_runtime::spawn({
        let sys = Arc::clone(&sys);
        async move {
            loop {
                {
                    let mut sys = sys.lock().unwrap();
                    sys.refresh_cpu_usage();

                    let cpus = sys.cpus();
                    if cpus.is_empty() {
                        continue;
                    }

                    // Moyenne des usages CPU
                    let average: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;

                    let info = SystemInfo {
                        cpu_usage: average,
                    };
                    // println!("CPU Usage: {}%", info.cpu_usage);
                    let _ = app_handle.emit("cpu_usage_update", info);
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    });
}
