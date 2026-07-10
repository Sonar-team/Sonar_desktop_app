//! Import des fichiers de labels CSV : lecture, validation de format,
//! déduplication « premier gagné », arbitrage des conflits et application
//! des labels au store, à la matrice et au graphe.

use log::{error, info};
use std::{
    collections::HashMap,
    io::ErrorKind,
    net::IpAddr,
    sync::{Arc, Mutex},
};
use tauri::{State, ipc::Channel};

// Helpers utilisés uniquement par la détection de conflits historique (tests).
#[cfg(test)]
use crate::state::flow_matrix::{is_non_unicast_mac, is_placeholder_ip};

use crate::{
    errors::{CaptureStateError, label::LabelError},
    events::CaptureEvent,
    setup::labels::{clean_csv_field, parse_label_fields},
    state::{
        capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData, labels_list::LabelStore,
    },
};

use super::event_channel;

// La détection de conflits « historique » (basée fichier) ne sert plus qu'aux
// tests : en production, `dedup_labels_first_wins` déduplique et enregistre les
// conflits pendant l'import.
#[cfg(test)]
type ConflictsList = Vec<(usize, usize, String, String, String, String, String)>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelRow {
    line: usize,
    raw: String,
    mac: String,
    ip: String,
    label: String,
}

impl LabelRow {
    fn into_tuple(self) -> (String, String, String) {
        (self.mac, self.ip, self.label)
    }
}

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

