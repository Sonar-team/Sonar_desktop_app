use std::fs;
use std::path::PathBuf;
use tauri::command;

use crate::errors::export::ExportError;


#[command(async)]
pub fn export_logs(destination: String) -> Result<String, ExportError> {
    let log_dir: PathBuf = {
        #[cfg(target_os = "linux")]
        {
            let base = std::env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .unwrap_or_else(|| PathBuf::from("/"))
                        .join(".local/share")
                });
            base.join("fr.sonar.app/logs")
        }

        #[cfg(target_os = "windows")]
        {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
                .join("fr.sonar.app\\logs")
        }

        #[cfg(target_os = "macos")]
        {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/Users/Shared"))
                .join("Library/Logs/fr.sonar.app")
        }
    };

    if !log_dir.exists() {
        return Err(ExportError::LogNotFound);
    }

    let destination = PathBuf::from(destination);

    if !destination.exists() {
        fs::create_dir_all(&destination)
            .map_err(|e| ExportError::Io(format!("create_dir_all: {}", e)))?;
    }

    for entry in fs::read_dir(&log_dir).map_err(|e| ExportError::Io(format!("read_dir: {}", e)))? {
        let entry = entry.map_err(|e| ExportError::Io(format!("entry: {}", e)))?;
        let src_path = entry.path();
        if src_path.is_file() {
            let file_name = src_path.file_name().unwrap();
            let dest_path = destination.join(file_name);
            fs::copy(&src_path, &dest_path)
                .map_err(|e| ExportError::Io(format!("copy: {}", e)))?;
        }
    }

    Ok("Logs exportés avec succès".to_string())
}

