//! Commande d'export des fichiers de logs de l'application.

use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, command};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::errors::CaptureStateError;
use crate::errors::export::ExportError;

/// Exporte les fichiers de logs de l'application vers une archive ZIP dont le
/// chemin est donné par l'utilisateur.
///
/// Le dossier source est résolu par Tauri (`app_log_dir`), donc toujours
/// aligné sur l'identifiant réel de l'application (`fr.sonar.ssf`) et sur
/// l'emplacement où `tauri-plugin-log` écrit — l'ancien chemin codé en dur
/// (`fr.sonar.app`) rendait l'export systématiquement introuvable.
///
/// `destination` désigne le fichier ZIP à créer, pas un dossier : l'ancienne
/// implémentation traitait systématiquement `destination` comme un
/// répertoire (`create_dir_all` + copie fichier par fichier), si bien que
/// choisir `sonar.log` dans le dialogue de sauvegarde créait un **dossier**
/// `sonar.log/` au lieu du fichier annoncé.
///
/// # Paramètres
///
/// - `destination`: Chemin du fichier ZIP à créer, fourni par le frontend.
///
/// # Retour
///
/// - `Ok(String)` : Message de succès si l'archive a été créée correctement.
/// - `Err(ExportError)` : Erreur en cas d’échec (log introuvable, erreur de
///   lecture/écriture, erreur d'archivage, etc.).
///
/// # Erreurs possibles
///
/// - [`ExportError::LogNotFound`] : Le dossier source de logs n’existe pas.
/// - [`ExportError::Io`] : Une erreur d’entrée/sortie s’est produite lors de la copie ou de la lecture des fichiers.
/// - [`ExportError::Zip`] : Une erreur s'est produite lors de l'écriture de l'archive ZIP.
///
/// # Exemple d’usage (frontend)
///
/// ```ts
/// const path = await save({ title: "Choisissez où sauvegarder les logs", defaultPath: "sonar-support.zip" });
/// if (path) {
///   const result = await invoke("export_logs", { destination: path });
///   console.log(result);
/// }
/// ```
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
    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    // Écriture dans un fichier temporaire puis renommage atomique, pour ne
    // jamais laisser une archive partielle au chemin final en cas d'échec.
    let tmp_destination = destination.with_extension("part");
    write_logs_zip(&log_dir, &tmp_destination).inspect_err(|_| {
        let _ = fs::remove_file(&tmp_destination);
    })?;
    fs::rename(&tmp_destination, &destination)?;

    Ok("Logs exportés avec succès".to_string())
}

fn write_logs_zip(log_dir: &Path, destination: &Path) -> Result<(), CaptureStateError> {
    let file = File::create(destination)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        if !src_path.is_file() {
            continue;
        }
        // Un fichier issu de read_dir a toujours un nom ; sinon on l'ignore
        // plutôt que de paniquer.
        let Some(file_name) = src_path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        zip.start_file(file_name, options)
            .map_err(ExportError::Zip)?;
        let mut src_file = File::open(&src_path)?;
        copy(&mut src_file, &mut zip)?;
    }

    zip.finish().map_err(ExportError::Zip)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_temp_dir(label: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "sonar_export_logs_test_{label}_{}_{n}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn write_logs_zip_produces_a_single_archive_file_not_a_directory() {
        let log_dir = unique_temp_dir("src");
        fs::write(log_dir.join("sonar.log"), b"hello").unwrap();
        fs::write(log_dir.join("sonar.log.1"), b"older").unwrap();
        // Un sous-dossier ne doit pas être archivé : seuls les fichiers le sont.
        fs::create_dir_all(log_dir.join("subdir")).unwrap();

        let out_dir = unique_temp_dir("out");
        let destination = out_dir.join("sonar-logs.zip");

        write_logs_zip(&log_dir, &destination).unwrap();

        assert!(
            destination.is_file(),
            "la destination doit être un fichier, pas un dossier"
        );

        let file = File::open(&destination).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut names: Vec<_> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["sonar.log", "sonar.log.1"]);

        fs::remove_dir_all(&log_dir).ok();
        fs::remove_dir_all(&out_dir).ok();
    }
}
