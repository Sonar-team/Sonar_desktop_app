//! Import des matrices de flux CSV (format de `FlowMatrix::export_to_csv`) :
//! lecture, fusion multi-fichiers avec provenance, reconstruction de la
//! matrice et du graphe.

use log::{error, info};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use sonar_flows_core::csv::{apply_row_labels, merge_rows};

use crate::{
    errors::CaptureStateError,
    events::CaptureEvent,
    state::{
        capture::CaptureState,
        flow_matrix::{FlowMatrix, FlowMatrixRow, unescape_formula_cell},
        graph::GraphData,
        labels_list::LabelStore,
    },
};

use super::{event_channel, labels::copy_labels_to_matrix};

#[tauri::command]
pub fn is_matrix_empty(
    state: tauri::State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<bool, CaptureStateError> {
    Ok(state.lock()?.matrix.is_empty())
}

/// Lit un CSV de matrice de flux (format de `FlowMatrix::export_to_csv`),
/// entièrement validé avant de toucher à l'état — lecture et validation
/// stricte (#148) déléguées au cœur partagé. La production passe par
/// `read_matrix_rows_per_file` ; ce raccourci ne sert plus qu'aux tests.
#[cfg(test)]
fn read_matrix_rows(csv_path: &str) -> Result<Vec<FlowMatrixRow>, CaptureStateError> {
    Ok(sonar_flows_core::csv::read_matrix_rows(
        std::path::Path::new(csv_path),
    )?)
}

/// Lit chaque fichier de matrice et retourne ses lignes groupées par fichier,
/// pour la comptabilité par fichier (événements `Finished`). La lecture et
/// l'héritage de la colonne `origin` viennent du cœur partagé.
fn read_matrix_rows_per_file(
    incoming_file_paths: &[String],
) -> Result<Vec<(String, Vec<FlowMatrixRow>)>, CaptureStateError> {
    if incoming_file_paths.is_empty() {
        return Err(std::io::Error::other("Aucun fichier de matrice sélectionné").into());
    }

    let paths: Vec<PathBuf> = incoming_file_paths.iter().map(PathBuf::from).collect();
    Ok(sonar_flows_core::csv::read_matrix_rows_per_file(&paths)?
        .into_iter()
        .zip(incoming_file_paths)
        .map(|((_, rows), path)| (path.clone(), rows))
        .collect())
}

// La production passe par `read_matrix_rows_per_file` (comptabilité par
// fichier) ; cette version aplatie ne sert plus qu'aux tests.
#[cfg(test)]
fn read_matrix_rows_from_files(
    incoming_file_paths: &[String],
) -> Result<Vec<FlowMatrixRow>, CaptureStateError> {
    Ok(read_matrix_rows_per_file(incoming_file_paths)?
        .into_iter()
        .flat_map(|(_, rows)| rows)
        .collect())
}

fn rebuild_matrix_and_graph_from_rows(
    rows: &[FlowMatrixRow],
    label_store: &LabelStore,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
) -> Result<(), CaptureStateError> {
    matrice.clear();
    graph.clear();

    // Labels du store courant, puis ceux portés par les fichiers (prioritaires
    // à clé égale puisque appliqués après) ; fusion des flux tunnel par tunnel
    // et provenance accumulée par le cœur partagé.
    copy_labels_to_matrix(label_store, matrice)?;
    apply_row_labels(matrice, rows);
    merge_rows(matrice, rows);

    for row in rows {
        let (flow, tunnel_rows) = row.to_flow_and_rows();
        let encap_ids: Vec<u64> = tunnel_rows.iter().filter_map(|(id, _)| *id).collect();

        let source_label = matrice.get_label(&row.mac_source, &row.ip_source);
        let destination_label = matrice.get_label(&row.mac_destination, &row.ip_destination);
        graph.add_packet_flow(
            &flow,
            source_label,
            destination_label,
            row.count,
            row.total_bytes,
            &encap_ids,
        );
    }

    Ok(())
}

#[tauri::command(async)]
pub fn import_matrix_file(
    incoming_file_path: String,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<(), CaptureStateError> {
    import_matrix_files(
        vec![incoming_file_path],
        matrice,
        graph,
        label_store,
        capture_state,
        on_event,
    )
}

#[tauri::command(async)]
pub fn import_matrix_files(
    incoming_file_paths: Vec<String>,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<(), CaptureStateError> {
    // Import et capture sont mutuellement exclusifs : l'import remplacerait
    // la matrice et le graphe pendant que le pipeline les alimente. La phase
    // `Importing` est réservée atomiquement et détenue jusqu'à la fin de la
    // commande, swap inclus (#139).
    let _import_guard = crate::state::capture::ImportGuard::acquire(
        capture_state.inner(),
        "import de matrice CSV",
    )?;
    let on_event = event_channel(&capture_state, on_event)?;

    info!(
        "[import_matrix_files] COMMAND CALLED avec {} fichier(s): {:?}",
        incoming_file_paths.len(),
        incoming_file_paths
    );

    // Un fichier invalide ne doit pas effacer la matrice courante. Les
    // fichiers doivent porter le même DLT (préambule #SFMS, Ethernet
    // implicite pour un export antérieur) : fusion inter-DLT refusée.
    let import_paths: Vec<PathBuf> = incoming_file_paths.iter().map(PathBuf::from).collect();
    let link_type = sonar_flows_core::csv::common_matrix_link_type(&import_paths)?;
    let files = read_matrix_rows_per_file(&incoming_file_paths)?;
    let line_counts: Vec<(String, usize)> = files
        .iter()
        .map(|(path, rows)| (path.clone(), rows.len()))
        .collect();
    let rows: Vec<FlowMatrixRow> = files.into_iter().flat_map(|(_, rows)| rows).collect();

    // Même ordre de verrouillage que convert_from_pcap_list et net_capture
    // (matrice -> graph -> label_store) pour éviter un interblocage ABBA.
    let mut matrice_guard = matrice.lock()?;
    let mut graph_guard = graph.lock()?;
    let mut label_store_guard = label_store.lock()?;

    // Les labels portés par les fichiers entrent dans le store — source de
    // vérité unique (#157), fichier prioritaire à clé égale : ils survivent
    // aux resynchronisations déclenchées par les mutations ultérieures.
    for row in &rows {
        for (mac, ip, label) in [
            (&row.mac_source, &row.ip_source, &row.label_source),
            (
                &row.mac_destination,
                &row.ip_destination,
                &row.label_destination,
            ),
        ] {
            if let Some(label) = label.as_ref().filter(|l| !l.is_empty()) {
                label_store_guard.set(mac, ip, &unescape_formula_cell(label));
            }
        }
    }

    rebuild_matrix_and_graph_from_rows(
        &rows,
        &label_store_guard,
        &mut matrice_guard,
        &mut graph_guard,
    )?;
    // Le relevé reconstruit porte le DLT commun des fichiers importés (le
    // rebuild passe par `clear()`, qui l'avait remis à zéro).
    matrice_guard.link_type = Some(link_type);

    info!(
        "[import_matrix_files] {} fichier(s), {} ligne(s) importée(s) -> {} flux fusionné(s), {} nœuds, {} arêtes",
        incoming_file_paths.len(),
        rows.len(),
        matrice_guard.row_count(),
        graph_guard.nodes.len(),
        graph_guard.edges.len()
    );

    let snapshot = graph_guard.get_all_graph_data();
    if let Err(e) = on_event.send(CaptureEvent::GraphSnapshot {
        graph_data: &snapshot,
    }) {
        error!("Erreur lors de l'envoi de GraphSnapshot: {:?}", e);
    }

    // Mêmes événements que l'import PCAP, pour que la barre de statut mette à
    // jour ses compteurs : un `Finished` par fichier (lignes lues, total de
    // flux fusionnés), puis un `Stats` final qui fixe les totaux affichés
    // (📥 = lignes lues au total, 📊 = flux de la matrice fusionnée).
    let matrix_total_count = matrice_guard.row_count();
    for (path, line_count) in &line_counts {
        if let Err(e) = on_event.send(CaptureEvent::Finished {
            file_name: path,
            packet_total_count: *line_count,
            matrix_total_count,
        }) {
            error!("Erreur lors de l'envoi de Finished: {:?}", e);
        }
    }
    if let Err(e) = on_event.send(CaptureEvent::Stats {
        session_id: 0,
        received: rows.len() as u32,
        dropped: 0,
        if_dropped: 0,
        app_dropped: 0,
        parse_errors: 0,
        integrated: 0,
        processed: matrix_total_count as u32,
    }) {
        error!("Erreur lors de l'envoi de Stats: {:?}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::pcap::handle_pcap_file;
    use super::super::test_support::{TempDir, tunnel_pcap};
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn new_matrix_is_empty() {
        let matrix = FlowMatrix::new();
        assert!(matrix.matrix.is_empty());
    }

    /// Reproduit la construction de `import_matrix_file` (labels puis flux).
    fn build_matrix_and_graph(rows: &[FlowMatrixRow]) -> (FlowMatrix, GraphData) {
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();
        let label_store = LabelStore::new();
        rebuild_matrix_and_graph_from_rows(rows, &label_store, &mut matrix, &mut graph).unwrap();

        (matrix, graph)
    }

    #[test]
    fn import_matrix_rows_merges_duplicate_flows() {
        let mut rows = read_matrix_rows(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test_files/20260703_NP_Matrice.csv")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let mut duplicate = rows[0].clone();
        duplicate.count = 7;
        duplicate.total_bytes = 1234;
        duplicate.last_seen = "2099-01-01 00:00:00".to_string();
        rows.push(duplicate);

        let (flow, _tunnel_rows) = rows[0].to_flow_and_rows();
        let (matrix, graph) = build_matrix_and_graph(&rows);
        let entries = matrix.matrix.get(&flow).unwrap();
        let merged_count: u64 = entries.iter().map(|(_, stats)| stats.count).sum();
        let merged_bytes: u64 = entries.iter().map(|(_, stats)| stats.total_bytes).sum();

        assert_eq!(
            matrix.row_count(),
            30,
            "le flux dupliqué doit être fusionné"
        );
        assert_eq!(merged_count, rows[0].count + 7);
        assert_eq!(merged_bytes, rows[0].total_bytes.saturating_add(1234));
        assert!(
            graph
                .edges
                .values()
                .any(|edge| edge.count >= rows[0].count + 7),
            "le graphe doit recevoir le trafic fusionné"
        );
    }

    /// Deux fichiers portant le même flux : la colonne `origin` de la ligne
    /// fusionnée doit contenir les deux noms de fichiers (triés, joints par `|`).
    #[test]
    fn import_matrix_records_origin_files_per_row() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        let dir = TempDir::new("sonar_test_matrix_origin");
        let file_a = dir.path().join("site-a.csv");
        let file_b = dir.path().join("site-b.csv");
        fs::copy(&source, &file_a).unwrap();
        fs::copy(&source, &file_b).unwrap();

        let rows = read_matrix_rows_from_files(&[
            file_a.to_str().unwrap().to_string(),
            file_b.to_str().unwrap().to_string(),
        ])
        .unwrap();
        let (matrix, _graph) = build_matrix_and_graph(&rows);

        // Deux copies du même fichier : chaque flux est vu dans les deux, donc
        // fusionné (pas dupliqué) et sa colonne `origin` porte les deux noms.
        assert_eq!(matrix.row_count(), 30, "flux fusionnés, pas dupliqués");
        let exported = matrix.to_flat_vec();
        assert!(
            exported.iter().all(|r| r.origin == "site-a.csv|site-b.csv"),
            "chaque ligne doit porter ses deux fichiers d'origine: {:?}",
            exported
                .iter()
                .map(|r| r.origin.clone())
                .collect::<Vec<_>>()
        );
    }

    /// Une matrice déjà fusionnée (colonne `origin` renseignée) réimportée sous
    /// un nouveau nom conserve sa provenance d'origine plutôt que de l'écraser.
    #[test]
    fn reimport_preserves_existing_origin_column() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        let dir = TempDir::new("sonar_test_matrix_origin_preserve");

        // Étape 1 : import sous "brut.csv" (origin = brut.csv) puis export.
        let brut = dir.path().join("brut.csv");
        fs::copy(&source, &brut).unwrap();
        let rows = read_matrix_rows_from_files(&[brut.to_str().unwrap().to_string()]).unwrap();
        let (matrix, _graph) = build_matrix_and_graph(&rows);
        let merged = dir.path().join("fusion.csv");
        FlowMatrix::write_rows_to_csv(
            &matrix.to_flat_vec(),
            matrix.link_type,
            merged.to_str().unwrap(),
        )
        .unwrap();

        // Étape 2 : réimport de fusion.csv -> l'origine "brut.csv" est préservée.
        let rows = read_matrix_rows_from_files(&[merged.to_str().unwrap().to_string()]).unwrap();
        let (matrix, _graph) = build_matrix_and_graph(&rows);
        let exported = matrix.to_flat_vec();
        assert!(
            exported.iter().all(|r| r.origin == "brut.csv"),
            "l'origine héritée doit être conservée, pas remplacée par fusion.csv: {:?}",
            exported
                .iter()
                .map(|r| r.origin.clone())
                .collect::<Vec<_>>()
        );
    }

    /// Deux fichiers dont la colonne `origin` est **déjà remplie** (matrices
    /// issues de fusions antérieures) : les provenances des deux colonnes sont
    /// fusionnées, et le nom des fichiers physiques importés n'est PAS ajouté.
    #[test]
    fn reimport_merges_existing_origin_columns() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        let dir = TempDir::new("sonar_test_matrix_origin_merge");

        // Prépare deux matrices déjà « étiquetées » par des origines distinctes,
        // en réexportant le même fichier source importé sous deux noms.
        let export_with_origin = |raw_name: &str, out_name: &str| {
            let raw = dir.path().join(raw_name);
            fs::copy(&source, &raw).unwrap();
            let rows = read_matrix_rows_from_files(&[raw.to_str().unwrap().to_string()]).unwrap();
            let (matrix, _graph) = build_matrix_and_graph(&rows);
            let out = dir.path().join(out_name);
            FlowMatrix::write_rows_to_csv(
                &matrix.to_flat_vec(),
                matrix.link_type,
                out.to_str().unwrap(),
            )
            .unwrap();
            out.to_str().unwrap().to_string()
        };
        let fusion_a = export_with_origin("a-raw.csv", "fusion-a.csv");
        let fusion_b = export_with_origin("b-raw.csv", "fusion-b.csv");

        // Réimport des deux matrices déjà renseignées (origin = "a-raw.csv" et
        // "b-raw.csv") : chaque flux partagé doit porter les DEUX provenances,
        // sans trace de "fusion-a.csv"/"fusion-b.csv".
        let rows = read_matrix_rows_from_files(&[fusion_a, fusion_b]).unwrap();
        let (matrix, _graph) = build_matrix_and_graph(&rows);
        let exported = matrix.to_flat_vec();
        assert!(
            exported.iter().all(|r| r.origin == "a-raw.csv|b-raw.csv"),
            "les colonnes origin doivent être fusionnées, sans les fichiers de fusion: {:?}",
            exported
                .iter()
                .map(|r| r.origin.clone())
                .collect::<Vec<_>>()
        );
    }

    /// Aller-retour complet sur le pcap réel : import PCAP -> export CSV ->
    /// réimport -> réexport. Les lignes (flux + ventilation par tunnel)
    /// doivent survivre au cycle à l'identique.
    #[test]
    fn pcap_matrix_survives_csv_roundtrip() {
        let pcap_path = tunnel_pcap();
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();
        let on_event = Channel::new(|_| Ok(()));
        handle_pcap_file(
            pcap_path.to_str().unwrap(),
            &mut matrix,
            &mut graph,
            &on_event,
            &mut None,
        )
        .unwrap();

        let dir = TempDir::new("sonar_test_matrix_roundtrip");
        let csv_path = dir.path().join("matrice.csv");
        FlowMatrix::write_rows_to_csv(
            &matrix.to_flat_vec(),
            matrix.link_type,
            csv_path.to_str().unwrap(),
        )
        .unwrap();

        let rows = read_matrix_rows(csv_path.to_str().unwrap()).unwrap();
        assert_eq!(rows.len(), matrix.row_count(), "une ligne CSV par flux");

        let (reimported, _graph) = build_matrix_and_graph(&rows);
        let normalize = |m: &FlowMatrix| {
            let mut v: Vec<String> = m
                .to_flat_vec()
                .into_iter()
                .map(|r| {
                    format!(
                        "{}|{}|{:?}|{:?}|{}|{}|{}",
                        r.mac_source,
                        r.mac_destination,
                        r.port_source,
                        r.port_destination,
                        r.count,
                        r.total_bytes,
                        r.encap_id
                    )
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(
            normalize(&matrix),
            normalize(&reimported),
            "compteurs et ventilation par tunnel identiques après réimport"
        );
    }

    /// Rejoue la chaîne de `import_matrix_file` sur le fichier réel de
    /// `test_files/` : lecture CSV, reconstruction des flux, remplissage de la
    /// matrice et du graphe.
    #[test]
    fn import_real_matrix_file_rebuilds_matrix_and_graph() {
        let matrix_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_NP_Matrice.csv")
            .to_str()
            .unwrap()
            .to_string();

        let rows = read_matrix_rows(&matrix_path).unwrap();
        assert_eq!(
            rows.len(),
            30,
            "30 lignes de données (l'en-tête est écarté)"
        );

        let (matrix, graph) = build_matrix_and_graph(&rows);

        assert_eq!(matrix.row_count(), 30, "un flux par ligne du fichier");
        assert!(!graph.nodes.is_empty(), "le graphe doit contenir des nœuds");
        assert!(
            !graph.edges.is_empty(),
            "le graphe doit contenir des arêtes"
        );

        // Les labels du fichier sont réappliqués sur les nœuds du graphe.
        let labelled = graph
            .nodes
            .values()
            .filter(|n| n.label.as_deref() == Some("pc sonar"))
            .count();
        assert!(
            labelled > 0,
            "au moins un nœud doit porter le label du fichier"
        );

        // Le CSV survit à un aller-retour export -> import (mêmes flux).
        let reexported = matrix.to_flat_vec();
        assert_eq!(reexported.len(), 30);
    }

    /// Même chaîne sur la matrice générée de 1000 lignes : vérifie la tenue
    /// en charge du parseur et la construction d'un graphe complet.
    #[test]
    fn import_1000_row_matrix_builds_full_graph() {
        let matrix_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_NP_Matrice_1000.csv")
            .to_str()
            .unwrap()
            .to_string();

        let rows = read_matrix_rows(&matrix_path).unwrap();
        assert_eq!(rows.len(), 1000);

        let (matrix, graph) = build_matrix_and_graph(&rows);

        assert_eq!(matrix.row_count(), 1000, "1000 flux distincts");
        assert_eq!(graph.nodes.len(), 232, "un nœud par adresse IP distincte");
        assert!(graph.edges.len() > 100, "arêtes: {}", graph.edges.len());

        // Les poids de trafic du fichier sont reportés sur les arêtes.
        assert!(
            graph
                .edges
                .values()
                .all(|e| e.count > 0 && e.total_bytes > 0),
            "chaque arête doit porter son trafic cumulé"
        );

        let labelled = graph.nodes.values().filter(|n| n.label.is_some()).count();
        assert!(labelled >= 20, "nœuds labellisés: {labelled}");
    }
}
