//! Commande d'export des fichiers de logs de l'application.

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, command};

use crate::errors::CaptureStateError;
use crate::errors::export::ExportError;

/// Exporte les fichiers de logs de l'application vers un chemin donné par l'utilisateur.
///
/// Le dossier source est résolu par Tauri (`app_log_dir`), donc toujours
/// aligné sur l'identifiant réel de l'application (`fr.sonar.ssf`) et sur
/// l'emplacement où `tauri-plugin-log` écrit — l'ancien chemin codé en dur
/// (`fr.sonar.app`) rendait l'export systématiquement introuvable.
///
/// # Paramètres
///
/// - `destination`: Chemin absolu ou relatif fourni par le frontend où les logs doivent être copiés.
///
/// # Retour
///
/// - `Ok(String)` : Message de succès si tous les fichiers ont été exportés correctement.
/// - `Err(ExportError)` : Erreur en cas d’échec (log introuvable, erreur de lecture ou écriture, etc.).
///
/// # Erreurs possibles
///
/// - [`ExportError::LogNotFound`] : Le dossier source de logs n’existe pas.
/// - [`ExportError::Io`] : Une erreur d’entrée/sortie s’est produite lors de la copie ou de la lecture des fichiers.
///
/// # Exemple d’usage (frontend)
///
/// ```ts
/// const path = await save({ title: "Choisissez où sauvegarder les logs" });
/// if (path) {
///   const result = await invoke("export_logs", { destination: path });
///   console.log(result);
/// }
/// ```
///
/// # Exemple d’enchaînement (logique utilisateur)
///
/// ```mermaid
/// sequenceDiagram
///     participant Utilisateur
///     participant Frontend (Vue.js)
///     participant Tauri Backend (Rust)
///     participant FS (Système de fichiers)
///
///     Utilisateur->>Frontend (Vue.js): Clique sur "Exporter les logs"
///     Frontend (Vue.js)->>Frontend (Vue.js): Ouvre une boîte de dialogue (save)
///     Frontend (Vue.js)->>Utilisateur: Demande de choisir un fichier `.log`
///     Utilisateur-->>Frontend (Vue.js): Sélectionne le chemin de destination
///
///     Frontend (Vue.js)->>Tauri Backend (Rust): invoke("export_logs", { destination })
///     Tauri Backend (Rust)->>Tauri Backend (Rust): Détecte le chemin de logs selon l'OS
///     Tauri Backend (Rust)->>FS: Vérifie si le dossier de logs existe
///     alt Le dossier n'existe pas
///         Tauri Backend (Rust)-->>Frontend (Vue.js): Retourne une erreur LogNotFound
///     else Le dossier existe
///         Tauri Backend (Rust)->>FS: Crée le dossier de destination (si nécessaire)
///         Tauri Backend (Rust)->>FS: Copie tous les fichiers de log
///         Tauri Backend (Rust)-->>Frontend (Vue.js): Retourne "Logs exportés avec succès"
///         Frontend (Vue.js)->>Utilisateur: Affiche confirmation de sauvegarde
///     end
/// ```
///
/// # Sécurité
///
/// Cette commande n’écrase pas les fichiers existants si leurs noms sont différents,
/// mais elle ne fait pas de vérification de doublons ou d'intégrité.
/// Il est recommandé de filtrer les fichiers côté Rust si nécessaire.
#[command(async)]
pub fn export_logs(app: AppHandle, destination: String) -> Result<String, CaptureStateError> {
    let log_dir: PathBuf = app
        .path()
        .app_log_dir()
        .map_err(|_| CaptureStateError::Export(ExportError::LogNotFound))?;

    if !log_dir.exists() {
        return Err(CaptureStateError::Export(ExportError::LogNotFound));
    }

    let destination = PathBuf::from(destination);

    if !destination.exists() {
        fs::create_dir_all(&destination)?;
    }

    for entry in fs::read_dir(&log_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        if src_path.is_file() {
            // Un fichier issu de read_dir a toujours un nom ; sinon on l'ignore
            // plutôt que de paniquer.
            let Some(file_name) = src_path.file_name() else {
                continue;
            };
            let dest_path = destination.join(file_name);
            fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok("Logs exportés avec succès".to_string())
}
