//! Lecture et fusion de matrices de flux au format CSV (SFMS).
//!
//! Chemin inverse de [`crate::matrix::FlowMatrix::export_to_csv`] : les lignes
//! lues sont refusionnées tunnel par tunnel, la colonne `origin` trace le ou
//! les fichiers de provenance de chaque flux, et les labels portés par les
//! fichiers sont réappliqués à la matrice reconstruite.

use std::path::{Path, PathBuf};

use packet_parser::LinkType;

use crate::matrix::{FlowMatrix, FlowMatrixRow, parse_origin_list, unescape_formula_cell};
use crate::sfms;
use crate::{Result, SonarCoreError, validate_batch_paths};

/// Lit toutes les lignes d'un fichier de matrice CSV. Le fichier est
/// entièrement validé : la première ligne invalide interrompt la lecture avec
/// son numéro de ligne. Le préambule `#SFMS` éventuel est contrôlé : un DLT
/// sans projection d'identité SFMS définie est refusé plutôt que reconstruit
/// sous un autre type de liaison.
pub fn read_matrix_rows(csv_path: &Path) -> Result<Vec<FlowMatrixRow>> {
    let preamble = sfms::read_preamble(csv_path)?;
    let link_type = preamble
        .as_ref()
        .and_then(|p| p.link_type)
        .unwrap_or(LinkType::ETHERNET);
    if !matches!(
        link_type,
        LinkType::ETHERNET | LinkType::RAW | LinkType::LINUX_SLL | LinkType::LINUX_SLL2
    ) {
        return Err(SonarCoreError::UnreimportableLinkType {
            path: csv_path.to_path_buf(),
            label: sfms::link_type_name(link_type),
        });
    }
    // Numéro de ligne réel dans le fichier : la première ligne de données
    // suit l'en-tête, lui-même précédé du préambule s'il est présent.
    let first_data_line = if preamble.is_some() { 3 } else { 2 };

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        // Saute la ligne de préambule `#SFMS` (et tout commentaire `#`).
        .comment(Some(b'#'))
        .from_path(csv_path)
        .map_err(|e| SonarCoreError::InvalidCsv {
            path: csv_path.to_path_buf(),
            message: format!("ouverture impossible: {e}"),
        })?;

    let mut rows = Vec::new();
    for (i, result) in rdr.deserialize::<FlowMatrixRow>().enumerate() {
        let row = result.map_err(|e| SonarCoreError::InvalidCsv {
            path: csv_path.to_path_buf(),
            message: format!("ligne {} invalide: {e}", i + first_data_line),
        })?;
        // Validation stricte (#148) : une IP ou une date illisible est
        // rejetée avec son numéro de ligne, pas dégradée silencieusement.
        row.validate(link_type)
            .map_err(|e| SonarCoreError::InvalidCsv {
                path: csv_path.to_path_buf(),
                message: format!("ligne {} invalide: {e}", i + first_data_line),
            })?;
        rows.push(row);
    }
    Ok(rows)
}

/// Type de liaison commun à des fichiers de matrice (préambule `#SFMS`,
/// Ethernet implicite pour un export antérieur) : la fusion de relevés de DLT
/// différents est refusée explicitement (arbitrage du 14/07/2026).
pub fn common_matrix_link_type(paths: &[PathBuf]) -> Result<LinkType> {
    let mut common: Option<(LinkType, &PathBuf)> = None;
    for path in paths {
        let link_type = sfms::matrix_file_link_type(path)?;
        match common {
            None => common = Some((link_type, path)),
            Some((expected, _)) if expected == link_type => {}
            Some((expected, _)) => {
                return Err(SonarCoreError::MixedLinkTypes {
                    path: path.clone(),
                    found: sfms::link_type_name(link_type),
                    expected: sfms::link_type_name(expected),
                });
            }
        }
    }
    common
        .map(|(link_type, _)| link_type)
        .ok_or(SonarCoreError::MissingInput)
}

