//! Rétention des fichiers de logs. Chaque lancement crée un fichier
//! `DR_SONAR_<date>_<heure>` : sans purge, ils s'accumulent indéfiniment
//! (#147). La rotation du plugin (`KeepSome`) borne une session ; cette
//! purge borne l'accumulation entre sessions.

use log::{info, warn};
use tauri::{AppHandle, Manager};

/// Nombre de fichiers de logs conservés (les plus récents).
const MAX_LOG_FILES: usize = 30;

/// Supprime les fichiers de logs les plus anciens au-delà de
/// [`MAX_LOG_FILES`]. Le nommage `DR_SONAR_YYYY-MM-DD_HH-MM-SS` rend le tri
/// lexical chronologique. Best-effort : une erreur n'empêche pas le
/// démarrage.
pub fn prune_old_logs(app: &AppHandle) {
    let Ok(log_dir) = app.path().app_log_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&log_dir) else {
        return;
    };

    let mut logs: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("DR_SONAR_"))
        })
        .collect();

    if logs.len() <= MAX_LOG_FILES {
        return;
    }

    logs.sort();
    let excess = logs.len() - MAX_LOG_FILES;
    for path in logs.into_iter().take(excess) {
        match std::fs::remove_file(&path) {
            Ok(()) => info!("Rétention logs : {} supprimé", path.display()),
            Err(e) => warn!(
                "Rétention logs : suppression de {} impossible : {e}",
                path.display()
            ),
        }
    }
}
