use std::sync::{Arc, Mutex};

use tauri::{command, State};

use crate::{
    errors::{export::ExportError, CaptureStateError},
    state::flow_matrix::FlowMatrix,
};

#[command(async)]
pub fn export_csv(
    state: State<'_, Arc<Mutex<FlowMatrix>>>,
    path: String,
) -> Result<(), CaptureStateError> {
    if path.trim().is_empty() {
        return Err(CaptureStateError::Export(ExportError::EmptyPath));
    }

    // Verrou + export (I/O) : la commande est déjà déplacée hors du thread UI
    let guard = state.lock().unwrap();

    guard.export_to_csv(path)?;
    Ok(())
}
