//! Import des matrices de flux CSV ou XLSX (schéma de
//! `FlowMatrix::export_to_csv`) : lecture, fusion multi-fichiers avec
//! provenance, reconstruction de la matrice et du graphe.

use calamine::{Data, DataType, Range, Reader, Xlsx, open_workbook};
use log::{error, info, warn};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use sonar_flows_core::csv::{apply_row_labels, merge_rows};

use crate::{
    errors::{CaptureStateError, import::PcapImportError},
    events::CaptureEvent,
    state::{
        capture::CaptureState,
        flow_matrix::{FlowMatrix, FlowMatrixRow, unescape_formula_cell},
        graph::GraphData,
        labels_list::LabelStore,
    },
};

use super::{event_channel, labels::copy_labels_to_matrix};

type MatrixRowsByFile = Vec<(String, Vec<FlowMatrixRow>)>;
type MatrixBatch = (packet_parser::LinkType, MatrixRowsByFile);

#[tauri::command]
pub fn is_matrix_empty(
    state: tauri::State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<bool, CaptureStateError> {
    Ok(state.lock()?.matrix.is_empty())
}

/// Lit une matrice de flux (format de `FlowMatrix::export_to_csv`),
/// entièrement validé avant de toucher à l'état — lecture et validation
/// stricte (#148). La production passe par
/// `read_matrix_rows_per_file` ; ce raccourci ne sert plus qu'aux tests.
#[cfg(test)]
fn read_matrix_rows(path: &str) -> Result<Vec<FlowMatrixRow>, CaptureStateError> {
    Ok(read_matrix_file(Path::new(path))?.1)
}

fn invalid_matrix(path: &Path, message: impl Into<String>) -> CaptureStateError {
    sonar_flows_core::SonarCoreError::InvalidCsv {
        path: path.to_path_buf(),
        message: message.into(),
    }
    .into()
}

fn is_xlsx(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xlsx"))
}

/// Convertit une cellule Excel en texte CSV. Les dates réellement typées par
/// Excel sont normalisées dans le format SFMS accepté par `last_seen` ; les
/// autres cellules gardent leur représentation textuelle.
fn xlsx_cell_text(cell: &Data, header: &str) -> String {
    if header == "last_seen"
        && let Some(date) = cell.as_datetime()
    {
        return date.format("%Y-%m-%d %H:%M:%S%.6f UTC").to_string();
    }
    cell.to_string()
}

/// Lit la première feuille d'un classeur XLSX. La première ligne non vide est
/// soit le préambule `#SFMS`, soit directement l'en-tête. On repasse ensuite
/// par le désérialiseur CSV et la validation métier existants afin que CSV et
/// XLSX appliquent exactement le même schéma, y compris l'ignorance des
/// colonnes supplémentaires.
fn read_xlsx_matrix(
    path: &Path,
) -> Result<(packet_parser::LinkType, Vec<FlowMatrixRow>), CaptureStateError> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|error| invalid_matrix(path, format!("ouverture XLSX impossible: {error}")))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| invalid_matrix(path, "le classeur XLSX ne contient aucune feuille"))?
        .map_err(|error| invalid_matrix(path, format!("lecture XLSX impossible: {error}")))?;

    parse_xlsx_range(path, &range)
}

