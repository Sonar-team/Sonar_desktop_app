use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Clone)]
pub struct SystemInfo {
    pub cpu_usage: f32,
}

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
