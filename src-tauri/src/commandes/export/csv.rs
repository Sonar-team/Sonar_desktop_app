//! Commande d'export de la matrice de flux en CSV.

use std::sync::{Arc, Mutex};

use tauri::{State, command};

use crate::{
    errors::{CaptureStateError, export::ExportError},
    state::flow_matrix::FlowMatrix,
};

/// Exporte la matrice de flux courante vers `path` (format de
/// `FlowMatrix::export_to_csv`, réimportable par `import_matrix_files`).
#[command(async)]
pub fn export_csv(
    state: State<'_, Arc<Mutex<FlowMatrix>>>,
    path: String,
) -> Result<(), CaptureStateError> {
    if path.trim().is_empty() {
        return Err(CaptureStateError::Export(ExportError::EmptyPath));
    }

    // Verrou + export (I/O) : la commande est déjà déplacée hors du thread UI
    let guard = state.lock()?;

    guard.export_to_csv(path)?;
    Ok(())
}
