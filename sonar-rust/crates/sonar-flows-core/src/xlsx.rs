//! Lecture des matrices de flux au format XLSX (même schéma SFMS que le CSV).
//!
//! Le classeur est converti en CSV en mémoire puis repassé au désérialiseur et
//! à la validation de [`crate::csv`] : CSV et XLSX appliquent ainsi exactement
//! le même schéma, la même validation stricte et les mêmes numéros de ligne.

use std::path::Path;

use calamine::{Data, DataType, Range, Reader, Xlsx, open_workbook};
use packet_parser::LinkType;

use crate::matrix::FlowMatrixRow;
use crate::sfms;
use crate::{Result, SonarCoreError};

/// Vrai si le chemin porte l'extension `.xlsx` (insensible à la casse).
pub fn is_xlsx(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"))
}

fn invalid(path: &Path, message: impl Into<String>) -> SonarCoreError {
    SonarCoreError::InvalidCsv {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

/// Convertit une cellule Excel en texte CSV. Les dates réellement typées par
/// Excel sont normalisées dans le format SFMS accepté par `last_seen` ; les
/// autres cellules gardent leur représentation textuelle.
fn cell_text(cell: &Data, header: &str) -> String {
    if header == "last_seen"
        && let Some(date) = cell.as_datetime()
    {
        return date.format("%Y-%m-%d %H:%M:%S%.6f UTC").to_string();
    }
    cell.to_string()
}

/// Charge la première feuille du classeur.
fn first_sheet(path: &Path) -> Result<Range<Data>> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|error| invalid(path, format!("ouverture XLSX impossible: {error}")))?;
    workbook
        .worksheet_range_at(0)
        .ok_or_else(|| invalid(path, "le classeur XLSX ne contient aucune feuille"))?
        .map_err(|error| invalid(path, format!("lecture XLSX impossible: {error}")))
}

/// Type de liaison annoncé par le préambule `#SFMS` du classeur (Ethernet
/// implicite pour un export antérieur au préambule).
fn range_link_type(path: &Path, range: &Range<Data>) -> Result<(LinkType, usize)> {
    let first = range
        .rows()
        .find(|row| row.iter().any(|cell| !cell.is_empty()))
        .ok_or_else(|| invalid(path, "la première feuille XLSX est vide"))?;
    let first_cell = first.first().map(ToString::to_string).unwrap_or_default();
    match sfms::parse_preamble(&first_cell).map_err(|message| invalid(path, message))? {
        Some(preamble) => Ok((preamble.link_type.unwrap_or(LinkType::ETHERNET), 1)),
        None => Ok((LinkType::ETHERNET, 0)),
    }
}

/// Type de liaison d'un classeur, sans désérialiser ses lignes.
pub fn matrix_file_link_type(path: &Path) -> Result<LinkType> {
    let range = first_sheet(path)?;
    let (link_type, _) = range_link_type(path, &range)?;
    sfms::ensure_reimportable(path, link_type)?;
    Ok(link_type)
}

