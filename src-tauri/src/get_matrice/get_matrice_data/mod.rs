use tauri::State;
use log::error;

use crate::tauri_state::SonarState;

pub fn get_matrice_data(shared_vec_infopackets: State<SonarState>) -> Result<String, String> {
    // Attempt to acquire the lock on the shared state
    match shared_vec_infopackets.0.lock() {
        Ok(matrice) => {
            // Serialize the hash map to a JSON string
            serde_json::to_string(&*matrice).map_err(|e| {
                let err_msg = format!("Serialization error: {}", e);
                error!("{}", err_msg);
                err_msg
            })
        },
        Err(_) => {
            let err_msg = "Failed to lock the mutex".to_string();
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}