fn parse_xlsx_range(
    path: &Path,
    range: &Range<Data>,
) -> Result<(packet_parser::LinkType, Vec<FlowMatrixRow>), CaptureStateError> {
    let non_empty_rows: Vec<&[Data]> = range
        .rows()
        .filter(|row| row.iter().any(|cell| !cell.is_empty()))
        .collect();
    let first = non_empty_rows
        .first()
        .ok_or_else(|| invalid_matrix(path, "la première feuille XLSX est vide"))?;
    let first_cell = first.first().map(ToString::to_string).unwrap_or_default();
    let (link_type, header_index) = match sonar_flows_core::sfms::parse_preamble(&first_cell)
        .map_err(|message| invalid_matrix(path, message))?
    {
        Some(preamble) => (
            preamble
                .link_type
                .unwrap_or(packet_parser::LinkType::ETHERNET),
            1,
        ),
        None => (packet_parser::LinkType::ETHERNET, 0),
    };
    let header_row = non_empty_rows
        .get(header_index)
        .ok_or_else(|| invalid_matrix(path, "en-tête de matrice XLSX absent"))?;
    let headers: Vec<String> = header_row
        .iter()
        .map(|cell| cell.to_string().trim().to_string())
        .collect();

    let mut csv_data = Vec::new();
    {
        let mut writer = csv::WriterBuilder::new().from_writer(&mut csv_data);
        writer.write_record(&headers).map_err(|error| {
            invalid_matrix(path, format!("conversion XLSX impossible: {error}"))
        })?;
        for row in non_empty_rows.iter().skip(header_index + 1) {
            let record = headers.iter().enumerate().map(|(column, header)| {
                row.get(column)
                    .map(|cell| xlsx_cell_text(cell, header))
                    .unwrap_or_default()
            });
            writer.write_record(record).map_err(|error| {
                invalid_matrix(path, format!("conversion XLSX impossible: {error}"))
            })?;
        }
        writer.flush().map_err(|error| {
            invalid_matrix(path, format!("conversion XLSX impossible: {error}"))
        })?;
    }

    let first_data_line = header_index + 2;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_slice());
    let mut rows = Vec::new();
    for (index, result) in reader.deserialize::<FlowMatrixRow>().enumerate() {
        let row = result.map_err(|error| {
            invalid_matrix(
                path,
                format!("ligne {} invalide: {error}", index + first_data_line),
            )
        })?;
        row.validate(link_type).map_err(|error| {
            invalid_matrix(
                path,
                format!("ligne {} invalide: {error}", index + first_data_line),
            )
        })?;
        rows.push(row);
    }
    Ok((link_type, rows))
}

fn read_matrix_file(
    path: &Path,
) -> Result<(packet_parser::LinkType, Vec<FlowMatrixRow>), CaptureStateError> {
    if is_xlsx(path) {
        read_xlsx_matrix(path)
    } else {
        Ok((
            sonar_flows_core::sfms::matrix_file_link_type(path)?,
            sonar_flows_core::csv::read_matrix_rows(path)?,
        ))
    }
}

/// Lit chaque fichier de matrice et retourne ses lignes groupées par fichier,
/// pour la comptabilité par fichier (événements `Finished`). La lecture et
/// l'héritage de la colonne `origin` viennent du cœur partagé. Le callback est
/// appelé après validation complète de chaque fichier pour publier sa
/// progression sans exposer de lignes partielles.
fn read_matrix_rows_per_file(
    incoming_file_paths: &[String],
    mut on_file_read: impl FnMut(usize, &str, usize),
) -> Result<MatrixBatch, CaptureStateError> {
    if incoming_file_paths.is_empty() {
        return Err(
            PcapImportError::MissingInput("l'import de matrice CSV/XLSX".to_string()).into(),
        );
    }

    let mut files = Vec::with_capacity(incoming_file_paths.len());
    let mut common_link_type = None;
    for (index, path) in incoming_file_paths.iter().enumerate() {
        let path_buf = PathBuf::from(path);
        let origin = sonar_flows_core::csv::origin_name_from_path(&path_buf);
        let (link_type, mut rows) = read_matrix_file(&path_buf)?;
        match common_link_type {
            None => common_link_type = Some(link_type),
            Some(expected) if expected == link_type => {}
            Some(expected) => {
                return Err(sonar_flows_core::SonarCoreError::MixedLinkTypes {
                    path: path_buf,
                    found: sonar_flows_core::sfms::link_type_name(link_type),
                    expected: sonar_flows_core::sfms::link_type_name(expected),
                }
                .into());
            }
        }
        for row in &mut rows {
            if row.origin.trim().is_empty() {
                row.origin = origin.clone();
            }
        }
        on_file_read(index, path, rows.len());
        files.push((path.clone(), rows));
    }
    let link_type = common_link_type
        .ok_or_else(|| PcapImportError::MissingInput("l'import de matrice CSV/XLSX".to_string()))?;
    Ok((link_type, files))
}

