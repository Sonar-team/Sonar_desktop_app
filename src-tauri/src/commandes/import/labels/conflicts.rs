//! Détection et résolution des conflits de labels : déduplication « premier
//! gagné » à l'import, et types partagés avec le front pour l'arbitrage.

use std::collections::HashMap;

#[cfg(test)]
use crate::state::flow_matrix::{is_non_unicast_mac, is_placeholder_ip};

#[cfg(test)]
use crate::errors::{CaptureStateError, label::LabelError};

use super::csv::LabelRow;
#[cfg(test)]
use super::csv::is_header_row;

// La détection de conflits « historique » (basée fichier) ne sert plus qu'aux
// tests : en production, `dedup_labels_first_wins` déduplique et enregistre les
// conflits pendant l'import.
#[cfg(test)]
type ConflictsList = Vec<(usize, usize, String, String, String, String, String)>;

/// Un choix de label en compétition sur une même clé : (n° de ligne, label, ligne brute).
pub type LabelChoice = (usize, String, String);

/// Conflit de labels résolu automatiquement à l'import : une même clé
/// `(mac, ip)` portait plusieurs labels différents. Le premier est retenu
/// (« premier gagné »), les suivants sont écartés mais conservés ici pour que
/// l'utilisateur puisse arbitrer via le module de gestion des labels.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabelConflictReport {
    pub mac: String,
    pub ip: String,
    pub kept: LabelChoice,
    pub dropped: Vec<LabelChoice>,
}

/// Résultat d'un import de labels : nombre de labels appliqués et conflits
/// résolus (non bloquants).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabelImportReport {
    pub applied: usize,
    pub conflicts: Vec<LabelConflictReport>,
}

/// État partagé : conflits du dernier import de labels, à arbitrer.
#[derive(Default)]
pub struct LabelConflictStore {
    pub conflicts: Vec<LabelConflictReport>,
}

/// Choix d'arbitrage envoyé par le front : le label retenu pour une clé
/// `(mac, ip)` en conflit.
#[derive(Debug, serde::Deserialize)]
pub struct LabelResolution {
    pub mac: String,
    pub ip: String,
    pub label: String,
}

/// Déduplique les lignes par clé `(mac, ip)` en gardant la **première**
/// occurrence. Retourne les lignes retenues (une par clé) et la liste des
/// conflits (clés ayant reçu au moins deux labels différents).
pub(super) fn dedup_labels_first_wins(
    rows: Vec<LabelRow>,
) -> (Vec<LabelRow>, Vec<LabelConflictReport>) {
    let mut order: Vec<(String, String)> = Vec::new();
    let mut kept: HashMap<(String, String), LabelRow> = HashMap::new();
    let mut dropped: HashMap<(String, String), Vec<LabelChoice>> = HashMap::new();

    for row in rows {
        let key = (row.mac.clone(), row.ip.clone());
        match kept.get(&key) {
            None => {
                order.push(key.clone());
                kept.insert(key, row);
            }
            Some(first) => {
                if first.label != row.label {
                    dropped.entry(key).or_default().push((
                        row.line,
                        row.label.clone(),
                        row.raw.clone(),
                    ));
                }
                // Doublon (même clé) : ignoré, le premier est conservé.
            }
        }
    }

    let mut conflicts = Vec::new();
    let mut kept_rows = Vec::new();
    for key in order {
        // Clé absente = déjà consommée (doublon dans `order`) : on passe.
        let Some(row) = kept.remove(&key) else {
            continue;
        };
        if let Some(dropped_choices) = dropped.remove(&key) {
            conflicts.push(LabelConflictReport {
                mac: row.mac.clone(),
                ip: row.ip.clone(),
                kept: (row.line, row.label.clone(), row.raw.clone()),
                dropped: dropped_choices,
            });
        }
        kept_rows.push(row);
    }

    (kept_rows, conflicts)
}

