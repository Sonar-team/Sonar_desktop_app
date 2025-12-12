use std::sync::{Arc, Mutex};
use tauri::{State, command};

use crate::state::flow_matrix::FlowMatrix;
// si tu veux un Result typé :
use crate::errors::CaptureStateError;

#[command]
pub fn add_label(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    mac: String,
    ip: String,
    label: String,
) -> Result<(), CaptureStateError> {
    let mut guard = matrix.lock()?;
    guard.add_label(mac, ip, label.clone());
    Ok(())
}

#[command]
pub fn get_label_list(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<Vec<String>, CaptureStateError> {
    let guard = matrix.lock()?;
    Ok(guard.get_label_list())
}