// La production passe par `read_matrix_rows_per_file` (comptabilité par
// fichier) ; cette version aplatie ne sert plus qu'aux tests.
#[cfg(test)]
fn read_matrix_rows_from_files(
    incoming_file_paths: &[String],
) -> Result<Vec<FlowMatrixRow>, CaptureStateError> {
    Ok(
        read_matrix_rows_per_file(incoming_file_paths, |_, _, _| {})?
            .1
            .into_iter()
            .flat_map(|(_, rows)| rows)
            .collect(),
    )
}

/// Contexte de relevé commun à des fichiers de matrice (#154).
///
/// Le contexte du préambule `#SFMS` décrit **un** point de mesure. Fusionner
/// des relevés qui n'ont pas le même — deux switchs, deux armoires — ne peut
/// donc pas produire un contexte unique honnête : on retourne alors un
/// contexte vide plutôt que d'élire arbitrairement celui du premier fichier
/// et d'étiqueter tout le reste avec.
///
/// Attribuer chaque flux à sa provenance demande le contexte **par ligne**
/// (généralisation d'`origin`), qui n'est pas encore livré. Jusque-là, un
/// contexte vide est la seule réponse qui ne ment pas.
fn common_survey_context(paths: &[String]) -> sonar_flows_core::sfms::SurveyContext {
    let mut common: Option<sonar_flows_core::sfms::SurveyContext> = None;
    for path in paths {
        let context = sonar_flows_core::sfms::read_preamble(std::path::Path::new(path))
            .ok()
            .flatten()
            .map(|preamble| preamble.context)
            .unwrap_or_default();
        match &common {
            None => common = Some(context),
            Some(expected) if *expected == context => {}
            Some(_) => {
                warn!(
                    "[import_matrix_files] contextes de relevé divergents entre fichiers : \
                     le relevé fusionné reste non qualifié"
                );
                return sonar_flows_core::sfms::SurveyContext::default();
            }
        }
    }
    common.unwrap_or_default()
}

