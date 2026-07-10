//! Commande d'export des labels en CSV.

use std::sync::{Arc, Mutex};

use tauri::{State, command};

use crate::{
    errors::{CaptureStateError, export::ExportError},
    state::flow_matrix::FlowMatrix,
};

/// Exporte les labels courants (matrice de flux) vers un fichier CSV
/// `mac,ip,label`. Les labels partiels édités à la main pendant la capture
/// sont complétés à partir des couples (MAC, IP) observés dans la matrice.
#[command(async)]
pub fn export_label_file(
    state: State<'_, Arc<Mutex<FlowMatrix>>>,
    path: String,
) -> Result<(), CaptureStateError> {
    if path.trim().is_empty() {
        return Err(CaptureStateError::Export(ExportError::EmptyPath));
    }

    // Verrou + export (I/O) : la commande est déjà déplacée hors du thread UI.
    let guard = state.lock()?;

    guard.export_labels_to_csv(path)?;
    Ok(())
}