/// Nom de fichier (sans le chemin) utilisé comme origine par défaut d'une
/// ligne importée. Repli sur le chemin complet si le nom ne peut être extrait.
pub fn origin_name_from_path(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Lit les lignes de chaque fichier de matrice, groupées par fichier (pour la
/// comptabilité par fichier des appelants). Une ligne sans provenance héritée
/// reçoit le nom du fichier importé ; une ligne qui portait déjà une origine
/// (matrice déjà fusionnée puis réexportée) la conserve telle quelle.
pub fn read_matrix_rows_per_file(paths: &[PathBuf]) -> Result<Vec<(PathBuf, Vec<FlowMatrixRow>)>> {
    if paths.is_empty() {
        return Err(SonarCoreError::MissingInput);
    }

    let mut files = Vec::new();
    for path in paths {
        let origin = origin_name_from_path(path);
        let mut rows = read_matrix_rows(path)?;
        for row in &mut rows {
            if row.origin.trim().is_empty() {
                row.origin = origin.clone();
            }
        }
        files.push((path.clone(), rows));
    }
    Ok(files)
}

/// Lit les lignes de plusieurs fichiers de matrice (version aplatie de
/// [`read_matrix_rows_per_file`]).
pub fn read_matrix_rows_from_files(paths: &[PathBuf]) -> Result<Vec<FlowMatrixRow>> {
    Ok(read_matrix_rows_per_file(paths)?
        .into_iter()
        .flat_map(|(_, rows)| rows)
        .collect())
}

/// Applique à une matrice les labels portés par des lignes de CSV : à clé
/// égale, le dernier inséré gagne (et écrase un label déjà présent dans la
/// matrice). Le préfixe anti-injection de formule posé à l'export est retiré
/// (aller-retour sans altération).
pub fn apply_row_labels(matrix: &mut FlowMatrix, rows: &[FlowMatrixRow]) {
    for row in rows {
        if let Some(label) = row.label_source.as_ref().filter(|l| !l.is_empty()) {
            matrix.add_label(
                row.mac_source.clone(),
                row.ip_source.clone(),
                unescape_formula_cell(label),
            );
        }
        if let Some(label) = row.label_destination.as_ref().filter(|l| !l.is_empty()) {
            matrix.add_label(
                row.mac_destination.clone(),
                row.ip_destination.clone(),
                unescape_formula_cell(label),
            );
        }
    }
}

/// Fusionne des lignes de CSV dans une matrice : les doublons éventuels sont
/// fusionnés tunnel par tunnel (comptabilité père/fils préservée) et les
/// origines s'accumulent par flux.
pub fn merge_rows(
    matrix: &mut FlowMatrix,
    rows: &[FlowMatrixRow],
    link_type: LinkType,
) -> Result<()> {
    for row in rows {
        row.validate(link_type)
            .map_err(SonarCoreError::InvalidMatrixRow)?;
        let (flow, tunnel_rows) = row
            .to_flow_and_rows(link_type)
            .map_err(SonarCoreError::InvalidMatrixRow)?;

        for (encap_id, stats) in tunnel_rows {
            matrix.merge_row(flow.clone(), encap_id, stats);
        }

        matrix.add_flow_origins(&flow, parse_origin_list(&row.origin));
    }
    Ok(())
}

/// Reconstruit une matrice depuis des lignes de CSV : labels des fichiers
/// réappliqués ([`apply_row_labels`]) puis flux fusionnés ([`merge_rows`]).
pub fn matrix_from_rows(rows: &[FlowMatrixRow], link_type: LinkType) -> Result<FlowMatrix> {
    let mut matrix = FlowMatrix::new();
    matrix.link_type = Some(link_type);
    apply_row_labels(&mut matrix, rows);
    merge_rows(&mut matrix, rows, link_type)?;
    Ok(matrix)
}

/// Fusionne plusieurs fichiers de matrice CSV en une matrice unique. Les
/// fichiers doivent porter le même type de liaison (fusion inter-DLT refusée),
/// que la matrice produite conserve.
pub fn merge_matrix_files(inputs: &[PathBuf]) -> Result<FlowMatrix> {
    let link_type = common_matrix_link_type(inputs)?;
    let rows = read_matrix_rows_from_files(inputs)?;
    matrix_from_rows(&rows, link_type)
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

    /// L'export écrit le préambule `#SFMS` en tête et le réimport le relit :
    /// aller-retour complet, DLT du relevé conservé.
    #[test]
    fn export_writes_sfms_preamble_and_reimport_reads_it_back() {
        let dir = std::env::temp_dir().join("sonar_core_preamble_roundtrip_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let out = dir.join("releve.csv");

        let matrix = merge_matrix_files(&[fixture("20260703_NP_Matrice.csv")])
            .expect("fixture sans préambule -> Ethernet implicite");
        assert_eq!(
            matrix.link_type,
            Some(packet_parser::LinkType::ETHERNET),
            "un export antérieur au préambule est un relevé Ethernet"
        );
        matrix
            .export_to_csv(out.to_string_lossy().into_owned())
            .expect("export");

        let first_line = std::fs::read_to_string(&out)
            .expect("relecture")
            .lines()
            .next()
            .map(str::to_string)
            .expect("fichier non vide");
        assert_eq!(first_line, "#SFMS version=2 dlt=ETHERNET");

        let reimported = merge_matrix_files(&[out]).expect("réimport avec préambule");
        assert_eq!(reimported.row_count(), matrix.row_count());
        assert_eq!(
            reimported.link_type,
            Some(packet_parser::LinkType::ETHERNET)
        );
    }

    /// RAW, SLL et SLL2 reconstruisent le vrai type de liaison depuis le
    /// préambule. Une adresse cooked non-Ethernet (4 octets) et un protocole
    /// inconnu restent inversibles sans colonne supplémentaire.
    #[test]
    fn raw_sll_and_sll2_matrix_identities_round_trip() {
        let dir = std::env::temp_dir().join("sonar_core_non_ethernet_roundtrip_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");

        let mut base = read_matrix_rows(&fixture("20260703_NP_Matrice.csv"))
            .expect("fixture")
            .remove(0);
        base.mac_destination.clear();
        base.vlan_id = None;
        base.origin.clear();

        for (link_type, source_address, protocol, file_name) in [
            (LinkType::RAW, "", "IPv4", "raw.csv"),
            (LinkType::LINUX_SLL, "01:02:03:04", "0xABCD", "sll.csv"),
            (
                LinkType::LINUX_SLL2,
                "01:02:03:04:05:06:07:08",
                "0xABCD",
                "sll2.csv",
            ),
        ] {
            let mut row = base.clone();
            row.mac_source = source_address.to_string();
            row.protocol_data_link = protocol.to_string();
            let path = dir.join(file_name);
            FlowMatrix::write_rows_to_csv(
                std::slice::from_ref(&row),
                Some(link_type),
                &path.to_string_lossy(),
            )
            .expect("export non-Ethernet");

            let contents = std::fs::read_to_string(&path).expect("relecture");
            assert!(
                !contents.lines().nth(1).unwrap().contains("link_details"),
                "le schéma SFMS reste inchangé"
            );

            let matrix =
                merge_matrix_files(std::slice::from_ref(&path)).expect("réimport non-Ethernet");
            assert_eq!(matrix.link_type, Some(link_type));
            assert_eq!(matrix.row_count(), 1);
            assert!(
                matrix
                    .matrix
                    .keys()
                    .all(|flow| flow.data_link.link_type() == link_type),
                "les clés doivent être reconstruites dans le vrai DLT"
            );
            let exported = matrix.to_flat_vec();
            assert_eq!(exported[0].mac_source, source_address);
            assert_eq!(exported[0].protocol_data_link, protocol);
            assert_eq!(exported[0].count, row.count);
            assert_eq!(exported[0].total_bytes, row.total_bytes);
        }
    }

    /// Deux matrices SLL2 du même réseau, observées par deux sondes, fusionnent
    /// la conversation et cumulent leur provenance sans `link_details`.
    #[test]
    fn sll2_matrices_from_two_observation_points_merge_one_row() {
        let dir = std::env::temp_dir().join("sonar_core_sll2_multi_probe_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");

        let mut row = read_matrix_rows(&fixture("20260703_NP_Matrice.csv"))
            .expect("fixture")
            .remove(0);
        row.mac_source = "01:02:03:04".to_string();
        row.mac_destination.clear();
        row.vlan_id = None;
        row.protocol_data_link = "IPv4".to_string();
        row.origin.clear();

        let site_a = dir.join("site-a.csv");
        let site_b = dir.join("site-b.csv");
        for path in [&site_a, &site_b] {
            FlowMatrix::write_rows_to_csv(
                std::slice::from_ref(&row),
                Some(LinkType::LINUX_SLL2),
                &path.to_string_lossy(),
            )
            .expect("export sonde");
        }

        let merged = merge_matrix_files(&[site_a, site_b]).expect("fusion multi-sondes");
        assert_eq!(merged.link_type, Some(LinkType::LINUX_SLL2));
        assert_eq!(merged.row_count(), 1, "la sonde ne segmente pas le flux");
        let merged_row = &merged.to_flat_vec()[0];
        assert_eq!(merged_row.count, row.count * 2);
        assert_eq!(merged_row.total_bytes, row.total_bytes * 2);
        assert_eq!(merged_row.origin, "site-a.csv|site-b.csv");
    }

    /// Un LINKTYPE sans projection SFMS définie reste refusé explicitement.
    #[test]
    fn reimport_of_an_undefined_sfms_linktype_is_refused() {
        let dir = std::env::temp_dir().join("sonar_core_unknown_dlt_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("releve_user0.csv");
        let rows = read_matrix_rows(&fixture("20260703_NP_Matrice.csv")).expect("fixture");
        FlowMatrix::write_rows_to_csv(&rows, Some(LinkType(147)), &path.to_string_lossy())
            .expect("export");

        let err = read_matrix_rows(&path).expect_err("DLT sans projection refusé");
        assert!(
            matches!(err, crate::SonarCoreError::UnreimportableLinkType { .. }),
            "erreur dédiée attendue, obtenu : {err}"
        );
    }

    /// Deux relevés de DLT différents ne se fusionnent pas : rejet explicite
    /// (arbitrage du 14/07/2026), avec les deux types cités.
    #[test]
    fn merging_matrices_of_different_dlt_is_refused() {
        let dir = std::env::temp_dir().join("sonar_core_mixed_dlt_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let body =
            std::fs::read_to_string(fixture("20260703_NP_Matrice.csv")).expect("lecture fixture");
        let ethernet = dir.join("site-eth.csv");
        let sll = dir.join("sonde-sll.csv");
        std::fs::write(&ethernet, format!("#SFMS version=1 dlt=ETHERNET\n{body}")).expect("a");
        std::fs::write(&sll, format!("#SFMS version=1 dlt=LINUX_SLL\n{body}")).expect("b");

        let Err(err) = merge_matrix_files(&[ethernet, sll]) else {
            panic!("la fusion inter-DLT doit être refusée");
        };
        assert!(
            matches!(err, crate::SonarCoreError::MixedLinkTypes { .. }),
            "erreur dédiée attendue, obtenu : {err}"
        );
        let message = err.to_string();
        assert!(message.contains("LINUX_SLL") && message.contains("ETHERNET"));
    }

    /// Chemins avec espaces, Unicode et caractères spéciaux shell (#88) :
    /// export, réimport et fusion de matrices fonctionnent, et la colonne
    /// `origin` porte fidèlement le nom du fichier.
    #[test]
    fn matrix_paths_with_spaces_unicode_and_special_characters_round_trip() {
        let dir = std::env::temp_dir().join("sonar_core_csv_weird_names_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");

        // Lignes sans provenance : au réimport, la colonne `origin` doit
        // hériter du nom (spécial) du fichier — une ligne qui portait déjà
        // une origine la conserverait, ce qui ne testerait rien.
        let mut rows = read_matrix_rows(&fixture("20260703_NP_Matrice.csv")).expect("fixture");
        for row in &mut rows {
            row.origin.clear();
        }

        for name in [
            "matrice de l'été (v2).csv",
            "relevé 中文 📡.csv",
            "site \"principal\" `usine`.csv",
        ] {
            let path = dir.join(name);
            FlowMatrix::write_rows_to_csv(
                &rows,
                Some(packet_parser::LinkType::ETHERNET),
                &path.to_string_lossy(),
            )
            .unwrap_or_else(|e| panic!("export vers {name}: {e}"));

            let reimported = merge_matrix_files(std::slice::from_ref(&path))
                .unwrap_or_else(|e| panic!("réimport de {name}: {e}"));
            assert!(reimported.row_count() > 0, "{name}");
            assert!(
                reimported
                    .to_flat_vec()
                    .iter()
                    .all(|row| row.origin == *name),
                "{name} : la colonne origin doit porter le nom exact du fichier"
            );
        }
    }

    /// Limite connue documentée (#88) : `|` est le séparateur de la colonne
    /// `origin` ; un nom de fichier qui en contient est découpé au réimport
    /// et devient DEUX origines. Ce test fige le comportement actuel — le
    /// changer demande un échappement du séparateur, pas une régression
    /// silencieuse.
    #[test]
    fn pipe_in_a_matrix_file_name_is_split_by_the_origin_column() {
        let dir = std::env::temp_dir().join("sonar_core_csv_pipe_name_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("site-a|site-b.csv");

        let mut rows = read_matrix_rows(&fixture("20260703_NP_Matrice.csv")).expect("fixture");
        for row in &mut rows {
            row.origin.clear();
        }
        FlowMatrix::write_rows_to_csv(
            &rows,
            Some(packet_parser::LinkType::ETHERNET),
            &path.to_string_lossy(),
        )
        .expect("export");

        let merged = merge_matrix_files(std::slice::from_ref(&path)).expect("réimport");
        let origins: std::collections::BTreeSet<String> =
            merged.origins.values().flatten().cloned().collect();
        assert_eq!(
            origins,
            ["site-a", "site-b.csv"]
                .map(str::to_string)
                .into_iter()
                .collect(),
            "le nom contenant `|` est découpé en deux origines (limite documentée)"
        );
    }

    /// Les champs qui ne peuvent pas appartenir à une projection cooked sont
    /// rejetés avec le vrai numéro de ligne, jamais ignorés à la reconstruction.
    #[test]
    fn invalid_cooked_identity_fields_are_rejected_explicitly() {
        let dir = std::env::temp_dir().join("sonar_core_invalid_cooked_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");

        let mut base = read_matrix_rows(&fixture("20260703_NP_Matrice.csv"))
            .expect("fixture")
            .remove(0);
        base.mac_source = "01:02:03:04".to_string();
        base.mac_destination.clear();
        base.vlan_id = None;
        base.protocol_data_link = "IPv4".to_string();

        let mut long_address = base.clone();
        long_address.mac_source = "00:01:02:03:04:05:06:07:08".to_string();
        let mut destination = base.clone();
        destination.mac_destination = "aa:bb:cc:dd:ee:ff".to_string();
        let mut vlan = base.clone();
        vlan.vlan_id = Some(42);
        let mut protocol = base;
        protocol.protocol_data_link = "illisible".to_string();

        for (name, row, expected) in [
            ("address.csv", long_address, "adresse cooked invalide"),
            ("destination.csv", destination, "adresse destination"),
            ("vlan.csv", vlan, "porter de VLAN"),
            (
                "protocol.csv",
                protocol,
                "protocol_data_link cooked invalide",
            ),
        ] {
            let path = dir.join(name);
            FlowMatrix::write_rows_to_csv(
                &[row],
                Some(LinkType::LINUX_SLL2),
                &path.to_string_lossy(),
            )
            .expect("écriture du cas invalide");
            let err = read_matrix_rows(&path).expect_err("identité cooked invalide");
            let message = err.to_string();
            assert!(message.contains("ligne 3"), "{name}: {message}");
            assert!(message.contains(expected), "{name}: {message}");
        }
    }
    /// Le numéro de ligne des erreurs tient compte du préambule : la première
    /// ligne de données d'un fichier avec préambule est la ligne 3.
    #[test]
    fn line_numbers_account_for_the_preamble() {
        let dir = std::env::temp_dir().join("sonar_core_preamble_lines_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tempdir");
        let bad = dir.join("bad.csv");
        std::fs::write(
            &bad,
            "#SFMS version=1 dlt=ETHERNET\nmac_source,mac_destination\nnot,enough\n",
        )
        .expect("écriture");

        let err = read_matrix_rows(&bad).expect_err("fichier invalide");
        assert!(
            err.to_string().contains("ligne 3"),
            "le numéro de ligne doit compter le préambule: {err}"
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