fn rebuild_matrix_and_graph_from_rows(
    rows: &[FlowMatrixRow],
    link_type: packet_parser::LinkType,
    label_store: &LabelStore,
    matrice: &mut FlowMatrix,
    graph: &mut GraphData,
) -> Result<(), CaptureStateError> {
    matrice.clear();
    graph.clear();
    matrice.link_type = Some(link_type);

    // Labels du store courant, puis ceux portés par les fichiers (prioritaires
    // à clé égale puisque appliqués après) ; fusion des flux tunnel par tunnel
    // et provenance accumulée par le cœur partagé.
    copy_labels_to_matrix(label_store, matrice)?;
    apply_row_labels(matrice, rows);
    merge_rows(matrice, rows, link_type)?;

    for row in rows {
        let (flow, tunnel_rows) = row
            .to_flow_and_rows(link_type)
            .map_err(sonar_flows_core::SonarCoreError::InvalidMatrixRow)?;
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
    // Refus avant la réservation, la sélection du channel, la lecture d'un
    // label et tout événement : une liste vide ne doit jamais vider la
    // session courante (#167).
    if incoming_file_paths.is_empty() {
        return Err(
            PcapImportError::MissingInput("l'import de matrice CSV/XLSX".to_string()).into(),
        );
    }

    // Import et capture sont mutuellement exclusifs : l'import remplacerait
    // la matrice et le graphe pendant que le pipeline les alimente. La phase
    // `Importing` est réservée atomiquement et détenue jusqu'à la fin de la
    // commande, swap inclus (#139).
    // Même garantie que pour les PCAP : une matrice fusionnée deux fois
    // doublerait ses compteurs et dupliquerait sa provenance (#161).
    let incoming_file_paths = super::dedupe_paths_by_identity(&incoming_file_paths);

    let import_guard = crate::state::capture::ImportGuard::acquire(
        capture_state.inner(),
        "import de matrice CSV/XLSX",
    )?;
    let on_event = event_channel(&capture_state, on_event)?;

    info!(
        "[import_matrix_files] COMMAND CALLED avec {} fichier(s): {:?}",
        incoming_file_paths.len(),
        incoming_file_paths
    );

    // Un fichier invalide ne doit pas effacer la matrice courante. Les
    // fichiers doivent porter le même DLT (préambule #SFMS, Ethernet
    // implicite pour un export antérieur) : fusion inter-DLT refusée. Chaque
    // classeur XLSX n'est ouvert qu'une fois, même pour les gros relevés.
    let files_total = incoming_file_paths.len();
    let (link_type, files) =
        read_matrix_rows_per_file(&incoming_file_paths, |index, path, line_count| {
            if let Err(e) = on_event.send(CaptureEvent::ImportProgress {
                file_name: path,
                file_index: index + 1,
                files_total,
                current: line_count,
                total: line_count,
            }) {
                error!("Erreur lors de l'envoi de ImportProgress: {:?}", e);
            }
        })?;

    // Même principe que l'import PCAP (`send_started_event`) : un import de
    // matrice CSV/XLSX est aussi une session, avec sa propre version de contrat
    // IPC (#142) — sans quoi ce chemin n'émettait jamais `Started` et le
    // frontend ne pouvait ni afficher le DLT ni détecter une dérive de
    // version pour cette voie d'import.
    if let Err(e) = on_event.send(CaptureEvent::Started {
        session_id: 0,
        device: "",
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
        snaplen: 0,
        link_type: &sonar_flows_core::sfms::link_type_name(link_type),
        protocol_version: crate::events::CAPTURE_EVENT_PROTOCOL_VERSION,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    }

    let line_counts: Vec<(String, usize)> = files
        .iter()
        .map(|(path, rows)| (path.clone(), rows.len()))
        .collect();
    let rows: Vec<FlowMatrixRow> = files.into_iter().flat_map(|(_, rows)| rows).collect();

    // Snapshot transactionnel local : labels, matrice et graphe sont bâtis
    // sans toucher à l'état partagé. Ils seront remplacés ensemble seulement
    // si la réservation qui a lu ces fichiers est encore courante.
    let mut new_label_store = LabelStore {
        rows: label_store.lock()?.get().clone(),
    };

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
                new_label_store.set(mac, ip, &unescape_formula_cell(label));
            }
        }
    }

    let mut new_matrix = FlowMatrix::new();
    let mut new_graph = GraphData::new();
    // Le contexte du relevé vient des fichiers importés, jamais de la session
    // précédente (#154) : `new_matrix` part d'un contexte vide, donc importer
    // ne peut pas rhabiller un relevé avec la qualification de l'ancien.
    new_matrix.context = common_survey_context(&incoming_file_paths);
    rebuild_matrix_and_graph_from_rows(
        &rows,
        link_type,
        &new_label_store,
        &mut new_matrix,
        &mut new_graph,
    )?;

    import_guard.verify_current("commit de l'import de matrice")?;

    // Le commit est certain à partir d'ici (le swap est une pure
    // affectation) : relevé marqué modifié avant de prendre les verrous de
    // données, `CaptureState` ne s'imbrique jamais avec eux (#166, #159).
    capture_state.lock()?.mark_dirty();

    // Même ordre de verrouillage que convert_from_pcap_list et net_capture
    // (matrice -> graph -> label_store) pour éviter un interblocage ABBA.
    let mut matrice_guard = matrice.lock()?;
    let mut graph_guard = graph.lock()?;
    let mut label_store_guard = label_store.lock()?;
    *matrice_guard = new_matrix;
    *graph_guard = new_graph;
    *label_store_guard = new_label_store;

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
            // Une matrice CSV/XLSX est validée ligne à ligne avant import : une
            // ligne invalide est fatale (#148), donc tout ce qui est lu est
            // intégré.
            integrated_count: *line_count,
            parse_error_count: 0,
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
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    fn matrix_invoke_request(paths: Vec<String>, channel_id: u32) -> InvokeRequest {
        InvokeRequest {
            cmd: "import_matrix_files".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(any(windows, target_os = "android")) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(serde_json::json!({
                "incomingFilePaths": paths,
                "onEvent": format!("__CHANNEL__:{channel_id}"),
            })),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    #[test]
    fn new_matrix_is_empty() {
        let matrix = FlowMatrix::new();
        assert!(matrix.matrix.is_empty());
    }

    #[test]
    fn xlsx_range_accepts_an_extra_column_in_the_middle() {
        let source_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        let expected = sonar_flows_core::csv::read_matrix_rows(&source_path)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        // Réutilise la sérialisation canonique pour obtenir les en-têtes et
        // insère une vraie colonne cliente au milieu de la feuille.
        let mut serialized = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut serialized);
            writer.serialize(&expected).unwrap();
            writer.flush().unwrap();
        }
        let mut reader = csv::Reader::from_reader(serialized.as_slice());
        let mut headers: Vec<String> = reader
            .headers()
            .unwrap()
            .iter()
            .map(str::to_string)
            .collect();
        let mut values: Vec<String> = reader
            .records()
            .next()
            .unwrap()
            .unwrap()
            .iter()
            .map(str::to_string)
            .collect();
        headers.insert(6, "commentaire_client".to_string());
        values.insert(6, "ajouté dans Excel".to_string());

        let last_column = u32::try_from(headers.len() - 1).unwrap();
        let mut range = Range::new((0, 0), (1, last_column));
        for (column, header) in headers.into_iter().enumerate() {
            range.set_value((0, u32::try_from(column).unwrap()), Data::String(header));
        }
        for (column, value) in values.into_iter().enumerate() {
            range.set_value((1, u32::try_from(column).unwrap()), Data::String(value));
        }

        let (link_type, rows) = parse_xlsx_range(Path::new("client.xlsx"), &range).unwrap();
        assert_eq!(link_type, packet_parser::LinkType::ETHERNET);
        assert_eq!(rows, vec![expected]);
    }

    /// Reproduit la construction de `import_matrix_file` (labels puis flux).
    fn build_matrix_and_graph(rows: &[FlowMatrixRow]) -> (FlowMatrix, GraphData) {
        build_matrix_and_graph_for_link_type(rows, packet_parser::LinkType::ETHERNET)
    }

    fn build_matrix_and_graph_for_link_type(
        rows: &[FlowMatrixRow],
        link_type: packet_parser::LinkType,
    ) -> (FlowMatrix, GraphData) {
        let mut matrix = FlowMatrix::new();
        let mut graph = GraphData::new();
        let label_store = LabelStore::new();
        rebuild_matrix_and_graph_from_rows(rows, link_type, &label_store, &mut matrix, &mut graph)
            .unwrap();

        (matrix, graph)
    }

    /// L'adaptateur desktop transmet le DLT du préambule à la matrice ET au
    /// graphe : un CSV SLL ne doit jamais être reconstruit comme Ethernet.
    #[test]
    fn sll_matrix_rebuild_keeps_typed_keys_and_graph() {
        let rows = read_matrix_rows(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test_files/20260703_NP_Matrice.csv")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let mut row = rows
            .into_iter()
            .find(|row| {
                !row.ip_source.is_empty()
                    && !row.ip_destination.is_empty()
                    && sonar_flows_core::link::cooked_protocol_from_text(&row.protocol_data_link)
                        .is_some()
            })
            .expect("la fixture contient un flux IP");
        row.mac_destination.clear();
        row.vlan_id = None;

        let (matrix, graph) =
            build_matrix_and_graph_for_link_type(&[row], packet_parser::LinkType::LINUX_SLL);

        assert_eq!(matrix.link_type, Some(packet_parser::LinkType::LINUX_SLL));
        assert!(
            matrix
                .matrix
                .keys()
                .all(|flow| { flow.data_link.link_type() == packet_parser::LinkType::LINUX_SLL })
        );
        assert!(!graph.nodes.is_empty());
        assert!(!graph.edges.is_empty());
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

        let (flow, _tunnel_rows) = rows[0]
            .to_flow_and_rows(packet_parser::LinkType::ETHERNET)
            .unwrap();
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

    #[test]
    fn matrix_read_reports_each_validated_file() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        let dir = TempDir::new("sonar_test_matrix_progress");
        let file_a = dir.path().join("site-a.csv");
        let file_b = dir.path().join("site-b.csv");
        fs::copy(&source, &file_a).unwrap();
        fs::copy(&source, &file_b).unwrap();
        let paths = vec![
            file_a.to_str().unwrap().to_string(),
            file_b.to_str().unwrap().to_string(),
        ];
        let mut progress = Vec::new();

        let (_link_type, files) = read_matrix_rows_per_file(&paths, |index, path, rows| {
            progress.push((index, path.to_string(), rows));
        })
        .unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(progress.len(), 2);
        assert_eq!(progress[0].0, 0);
        assert_eq!(progress[0].1, paths[0]);
        assert_eq!(progress[0].2, files[0].1.len());
        assert_eq!(progress[1].0, 1);
        assert_eq!(progress[1].1, paths[1]);
        assert_eq!(progress[1].2, files[1].1.len());
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
            1,
            1,
            &mut matrix,
            &mut graph,
            &on_event,
            &mut None,
            &std::sync::atomic::AtomicBool::new(false),
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

    /// Même garantie que la voie PCAP : l'appel IPC avec `[]` échoue avant
    /// tout événement, lecture de labels ou remplacement partagé (#167).
    #[test]
    fn tauri_matrix_command_refuses_empty_list_without_touching_session() {
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        let rows = read_matrix_rows(source.to_str().unwrap()).unwrap();
        let (seeded_matrix, seeded_graph) = build_matrix_and_graph(&rows);
        let mut seeded_labels = LabelStore::new();
        seeded_labels.add((
            "aa:bb:cc:dd:ee:ff".to_string(),
            "198.51.100.9".to_string(),
            "label sentinelle".to_string(),
        ));
        let mut seeded_capture_state = CaptureState::new();
        seeded_capture_state.session_id = 73;
        seeded_capture_state.filter = Some("udp".to_string());
        seeded_capture_state.config.device_name = "matrice-sentinelle".to_string();

        let matrix = Arc::new(Mutex::new(seeded_matrix));
        let graph = Arc::new(Mutex::new(seeded_graph));
        let labels = Arc::new(Mutex::new(seeded_labels));
        let capture_state = Arc::new(Mutex::new(seeded_capture_state));
        let before_matrix = matrix.lock().unwrap().to_flat_vec();
        let before_graph = serde_json::to_value(&*graph.lock().unwrap()).unwrap();
        let before_labels = labels.lock().unwrap().rows.clone();
        let before_config = serde_json::to_value(&capture_state.lock().unwrap().config).unwrap();
        let events = Arc::new(Mutex::new(Vec::<serde_json::Value>::new()));
        let intercepted_events = Arc::clone(&events);

        let app = mock_builder()
            .channel_interceptor(move |_webview, _callback, _index, body| {
                if let InvokeResponseBody::Json(json) = body {
                    intercepted_events
                        .lock()
                        .unwrap()
                        .push(serde_json::from_str(json).unwrap());
                }
                true
            })
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&labels))
            .manage(Arc::clone(&capture_state))
            .invoke_handler(tauri::generate_handler![import_matrix_files])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let error = get_ipc_response(&webview, matrix_invoke_request(vec![], 167))
            .expect_err("la liste de matrices vide doit être refusée");

        assert_eq!(error["kind"], "import");
        assert_eq!(error["message"]["kind"], "missingInput");
        assert!(events.lock().unwrap().is_empty(), "aucun événement émis");
        assert_eq!(matrix.lock().unwrap().to_flat_vec(), before_matrix);
        assert_eq!(
            serde_json::to_value(&*graph.lock().unwrap()).unwrap(),
            before_graph
        );
        assert_eq!(labels.lock().unwrap().rows, before_labels);
        let locked_state = capture_state.lock().unwrap();
        assert_eq!(
            locked_state.phase,
            crate::state::capture::capture_status::CapturePhase::Idle
        );
        assert_eq!(locked_state.session_id, 73);
        assert_eq!(locked_state.filter.as_deref(), Some("udp"));
        assert_eq!(
            serde_json::to_value(&locked_state.config).unwrap(),
            before_config
        );
    }

    /// Le contexte d'un relevé importé vient du fichier, et de lui seul
    /// (#154). Un import ne doit jamais rhabiller les données entrantes avec
    /// la qualification de la session précédente.
    #[test]
    fn survey_context_comes_from_the_imported_file() {
        let dir = TempDir::new("import_context");
        let path = dir.path().join("relevé switch 7.csv");
        let context = sonar_flows_core::sfms::SurveyContext {
            site: Some("Usine Nord".to_string()),
            sensor: Some("PC de relevé".to_string()),
            interface: Some("eth0".to_string()),
        };
        FlowMatrix::write_rows_to_csv_with_context(
            &[],
            Some(packet_parser::LinkType::ETHERNET),
            &context,
            &path.to_string_lossy(),
        )
        .unwrap();

        let read = common_survey_context(&[path.to_string_lossy().into_owned()]);
        assert_eq!(read, context, "le contexte du préambule doit être rétabli");
    }

    /// Fusionner deux relevés de points de mesure différents ne peut pas
    /// produire un contexte unique honnête : le résultat reste non qualifié
    /// plutôt que d'hériter arbitrairement du premier fichier.
    ///
    /// Attribuer chaque flux à sa provenance demande le contexte par ligne,
    /// pas encore livré.
    #[test]
    fn merging_two_measurement_points_yields_no_context() {
        let dir = TempDir::new("import_context_conflict");
        let mut paths = Vec::new();
        for (index, site) in ["Armoire B3", "Armoire C1"].iter().enumerate() {
            let path = dir.path().join(format!("relevé {index}.csv"));
            let context = sonar_flows_core::sfms::SurveyContext {
                site: Some((*site).to_string()),
                sensor: Some("PC de relevé".to_string()),
                interface: Some("eth0".to_string()),
            };
            FlowMatrix::write_rows_to_csv_with_context(
                &[],
                Some(packet_parser::LinkType::ETHERNET),
                &context,
                &path.to_string_lossy(),
            )
            .unwrap();
            paths.push(path.to_string_lossy().into_owned());
        }

        assert!(
            common_survey_context(&paths).is_empty(),
            "des contextes divergents doivent donner un relevé non qualifié, \
             pas celui du premier fichier"
        );
    }

    /// Deux fichiers du même point de mesure gardent leur contexte : la règle
    /// ci-dessus ne doit pas punir une fusion légitime (un même relevé
    /// découpé en plusieurs exports).
    #[test]
    fn merging_the_same_measurement_point_keeps_the_context() {
        let dir = TempDir::new("import_context_same");
        let context = sonar_flows_core::sfms::SurveyContext {
            site: Some("Armoire B3".to_string()),
            sensor: Some("PC de relevé".to_string()),
            interface: Some("eth0".to_string()),
        };
        let mut paths = Vec::new();
        for index in 0..2 {
            let path = dir.path().join(format!("part {index}.csv"));
            FlowMatrix::write_rows_to_csv_with_context(
                &[],
                Some(packet_parser::LinkType::ETHERNET),
                &context,
                &path.to_string_lossy(),
            )
            .unwrap();
            paths.push(path.to_string_lossy().into_owned());
        }

        assert_eq!(common_survey_context(&paths), context);
    }
}