#[cfg(test)]
pub(super) fn verif_labels_conflicts(file_path: String) -> Result<(), CaptureStateError> {
    let rows = super::csv::read_label_rows(&file_path)?;

    // Le store de labels est indexé par la clé `(mac, ip)`. Deux lignes ne
    // s'écrasent (perte de donnée) que si elles portent **la même clé** avec un
    // label différent. Deux équipements distincts partageant une IP (MAC
    // différentes) cohabitent sans ambiguïté : ce n'est pas un conflit.
    //
    // Les champs non identifiants (IP placeholder `0.0.0.0`/`::`, MAC broadcast
    // ou multicast) sont ignorés : ils peuvent être partagés par plusieurs
    // machines et produisaient de faux conflits, notamment sur des fichiers de
    // labels exportés par SONAR lui-même.
    let mut same_ip_different_label: ConflictsList = Vec::new();
    // Conservé pour compatibilité de l'API d'erreur ; plus alimenté.
    let same_ip_different_mac: ConflictsList = Vec::new();

    let is_identifying =
        |row: &LabelRow| !is_placeholder_ip(&row.ip) && !is_non_unicast_mac(&row.mac);

    let skip = usize::from(rows.first().is_some_and(is_header_row));
    for (i, row1) in rows.iter().enumerate().skip(skip) {
        if !is_identifying(row1) || (row1.mac.is_empty() && row1.ip.is_empty()) {
            continue;
        }
        for row2 in rows[i + 1..].iter() {
            if !is_identifying(row2) {
                continue;
            }
            // Même clé de store (mac, ip) mais label différent -> écrasement
            // silencieux -> conflit réel.
            if row1.mac == row2.mac && row1.ip == row2.ip && row1.label != row2.label {
                same_ip_different_label.push((
                    row1.line,
                    row2.line,
                    row1.ip.clone(),
                    row1.label.clone(),
                    row2.label.clone(),
                    row1.raw.clone(),
                    row2.raw.clone(),
                ))
            }
        }
    }

    if !same_ip_different_label.is_empty() || !same_ip_different_mac.is_empty() {
        Err(LabelError::LabelLinesConflicts {
            same_ip_diff_mac: same_ip_different_mac,
            same_ip_diff_label: same_ip_different_label,
        }
        .into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::test_support::TempDir;
    use super::*;
    use std::fs;

    #[test]
    fn empty_file_conflicts_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file_conflicts");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn no_conflict_returns_ok() {
        let dir = TempDir::new("sonar_test_no_conflict");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:1f,192.168.1.2,ma-tablette\naa:bb:cc:dd:ee:ff,192.168.1.3,mon-tel\n").unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok())
    }

    #[test]
    fn same_ip_different_mac_is_not_a_conflict() {
        // Deux équipements distincts (MAC différentes) partageant une IP :
        // clés (mac, ip) distinctes, cohabitation dans le store -> pas un conflit.
        let dir = TempDir::new("sonar_test_same_ip_diff_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,poste-A\naa:bb:cc:dd:ee:1f,192.168.1.1,poste-B\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn same_mac_ip_key_different_label_returns_conflict_error() {
        // Même clé (mac, ip) avec deux labels -> écrasement silencieux -> conflit.
        let dir = TempDir::new("sonar_test_same_key_diff_label");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\naa:bb:cc:dd:ee:ff,192.168.1.1,ma-tablette\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::LabelLinesConflicts {
                same_ip_diff_mac,
                same_ip_diff_label,
            }) => {
                assert!(same_ip_diff_mac.is_empty());
                assert_eq!(same_ip_diff_label.len(), 1);
                let (_, _, ip, l1, l2, _, _) = &same_ip_diff_label[0];
                assert_eq!(ip, "192.168.1.1");
                assert_eq!(l1, "mon-pc");
                assert_eq!(l2, "ma-tablette");
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn placeholder_ip_does_not_trigger_conflict() {
        // Plusieurs équipements avec 0.0.0.0 / :: : IP non identifiante -> pas de conflit.
        let dir = TempDir::new("sonar_test_placeholder_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "48:21:0b:41:45:65,0.0.0.0,TVD09\nfc:3f:db:37:5e:0f,0.0.0.0,SIE\n48:21:0b:41:45:65,::,TVD09\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn broadcast_mac_does_not_trigger_conflict() {
        // Une ligne avec MAC broadcast sur une IP réelle ne doit pas entrer en
        // conflit avec la MAC réelle du même équipement.
        let dir = TempDir::new("sonar_test_broadcast_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "48:9e:bd:45:40:92,100.180.65.40,SIC21-CSD\nff:ff:ff:ff:ff:ff,100.180.65.40,SIC21-CSD\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    fn label_row(line: usize, mac: &str, ip: &str, label: &str) -> LabelRow {
        LabelRow {
            line,
            raw: format!("{mac},{ip},{label}"),
            mac: mac.to_string(),
            ip: ip.to_string(),
            label: label.to_string(),
        }
    }

    #[test]
    fn dedup_first_wins_keeps_first_and_records_conflict() {
        let rows = vec![
            label_row(1, "48:9e:bd:31:62:32", "100.180.65.57", "SIE - PC TELEC"),
            label_row(2, "48:9e:bd:31:62:32", "100.180.65.57", "TVD09"),
            label_row(3, "aa:bb:cc:dd:ee:ff", "192.168.1.1", "autre"),
        ];

        let (kept, conflicts) = dedup_labels_first_wins(rows);

        // Une seule ligne conservée pour la clé en doublon (la première).
        assert_eq!(kept.len(), 2);
        let telec = kept.iter().find(|r| r.mac == "48:9e:bd:31:62:32").unwrap();
        assert_eq!(telec.label, "SIE - PC TELEC", "le premier label est gardé");

        // Le conflit est enregistré avec le label écarté.
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].ip, "100.180.65.57");
        assert_eq!(conflicts[0].kept.1, "SIE - PC TELEC");
        assert_eq!(conflicts[0].dropped.len(), 1);
        assert_eq!(conflicts[0].dropped[0].1, "TVD09");
    }

    #[test]
    fn dedup_first_wins_no_conflict_when_same_label_repeated() {
        let rows = vec![
            label_row(1, "48:9e:bd:45:40:92", "100.180.65.40", "SIC21-CSD"),
            label_row(2, "48:9e:bd:45:40:92", "100.180.65.40", "SIC21-CSD"),
        ];

        let (kept, conflicts) = dedup_labels_first_wins(rows);

        assert_eq!(kept.len(), 1);
        assert!(conflicts.is_empty(), "même label répété -> pas un conflit");
    }

    #[test]
    fn dedup_first_wins_distinct_keys_are_all_kept() {
        // Deux équipements distincts partageant une IP : clés différentes -> conservés.
        let rows = vec![
            label_row(1, "84:a9:38:5e:d1:51", "100.180.65.32", "SIE - COMMIS"),
            label_row(2, "a0:2b:b8:3d:a2:7e", "100.180.65.32", "SIC21-SIC"),
        ];

        let (kept, conflicts) = dedup_labels_first_wins(rows);

        assert_eq!(kept.len(), 2);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn empty_ip_does_not_trigger_conflict() {
        let dir = TempDir::new("sonar_test_empty_ip_no_conflict");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:f1,,mon-pc\naa:bb:cc:dd:ee:ff,,ma-tablette\n",
        )
        .unwrap();

        let result = verif_labels_conflicts(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok())
    }
}
