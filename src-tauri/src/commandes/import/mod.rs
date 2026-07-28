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
pub use pcap::{cancel_import, convert_from_pcap_list};

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use crate::{errors::CaptureStateError, events::CaptureEvent, state::capture::CaptureState};

/// Déduplique une liste de fichiers à importer par **identité réelle** et non
/// par égalité de chaîne : un même fichier désigné deux fois — doublon exact,
/// chemin relatif, composants `.`/`..`, lien symbolique — ne doit être importé
/// qu'une fois, sinon ses paquets seraient comptés deux fois dans la matrice
/// (#161). La première occurrence est conservée, dans l'ordre de sélection,
/// et c'est son libellé d'origine qui est réutilisé (colonne `origin`).
///
/// Un chemin non canonicalisable (fichier absent, droits insuffisants) est
/// conservé tel quel : l'échec doit venir de la commande d'import, avec son
/// message précis, jamais d'une normalisation qui l'escamoterait.
pub(crate) fn dedupe_paths_by_identity(paths: &[String]) -> Vec<String> {
    let mut seen: HashSet<PathBuf> = HashSet::with_capacity(paths.len());
    let mut unique = Vec::with_capacity(paths.len());
    for path in paths {
        let identity = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
        if seen.insert(identity) {
            unique.push(path.clone());
        }
    }
    unique
}

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

#[cfg(test)]
mod dedupe_tests {
    use super::dedupe_paths_by_identity;
    use super::test_support::TempDir;
    use std::fs;

    /// Doublon exact : le cas du sélecteur natif qui renvoie deux fois le
    /// même fichier, ou de la resélection après un drag & drop.
    #[test]
    fn exact_duplicate_is_imported_once() {
        let dir = TempDir::new("sonar-dedupe-exact");
        let file = dir.path().join("capture.pcap");
        fs::write(&file, b"x").unwrap();
        let path = file.to_string_lossy().to_string();

        let unique = dedupe_paths_by_identity(&[path.clone(), path.clone()]);

        assert_eq!(unique, vec![path]);
    }

    /// Chemin relatif et composants `.`/`..` : chaînes différentes, même
    /// fichier — la déduplication par égalité de chaîne les laissait passer
    /// et doublait les compteurs de la matrice.
    #[test]
    fn same_file_reached_by_a_detour_is_imported_once() {
        let dir = TempDir::new("sonar-dedupe-detour");
        let file = dir.path().join("capture.pcap");
        fs::write(&file, b"x").unwrap();
        let direct = file.to_string_lossy().to_string();
        let detour = dir
            .path()
            .join("sub")
            .join("..")
            .join("capture.pcap")
            .to_string_lossy()
            .to_string();
        fs::create_dir_all(dir.path().join("sub")).unwrap();

        let unique = dedupe_paths_by_identity(&[direct.clone(), detour]);

        assert_eq!(
            unique,
            vec![direct],
            "seule la première désignation du fichier est conservée"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_to_an_already_selected_file_is_imported_once() {
        let dir = TempDir::new("sonar-dedupe-symlink");
        let file = dir.path().join("capture.pcap");
        fs::write(&file, b"x").unwrap();
        let link = dir.path().join("alias.pcap");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        let direct = file.to_string_lossy().to_string();

        let unique =
            dedupe_paths_by_identity(&[direct.clone(), link.to_string_lossy().to_string()]);

        assert_eq!(unique, vec![direct]);
    }

    /// Fichiers réellement distincts : aucun ne doit disparaître, et l'ordre
    /// de sélection est préservé (il détermine l'ordre d'import et la
    /// progression affichée).
    #[test]
    fn distinct_files_are_all_kept_in_order() {
        let dir = TempDir::new("sonar-dedupe-distinct");
        let mut paths = Vec::new();
        for name in ["b.pcap", "a.pcap", "c.pcap"] {
            let file = dir.path().join(name);
            fs::write(&file, b"x").unwrap();
            paths.push(file.to_string_lossy().to_string());
        }

        assert_eq!(dedupe_paths_by_identity(&paths), paths);
    }

    /// Un chemin inexistant n'est pas canonicalisable : il doit être conservé
    /// pour que l'import échoue avec son erreur précise (fichier introuvable)
    /// au lieu d'être silencieusement retiré de la liste.
    #[test]
    fn unresolvable_path_is_kept_for_the_import_error() {
        let missing = "/sonar/chemin/inexistant.pcap".to_string();

        let unique = dedupe_paths_by_identity(&[missing.clone(), missing.clone()]);

        assert_eq!(
            unique,
            vec![missing],
            "conservé une fois : ni escamoté, ni dupliqué"
        );
    }
}
