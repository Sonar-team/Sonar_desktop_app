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

    // Verrou court : snapshot des lignes, du DLT et du contexte seulement.
    // L'écriture disque se fait hors verrou pour ne pas bloquer le pipeline
    // de capture (le processing thread verrouille la matrice à chaque paquet).
    let (rows, link_type, context) = {
        let matrix = state.lock()?;
        (
            matrix.to_flat_vec(),
            matrix.link_type,
            matrix.context.clone(),
        )
    };

    // Un export CSV porte le contexte du relevé dans son préambule `#SFMS`
    // (#154) : le fichier reste qualifié une fois sorti de l'application.
    FlowMatrix::write_rows_to_csv_with_context(&rows, link_type, &context, &path)?;
    Ok(())
}
