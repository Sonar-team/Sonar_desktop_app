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
    labels_to_matrix, normalize_label_key, resolve_label_conflicts,
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

    /// Capture réelle de tunnels CAPWAP, versionnée dans le dépôt : corpus
    /// public nDPI (`tests/cfgs/default/pcap/capwap.pcap`, projet ntop/nDPI),
    /// 422 paquets — canal data 5247 transportant le trafic du client
    /// `kawai-ipad3` (DHCP, mDNS, ICMPv6) à travers le tunnel. Remplace la
    /// capture de mission LOC42.pcapng, non publiable, dont l'absence faisait
    /// passer les tests de tunnels pour verts sans les exécuter (#151).
    pub(super) fn tunnel_pcap() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/ndpi_capwap.pcap")
    }

    pub(super) fn tshark_corpus(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/pcaps/import/pcap_tshark_corpus")
            .join(name)
    }
}
