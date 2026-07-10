//! Commandes d'import, découpées par domaine :
//! - [`labels`] : fichiers de labels CSV (validation, conflits, arbitrage) ;
//! - [`matrix`] : matrices de flux CSV (fusion multi-fichiers, provenance) ;
//! - [`pcap`] : conversion de fichiers PCAP en matrice + graphe ;
//! - [`timing`] : instrumentation optionnelle (feature `capture_timing`).

mod labels;
mod matrix;
mod pcap;
mod timing;

pub use labels::{
    LabelConflictStore, clear_label_store, get_label_conflicts, get_label_rows, import_label_file,
    labels_to_matrix, resolve_label_conflict,
};
pub use matrix::{import_matrix_file, import_matrix_files, is_matrix_empty};
pub use pcap::convert_from_pcap_list;

use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use crate::{errors::CaptureStateError, events::CaptureEvent, state::capture::CaptureState};

/// Un `Channel` Tauri est lié à une seule commande ; pendant une capture live
/// les événements doivent passer par le channel de la capture pour arriver au
/// front. Retourne ce dernier s'il existe, sinon celui passé à la commande.
fn event_channel(
    capture_state: &State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<Channel<CaptureEvent<'static>>, CaptureStateError> {
    let live = capture_state.lock()?.on_event.clone();
    Ok(live.unwrap_or(on_event))
}

#[cfg(test)]
mod test_support {
    use std::fs;
    use std::path::{Path, PathBuf};

    pub(super) struct TempDir(PathBuf);

    impl TempDir {
        pub(super) fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(name);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        pub(super) fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Capture réelle utilisée par les tests de tunnels, volontairement non
    /// versionnée (données de mission, dépôt public). Retourne `None` quand
    /// elle est absente (ex. CI) : le test se saute proprement.
    pub(super) fn local_tunnel_pcap() -> Option<PathBuf> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/LOC42.pcapng");
        if path.exists() {
            Some(path)
        } else {
            eprintln!(
                "test sauté : test_files/LOC42.pcapng absent (capture locale non versionnée)"
            );
            None
        }
    }
}