/// Déduplique les lignes par clé `(mac, ip)` en gardant la **première**
/// occurrence. Retourne les lignes retenues (une par clé) et la liste des
/// conflits (clés ayant reçu au moins deux labels différents).
fn dedup_labels_first_wins(rows: Vec<LabelRow>) -> (Vec<LabelRow>, Vec<LabelConflictReport>) {
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

fn read_label_file(csv_path: &str) -> Result<String, CaptureStateError> {
    match std::fs::read_to_string(csv_path) {
        Ok(csv_data) => Ok(csv_data),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command(async)]
pub fn get_label_rows(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<Vec<(String, String, String)>, CaptureStateError> {
    let label_store = label_store.lock()?;
    Ok(label_store.get().clone())
}

/// Vide le store de labels (la table « contenu importé » du panneau).
///
/// Sémantique **assumée** : ne retire pas les labels déjà appliqués à la
/// matrice ou au graphe — même logique de fusion que [`import_label_file`].
#[tauri::command(async)]
pub fn clear_label_store(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<(), CaptureStateError> {
    let mut labels = label_store.lock()?;
    labels.clear();
    Ok(())
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

fn read_label_rows(csv_path: &str) -> Result<Vec<LabelRow>, CaptureStateError> {
    Ok(read_label_records(csv_path)?
        .into_iter()
        .filter_map(|(line, record)| {
            let raw = label_record_to_display(&record);
            parse_label_fields(record.iter()).map(|(mac, ip, label)| LabelRow {
                line,
                raw,
                mac,
                ip,
                label,
            })
        })
        .collect())
}

fn verif_label_rows_format(file: String) -> Result<(), CaptureStateError> {
    read_label_records(&file)?;
    Ok(())
}

#[cfg(test)]
fn verif_labels_conflicts(file_path: String) -> Result<(), CaptureStateError> {
    let rows = read_label_rows(&file_path)?;

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

pub fn verif_mac_ip_format(csv_path: String) -> Result<(), CaptureStateError> {
    let mut invalid_ip: Vec<(usize, String, String)> = Vec::new();
    let mut invalid_mac: Vec<(usize, String, String)> = Vec::new();

    let rows = read_label_rows(&csv_path)?;

    let skip = usize::from(rows.first().is_some_and(is_header_row));
    for row in rows.iter().skip(skip) {
        if !is_ip_address(&row.ip) && !row.ip.is_empty() {
            invalid_ip.push((row.line, row.ip.clone(), row.raw.clone()));
        }
        if !is_mac_address(&row.mac) && !row.mac.is_empty() {
            invalid_mac.push((row.line, row.mac.clone(), row.raw.clone()));
        }
    }

    if !invalid_ip.is_empty() || !invalid_mac.is_empty() {
        return Err(LabelError::InvalidMacIpFormat {
            invalid_mac,
            invalid_ip,
        }
        .into());
    }
    Ok(())
}

fn is_ip_address(value: &str) -> bool {
    value.parse::<IpAddr>().is_ok()
}

fn is_mac_address(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

// Une ligne d'en-tête ("mac,ip,label") ne peut être que la première du fichier :
// on la reconnaît parce que ni son champ MAC ni son champ IP ne sont des adresses valides.
fn is_header_row(row: &LabelRow) -> bool {
    !is_mac_address(&row.mac) && !is_ip_address(&row.ip)
}

/// Importe un fichier de labels CSV.
///
/// Sémantique **assumée de fusion** côté matrice/graphe : le store est
/// remplacé par le contenu du fichier, mais les labels précédemment appliqués
/// à la matrice ou aux nœuds du graphe survivent tant qu'une nouvelle valeur
/// ne les écrase pas (clé `(mac, ip)` identique). Un import ne « désétiquette »
/// donc jamais un équipement. Pour repartir de zéro : reset de la capture.
#[tauri::command(async)]
pub fn import_label_file(
    incoming_file_path: String,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<LabelImportReport, CaptureStateError> {
    let on_event = event_channel(&capture_state, on_event)?;

    // Les erreurs de FORMAT restent bloquantes ; les conflits de labels, eux,
    // ne bloquent plus : on garde le premier label par clé (mac, ip) et on
    // enregistre les doublons écartés pour arbitrage.
    verif_label_rows_format(incoming_file_path.clone())?;
    verif_mac_ip_format(incoming_file_path.clone())?;

    let conflicts = {
        let mut label_store = label_store.lock()?;
        label_store.clear();

        let mut rows = read_label_rows(&incoming_file_path)?;

        // L'en-tête éventuel est écarté ici pour que le store ne contienne que des données.
        if rows.first().is_some_and(is_header_row) {
            rows.remove(0);
        }

        let (rows, conflicts) = dedup_labels_first_wins(rows);

        for row in rows {
            label_store.add(row.into_tuple())
        }

        conflicts
    };

    let applied = {
        let mut state_label = state_label.lock()?;
        labels_to_matrix(label_store, &mut state_label)?;

        // Réapplique les labels sur les nœuds du graphe (même résolution que la
        // capture) et notifie le front nœud par nœud : pas de snapshot complet,
        // pour préserver la disposition du graphe côté UI.
        let updates = {
            let mut graph_guard = graph.lock()?;
            graph_guard.refresh_labels(|mac, ip| state_label.get_label(mac, ip))
        };

        info!(
            "[import_label_file] {} label(s) de nœud mis à jour dans le graphe, {} conflit(s) résolu(s)",
            updates.len(),
            conflicts.len()
        );

        for update in &updates {
            if let Err(e) = on_event.send(CaptureEvent::Graph { update }) {
                error!("Erreur lors de l'envoi du GraphUpdate label: {:?}", e);
            }
        }

        updates.len()
    };

    // Mémorise les conflits pour le module d'arbitrage.
    conflict_store.lock()?.conflicts = conflicts.clone();

    Ok(LabelImportReport { applied, conflicts })
}

/// Conflits de labels résolus lors du dernier import (pour le module
/// d'arbitrage / la vue de gestion des labels).
#[tauri::command]
pub fn get_label_conflicts(
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
) -> Result<Vec<LabelConflictReport>, CaptureStateError> {
    Ok(conflict_store.lock()?.conflicts.clone())
}

/// Arbitrage d'un conflit : applique le `label` choisi à la clé `(mac, ip)`
/// dans la matrice, le store et le graphe, retire le conflit de la liste et
/// renvoie les conflits restants.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn resolve_label_conflict(
    mac: String,
    ip: String,
    label: String,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
) -> Result<Vec<LabelConflictReport>, CaptureStateError> {
    // Applique le label choisi à la matrice (résolution des flux) et au store
    // (table + réexport).
    matrix
        .lock()?
        .add_label(mac.clone(), ip.clone(), label.clone());
    label_store.lock()?.set(&mac, &ip, &label);

    // Rafraîchit le nœud du graphe et notifie l'UI, sans reconstruire le graphe.
    let graph_update = graph.lock()?.update_node_label(&mac, &ip, label.clone());
    let on_event = capture_state.lock()?.on_event.clone();
    if let (Some(update), Some(on_event)) = (graph_update, on_event)
        && let Err(e) = on_event.send(CaptureEvent::Graph { update: &update })
    {
        error!("Erreur d'envoi du GraphUpdate arbitrage: {e}");
    }

    // Retire le conflit résolu, renvoie ceux qui restent.
    let mut store = conflict_store.lock()?;
    store.conflicts.retain(|c| !(c.mac == mac && c.ip == ip));
    Ok(store.conflicts.clone())
}

pub fn labels_to_matrix(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    let label_store = label_store.lock()?;
    copy_labels_to_matrix(&label_store, matrice)
}

pub fn copy_labels_to_matrix(
    label_store: &LabelStore,
    matrice: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    for (mac, ip, label) in label_store.get() {
        matrice.add_label(mac.to_string(), ip.to_string(), label.to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_support::TempDir;
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn valid_csv_format_returns_ok() {
        let dir = TempDir::new("sonar_test_valid_csv_format");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_label_rows_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
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

    #[test]
    fn valid_mac_and_ip_returns_ok() {
        let dir = TempDir::new("sonar_test_valid_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:ee:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn malformed_ip_returns_error() {
        let dir = TempDir::new("sonar_test_malformed_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "mac,ip,label\naa:bb:cc:dd:ee:ff,192.168.11,mon-pc\n",
        )
        .unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        match result.unwrap_err() {
            CaptureStateError::Label(LabelError::InvalidMacIpFormat {
                invalid_mac,
                invalid_ip,
            }) => {
                assert!(invalid_mac.is_empty());
                assert_eq!(
                    invalid_ip,
                    vec![(
                        2,
                        "192.168.11".to_string(),
                        "aa:bb:cc:dd:ee:ff,192.168.11,mon-pc".to_string()
                    )]
                );
            }
            error => panic!("erreur inattendue: {error:?}"),
        }
    }

    #[test]
    fn malformed_mac_returns_error() {
        let dir = TempDir::new("sonar_test_malformed_mac");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "aa:bb:cc:dd:e:ff,192.168.1.1,mon-pc\n").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_err());
    }

    #[test]
    fn empty_mac_and_ip_are_accepted() {
        let dir = TempDir::new("sonar_test_empty_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(
            &file_path,
            "aa:bb:cc:dd:ee:ff,,mon-pc\n,192.168.1.1,mon-pc\n,,mon-pc\n",
        )
        .unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

    #[test]
    fn empty_file_mac_ip_format_returns_ok() {
        let dir = TempDir::new("sonar_test_empty_file_mac_ip");
        let file_path = dir.path().join("labels.csv");
        fs::write(&file_path, "").unwrap();

        let result = verif_mac_ip_format(file_path.to_str().unwrap().to_string());

        assert!(result.is_ok());
    }

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

    #[test]
    fn labels_to_matrix_loads_labels_into_matrix() {
        let mut matrix = FlowMatrix::new();
        let mut label_store = LabelStore::new();
        let tab_test = [
            (
                String::from("aa:bb:cc:dd:ee:ff"),
                String::from("192.168.1.1"),
                String::from("mon-pc"),
            ),
            (
                String::from("aa:bb:cc:d5:ee:ff"),
                String::from("192.168.1.10"),
                String::from("ma-télé"),
            ),
            (
                String::from("aa:bb:cc:dd:ee:55"),
                String::from("aa:bb:cc:dd:ee:55"),
                String::from("mon-aspi"),
            ),
        ];

        for row in tab_test {
            label_store.add(row);
        }

        copy_labels_to_matrix(&label_store, &mut matrix).unwrap();

        assert_eq!(matrix.get_label_list().len(), 3)
    }

    /// Rejoue la chaîne complète de `import_label_file` sur les fichiers réels
    /// de `test_files/` : vérifications, écart de l'en-tête, remplissage du
    /// store puis résolution des labels comme le ferait la matrice de flux.
    #[test]
    fn import_real_label_file_resolves_labels_for_matrix_hosts() {
        let label_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_NP_Labels.csv")
            .to_str()
            .unwrap()
            .to_string();

        verif_label_rows_format(label_path.clone()).unwrap();
        verif_mac_ip_format(label_path.clone()).unwrap();
        verif_labels_conflicts(label_path.clone()).unwrap();

        let mut rows = read_label_rows(&label_path).unwrap();
        if rows.first().is_some_and(is_header_row) {
            rows.remove(0);
        }

        let mut label_store = LabelStore::new();
        for row in rows {
            label_store.add(row.into_tuple());
        }
        assert_eq!(label_store.get().len(), 9, "l'en-tête doit être écarté");

        let mut matrix = FlowMatrix::new();
        copy_labels_to_matrix(&label_store, &mut matrix).unwrap();

        // Correspondance exacte MAC + IP.
        assert_eq!(
            matrix.get_label("e0:d5:5e:28:9b:d4", "192.168.1.181"),
            Some(String::from("PC Sonar"))
        );
        assert_eq!(
            matrix.get_label("44:15:24:20:a5:64", "192.168.1.254"),
            Some(String::from("Passerelle"))
        );
        // Repli sur l'IP seule, prioritaire sur la MAC seule.
        assert_eq!(
            matrix.get_label("e0:d5:5e:28:9b:d4", "2001:861:3fc7:9b00:e09f:22b:19f7:888e"),
            Some(String::from("PC Sonar public"))
        );
        // Repli sur la MAC seule quand l'IP est inconnue du fichier.
        assert_eq!(
            matrix.get_label("e0:d5:5e:28:9b:d4", "fe80::b861:e852:5fa3:d3e5"),
            Some(String::from("Machine de capture"))
        );
        // La notation CIDR du fichier est ramenée à l'adresse seule.
        assert_eq!(
            matrix.get_label("44:15:24:20:a5:64", "2600:1901:0:3084::"),
            Some(String::from("Serveur GCP (CIDR)"))
        );
        // Un label vide devient "Label?".
        assert_eq!(
            matrix.get_label("44:15:24:20:a5:64", "2606:50c0:8002::154"),
            Some(String::from("Label?"))
        );
        // Hôte absent du fichier de labels : aucun label.
        assert_eq!(matrix.get_label("44:15:24:20:a5:64", "2607:6bc0::10"), None);
    }
}