/// Lit toutes les lignes d'une matrice XLSX, entièrement validée : la première
/// ligne invalide interrompt la lecture avec son numéro de ligne (numérotation
/// de la feuille, préambule compris).
pub fn read_matrix_rows(path: &Path) -> Result<Vec<FlowMatrixRow>> {
    let range = first_sheet(path)?;
    let (link_type, header_index) = range_link_type(path, &range)?;
    sfms::ensure_reimportable(path, link_type)?;

    let non_empty_rows: Vec<&[Data]> = range
        .rows()
        .filter(|row| row.iter().any(|cell| !cell.is_empty()))
        .collect();
    let header_row = non_empty_rows
        .get(header_index)
        .ok_or_else(|| invalid(path, "en-tête de matrice XLSX absent"))?;
    let headers: Vec<String> = header_row
        .iter()
        .map(|cell| cell.to_string().trim().to_string())
        .collect();

    let mut csv_data = Vec::new();
    {
        let mut writer = csv::WriterBuilder::new().from_writer(&mut csv_data);
        writer
            .write_record(&headers)
            .map_err(|error| invalid(path, format!("conversion XLSX impossible: {error}")))?;
        for row in non_empty_rows.iter().skip(header_index + 1) {
            let record = headers.iter().enumerate().map(|(column, header)| {
                row.get(column)
                    .map(|cell| cell_text(cell, header))
                    .unwrap_or_default()
            });
            writer
                .write_record(record)
                .map_err(|error| invalid(path, format!("conversion XLSX impossible: {error}")))?;
        }
        writer
            .flush()
            .map_err(|error| invalid(path, format!("conversion XLSX impossible: {error}")))?;
    }

    // Numéro de ligne réel dans la feuille : la première ligne de données suit
    // l'en-tête, lui-même précédé du préambule s'il est présent.
    let first_data_line = header_index + 2;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_slice());
    let mut rows = Vec::new();
    for (index, result) in reader.deserialize::<FlowMatrixRow>().enumerate() {
        let row = result.map_err(|error| {
            invalid(
                path,
                format!("ligne {} invalide: {error}", index + first_data_line),
            )
        })?;
        row.validate(link_type).map_err(|error| {
            invalid(
                path,
                format!("ligne {} invalide: {error}", index + first_data_line),
            )
        })?;
        rows.push(row);
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../src-tauri/test_files")
            .join(name)
    }

    /// Le test central : XLSX et CSV décrivent le MÊME relevé. La fixture
    /// `.xlsx` est générée depuis le `.csv` par
    /// `script/matrix/generate-xlsx-fixture.py` — si les deux lectures
    /// divergeaient, un opérateur obtiendrait deux matrices différentes pour
    /// un même contenu selon le format qu'il a sous la main.
    #[test]
    fn xlsx_and_csv_yield_identical_rows() {
        let from_csv =
            crate::csv::read_matrix_rows(&fixture("20260703_NP_Matrice.csv")).expect("lecture CSV");
        let from_xlsx =
            read_matrix_rows(&fixture("20260703_NP_Matrice.xlsx")).expect("lecture XLSX");

        assert!(!from_csv.is_empty(), "la fixture porte des lignes");
        assert_eq!(
            from_xlsx.len(),
            from_csv.len(),
            "même nombre de lignes dans les deux formats"
        );
        assert_eq!(
            from_xlsx, from_csv,
            "chaque champ doit être identique entre XLSX et CSV"
        );
    }

    /// L'aiguillage de `crate::csv::read_matrix_rows` reconnaît le classeur :
    /// un appelant n'a pas à savoir quel format il tient.
    #[test]
    fn the_csv_entry_point_dispatches_to_xlsx() {
        let rows = crate::csv::read_matrix_rows(&fixture("20260703_NP_Matrice.xlsx"))
            .expect("aiguillage vers le lecteur XLSX");
        assert!(!rows.is_empty());
    }

    #[test]
    fn is_xlsx_ignores_case_and_other_extensions() {
        assert!(is_xlsx(Path::new("releve.xlsx")));
        assert!(is_xlsx(Path::new("RELEVE.XLSX")));
        assert!(!is_xlsx(Path::new("releve.csv")));
        assert!(!is_xlsx(Path::new("releve")));
    }

    /// Le type de liaison se lit sans désérialiser les lignes, et il vient du
    /// préambule `#SFMS` comme pour le CSV.
    #[test]
    fn link_type_comes_from_the_preamble() {
        let from_xlsx =
            matrix_file_link_type(&fixture("20260703_NP_Matrice.xlsx")).expect("DLT du classeur");
        let from_csv =
            sfms::matrix_file_link_type(&fixture("20260703_NP_Matrice.csv")).expect("DLT du CSV");
        assert_eq!(from_xlsx, from_csv);
    }

    /// Un fichier qui n'est pas un classeur échoue proprement, avec son
    /// chemin dans le message — jamais un panic ni une matrice vide silencieuse.
    #[test]
    fn a_non_workbook_file_fails_with_its_path() {
        let error = read_matrix_rows(&fixture("20260703_NP_Matrice.csv"))
            .expect_err("un CSV n'est pas un classeur XLSX");
        assert!(
            error.to_string().contains("20260703_NP_Matrice.csv"),
            "le message doit nommer le fichier, obtenu : {error}"
        );
    }
}
