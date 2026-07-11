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

    // Verrou court : snapshot des lignes seulement. L'écriture disque se
    // fait hors verrou pour ne pas bloquer le pipeline de capture (le
    // processing thread verrouille la matrice à chaque paquet).
    let rows = state.lock()?.to_flat_vec();

    FlowMatrix::write_rows_to_csv(&rows, &path)?;
    Ok(())
}
