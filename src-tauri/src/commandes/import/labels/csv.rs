//! Lecture et parsing des fichiers de labels CSV : lignes brutes, détection
//! d'en-tête, normalisation des champs mac/ip/label vers les formes produites
//! par le parser de paquets.

use std::{io::ErrorKind, net::IpAddr};

use crate::{
    errors::{CaptureStateError, label::LabelError},
    setup::labels::{clean_csv_field, parse_label_fields},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LabelRow {
    pub(super) line: usize,
    pub(super) raw: String,
    pub(super) mac: String,
    pub(super) ip: String,
    pub(super) label: String,
}

impl LabelRow {
    pub(super) fn into_tuple(self) -> (String, String, String) {
        (self.mac, self.ip, self.label)
    }
}

fn read_label_file(csv_path: &str) -> Result<String, CaptureStateError> {
    match std::fs::read_to_string(csv_path) {
        Ok(csv_data) => Ok(csv_data),
        // Un chemin inexistant était lu comme un fichier vide : l'import
        // vidait alors le store de labels sans un mot. Erreur explicite (#153).
        Err(error) if error.kind() == ErrorKind::NotFound => Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("fichier de labels introuvable : {csv_path}"),
        )
        .into()),
        Err(error) => Err(error.into()),
    }
}

fn label_record_line(record: &csv::StringRecord, fallback: usize) -> usize {
    record
        .position()
        .map(|position| position.line() as usize)
        .unwrap_or(fallback)
}

fn label_error_line(error: &csv::Error, fallback: usize) -> usize {
    error
        .position()
        .map(|position| position.line() as usize)
        .unwrap_or(fallback)
}

fn label_record_to_display(record: &csv::StringRecord) -> String {
    record.iter().collect::<Vec<_>>().join(",")
}

/// Vrai si la ligne est l'en-tête d'une matrice de flux exportée
/// (`FlowMatrixRow`) : l'utilisateur s'est trompé de fichier.
fn is_matrix_export_header(record: &csv::StringRecord) -> bool {
    let mut fields = record.iter().map(str::trim);
    fields.next() == Some("mac_source") && fields.next() == Some("mac_destination")
}

fn read_label_records(
    csv_path: &str,
) -> Result<Vec<(usize, csv::StringRecord)>, CaptureStateError> {
    let file = read_label_file(csv_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file.as_bytes());

    let mut records = Vec::new();
    let mut invalid_lines = Vec::new();

    for (index, result) in rdr.records().enumerate() {
        let fallback_line = index + 1;
        match result {
            Ok(record) => {
                let line = label_record_line(&record, fallback_line);
                if record.iter().all(|field| clean_csv_field(field).is_empty()) {
                    continue;
                }
                // Mauvais type de fichier : une matrice de flux exportée
                // commence par son en-tête `mac_source,mac_destination,…`.
                // Sans cette détection, chacune de ses lignes devenait une
                // « erreur MAC/IP » et l'utilisateur recevait des centaines
                // d'erreurs au lieu d'un diagnostic clair.
                if index == 0 && is_matrix_export_header(&record) {
                    return Err(LabelError::InvalidRowsFormat {
                        invalid_lines: vec![(
                            line,
                            "ce fichier est une matrice de flux exportée (colonnes \
                             mac_source, mac_destination…) : importez-le via « Ouvrir \
                             un fichier CSV ». Un fichier de labels contient mac, ip, label."
                                .to_string(),
                        )],
                    }
                    .into());
                }
                if record.len() < 3 {
                    invalid_lines.push((line, label_record_to_display(&record)));
                } else {
                    records.push((line, record));
                }
            }
            Err(error) => {
                invalid_lines.push((label_error_line(&error, fallback_line), error.to_string()))
            }
        }
    }

    if invalid_lines.is_empty() {
        Ok(records)
    } else {
        Err(LabelError::InvalidRowsFormat { invalid_lines }.into())
    }
}

pub(super) fn read_label_rows(csv_path: &str) -> Result<Vec<LabelRow>, CaptureStateError> {
    Ok(read_label_records(csv_path)?
        .into_iter()
        .filter_map(|(line, record)| {
            let raw = label_record_to_display(&record);
            parse_label_fields(record.iter()).map(|(mac, ip, label)| LabelRow {
                line,
                raw,
                // Normalisation vers les formes produites par le parser de
                // paquets : MAC en minuscules, IP canonique (IPv6 compressée).
                // Sans elle, un label « AA:BB:… » ou « ::0001 » ne
                // correspondait jamais, sans erreur visible (#153). La forme
                // brute reste dans `raw` pour les messages d'erreur.
                mac: mac.to_ascii_lowercase(),
                ip: ip
                    .parse::<IpAddr>()
                    .map(|addr| addr.to_string())
                    .unwrap_or(ip),
                // Retire le préfixe anti-injection de formule posé à
                // l'export (#148) : aller-retour sans altération.
                label: crate::state::flow_matrix::unescape_formula_cell(&label),
            })
        })
        .collect())
}

// Une ligne d'en-tête ne peut être que la première du fichier et n'est
// reconnue que si elle utilise une paire de colonnes explicitement supportée
// (insensible à la casse). L'ancienne heuristique « champs non parseables »
// avalait silencieusement une première ligne de données invalide (#153).
pub(super) fn is_header_row(row: &LabelRow) -> bool {
    (row.mac.eq_ignore_ascii_case("mac") && row.ip.eq_ignore_ascii_case("ip"))
        || (row.mac.eq_ignore_ascii_case("adresse_mac")
            && row.ip.eq_ignore_ascii_case("adresse_ip"))
}

