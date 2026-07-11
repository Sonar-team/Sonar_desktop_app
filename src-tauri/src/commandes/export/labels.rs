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

    // Verrou court : snapshot des labels résolus seulement. L'écriture
    // disque se fait hors verrou pour ne pas bloquer le pipeline de capture.
    let rows = state.lock()?.export_labels();

    FlowMatrix::write_label_rows_to_csv(&rows, &path)?;
    Ok(())
}
