//! Lecture et fusion de matrices de flux au format CSV (SFMS).
//!
//! Chemin inverse de [`crate::matrix::FlowMatrix::export_to_csv`] : les lignes
//! lues sont refusionnées tunnel par tunnel, la colonne `origin` trace le ou
//! les fichiers de provenance de chaque flux, et les labels portés par les
//! fichiers sont réappliqués à la matrice reconstruite.

use std::path::{Path, PathBuf};

use crate::matrix::{FlowMatrix, FlowMatrixRow, parse_origin_list};
use crate::{Result, SonarCoreError, validate_batch_paths};

/// Lit toutes les lignes d'un fichier de matrice CSV. Le fichier est
/// entièrement validé : la première ligne invalide interrompt la lecture avec
/// son numéro de ligne.
pub fn read_matrix_rows(csv_path: &Path) -> Result<Vec<FlowMatrixRow>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(csv_path)
        .map_err(|e| SonarCoreError::InvalidCsv {
            path: csv_path.to_path_buf(),
            message: format!("ouverture impossible: {e}"),
        })?;

    let mut rows = Vec::new();
    for (i, result) in rdr.deserialize::<FlowMatrixRow>().enumerate() {
        let row = result.map_err(|e| SonarCoreError::InvalidCsv {
            path: csv_path.to_path_buf(),
            message: format!("ligne {} invalide: {e}", i + 2),
        })?;
        rows.push(row);
    }
    Ok(rows)
}

/// Nom de fichier (sans le chemin) utilisé comme origine par défaut d'une
/// ligne importée. Repli sur le chemin complet si le nom ne peut être extrait.
pub fn origin_name_from_path(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Lit les lignes de plusieurs fichiers de matrice. Une ligne sans provenance
/// héritée reçoit le nom du fichier importé ; une ligne qui portait déjà une
/// origine (matrice déjà fusionnée puis réexportée) la conserve telle quelle.
pub fn read_matrix_rows_from_files(paths: &[PathBuf]) -> Result<Vec<FlowMatrixRow>> {
    if paths.is_empty() {
        return Err(SonarCoreError::MissingInput);
    }

    let mut rows = Vec::new();
    for path in paths {
        let origin = origin_name_from_path(path);
        for mut row in read_matrix_rows(path)? {
            if row.origin.trim().is_empty() {
                row.origin = origin.clone();
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

/// Reconstruit une matrice depuis des lignes de CSV : les doublons éventuels
/// sont fusionnés tunnel par tunnel (comptabilité père/fils préservée), les
/// origines s'accumulent par flux et les labels portés par les fichiers sont
/// réappliqués.
pub fn matrix_from_rows(rows: &[FlowMatrixRow]) -> FlowMatrix {
    let mut matrix = FlowMatrix::new();

    // Labels portés par les fichiers d'abord : à clé égale, le dernier inséré
    // gagne, comme dans l'application desktop.
    for row in rows {
        if let Some(label) = row.label_source.as_ref().filter(|l| !l.is_empty()) {
            matrix.add_label(row.mac_source.clone(), row.ip_source.clone(), label.clone());
        }
        if let Some(label) = row.label_destination.as_ref().filter(|l| !l.is_empty()) {
            matrix.add_label(
                row.mac_destination.clone(),
                row.ip_destination.clone(),
                label.clone(),
            );
        }
    }

    for row in rows {
        let (flow, tunnel_rows) = row.to_flow_and_rows();

        for (encap_id, stats) in tunnel_rows {
            matrix.merge_row(flow.clone(), encap_id, stats);
        }

        matrix.add_flow_origins(&flow, parse_origin_list(&row.origin));
    }

    matrix
}

/// Fusionne plusieurs fichiers de matrice CSV en une matrice unique.
pub fn merge_matrix_files(inputs: &[PathBuf]) -> Result<FlowMatrix> {
    let rows = read_matrix_rows_from_files(inputs)?;
    Ok(matrix_from_rows(&rows))
}

/// Fusionne plusieurs fichiers de matrice CSV et exporte le résultat.
/// Retourne le nombre de lignes (flux distincts) de la matrice fusionnée.
pub fn merge_matrix_files_to_csv(inputs: &[PathBuf], output: &Path) -> Result<usize> {
    validate_batch_paths(inputs, output)?;
    let matrix = merge_matrix_files(inputs)?;
    matrix.export_to_csv(output.to_string_lossy().into_owned())?;
    Ok(matrix.row_count())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../src-tauri/test_files")
            .join(name)
    }

    #[test]
    fn read_matrix_rows_parses_reference_fixture() {
        let rows = read_matrix_rows(&fixture("20260703_NP_Matrice.csv")).expect("lecture fixture");
        assert!(!rows.is_empty(), "la fixture contient des lignes");
    }

    #[test]
    fn merge_two_identical_files_doubles_counters_and_tracks_origins() {
        let dir = std::env::temp_dir().join("sonar_core_merge_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let file_a = dir.join("site-a.csv");
        let file_b = dir.join("site-b.csv");
        std::fs::copy(fixture("20260703_NP_Matrice.csv"), &file_a).expect("copie a");
        std::fs::copy(fixture("20260703_NP_Matrice.csv"), &file_b).expect("copie b");

        let single = merge_matrix_files(std::slice::from_ref(&file_a)).expect("fusion simple");
        let merged = merge_matrix_files(&[file_a, file_b]).expect("fusion double");

        assert_eq!(
            merged.row_count(),
            single.row_count(),
            "mêmes flux -> même nombre de lignes"
        );

        let single_count: u64 = single.to_flat_vec().iter().map(|r| r.count).sum();
        let merged_count: u64 = merged.to_flat_vec().iter().map(|r| r.count).sum();
        assert_eq!(merged_count, single_count * 2, "compteurs cumulés");

        assert!(
            merged
                .to_flat_vec()
                .iter()
                .all(|r| r.origin == "site-a.csv|site-b.csv"),
            "chaque ligne trace ses deux fichiers d'origine"
        );
    }

    #[test]
    fn read_matrix_rows_reports_line_number_on_invalid_file() {
        let dir = std::env::temp_dir().join("sonar_core_invalid_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let bad = dir.join("bad.csv");
        std::fs::write(
            &bad,
            "mac_source,mac_destination\nnot,enough,columns,at,all\n",
        )
        .expect("écriture");

        let err = read_matrix_rows(&bad).expect_err("fichier invalide");
        assert!(
            err.to_string().contains("ligne 2"),
            "le numéro de ligne doit être signalé: {err}"
        );
    }
}