#[cfg(test)]
pub(super) fn verif_label_rows_format(file: String) -> Result<(), CaptureStateError> {
    read_label_records(&file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::super::test_support::TempDir;
    use super::*;
    use std::fs;

    /// Chemins avec espaces, Unicode et caractères spéciaux shell (#88) :
    /// l'import de labels lit le fichier tel quel (`std::fs`, jamais de
    /// shell ni de concaténation) et les labels Unicode traversent intacts.
    #[test]
    fn label_paths_with_spaces_unicode_and_special_characters_import() {
        let dir = TempDir::new("sonar_test_label_weird_names");
        for name in [
            "labels de l'été (v2).csv",
            "étiquettes 中文 📡.csv",
            "site \"principal\" `usine`.csv",
        ] {
            let path = dir.path().join(name);
            fs::write(
                &path,
                "aa:bb:cc:dd:ee:ff,192.168.1.10,Automate n°1 (atelier été)\n",
            )
            .unwrap();

            let rows = read_label_rows(path.to_str().unwrap())
                .unwrap_or_else(|e| panic!("import de {name}: {e:?}"));
            assert_eq!(rows.len(), 1, "{name}");
            assert_eq!(rows[0].label, "Automate n°1 (atelier été)", "{name}");
        }
    }

    /// Une matrice de flux exportée importée comme fichier de labels doit
    /// produire UNE erreur claire (« mauvais type de fichier »), pas une
    /// erreur MAC/IP par ligne du fichier.
    #[test]
    fn matrix_export_imported_as_labels_gives_one_clear_error() {
        let dir = TempDir::new("sonar_test_label_wrong_kind");
        let path = dir.path().join("matrice.csv");
        fs::write(
            &path,
            "mac_source,mac_destination,vlan_id,protocol_data_link,ip_source\n\
             aa:bb:cc:dd:ee:ff,ff:ff:ff:ff:ff:ff,,ARP,172.24.8.29\n",
        )
        .unwrap();

        let err = read_label_rows(path.to_str().unwrap()).unwrap_err();
        let CaptureStateError::Label(LabelError::InvalidRowsFormat { invalid_lines }) = err else {
            panic!("erreur InvalidRowsFormat attendue, reçu {err:?}");
        };
        assert_eq!(
            invalid_lines.len(),
            1,
            "une seule erreur, pas une par ligne"
        );
        assert!(invalid_lines[0].1.contains("matrice de flux"));
    }

    #[test]
    fn valid_csv_format_returns_ok() {
        let dir = TempDir::new("sonar_test_valid_csv_format");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    /// Un chemin inexistant est une erreur explicite : il était lu comme un
    /// fichier vide et l'import vidait le store de labels (#153).
    #[test]
    fn missing_file_is_an_explicit_error() {
        let result = read_label_rows("/chemin/qui/n/existe/pas/labels.csv");
        match result.unwrap_err() {
            CaptureStateError::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
                assert!(e.to_string().contains("introuvable"), "{e}");
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    /// L'en-tête n'est reconnu que s'il dit littéralement `mac,ip` : une
    /// première ligne de données invalide n'est plus avalée en silence (#153).
    #[test]
    fn header_detection_is_strict() {
        let header = LabelRow {
            line: 1,
            raw: "MAC,IP,Label".into(),
            mac: "MAC".into(),
            ip: "IP".into(),
            label: "Label".into(),
        };
        assert!(is_header_row(&header));

        let invalid_data = LabelRow {
            line: 1,
            raw: "pas-une-mac,pas-une-ip,label".into(),
            mac: "pas-une-mac".into(),
            ip: "pas-une-ip".into(),
            label: "label".into(),
        };
        assert!(
            !is_header_row(&invalid_data),
            "une ligne invalide n'est pas un en-tête"
        );
    }

    /// MAC/IP non canoniques (casse, forme IPv6 étendue) sont normalisées à
    /// l'import vers les formes du parser : le label s'applique (#153).
    #[test]
    fn labels_are_normalized_to_parser_forms() {
        let dir = TempDir::new("sonar_test_label_normalization");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "AA:BB:CC:DD:EE:FF,192.168.1.1,pc-majuscules\n00:11:22:33:44:55,2001:0db8:0000:0000:0000:0000:0000:0001,ipv6-étendue\n",
        )
        .unwrap();

        let rows = read_label_rows(file_path.to_str().unwrap()).unwrap();

        assert_eq!(rows[0].mac, "aa:bb:cc:dd:ee:ff", "MAC en minuscules");
        assert_eq!(rows[1].ip, "2001:db8::1", "IPv6 compressée canonique");
    }

    #[test]
    fn missing_column_returns_invalid_file_format_error() {
        let dir = TempDir::new("sonar_test_missing_column");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "192.168.1.1,mon-pc\n").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::InvalidRowsFormat { invalid_lines }) => {
                assert_eq!(invalid_lines, vec![(1, "192.168.1.1,mon-pc".to_string())]);
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn extra_column_is_merged_into_label() {
        let dir = TempDir::new("sonar_test_extra_column");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc,réseau-2\n",
        )
        .unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());
        assert!(result.is_ok());

        let rows = read_label_rows(file_path.to_str().unwrap()).unwrap();
        assert_eq!(
            rows.into_iter()
                .map(LabelRow::into_tuple)
                .collect::<Vec<_>>(),
            vec![(
                "aa:bb:cc:dd:ee:ff".to_string(),
                "192.168.1.1".to_string(),
                "mon-pc réseau-2".to_string()
            )]
        );
    }

    #[test]
    fn empty_file_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }
}
