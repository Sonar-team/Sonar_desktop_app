use std::sync::Arc;

use parking_lot::Mutex;
use tauri::State;

use crate::state::flow_matrix::FlowMatrix;

#[tauri::command]
pub fn add_label(
    matrix:  State<'_, Arc<Mutex<FlowMatrix>>>, 
    mac: String, 
    ip: String, 
    label: String
) -> Result<(), String> {
    let mut matrix = matrix.lock();
    matrix.add_label(mac, ip, label);
    Ok(())
}
    