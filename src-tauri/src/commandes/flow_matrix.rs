use std::sync::{Arc, Mutex};
use serde::Serialize;
use tauri::{State, command};

use crate::{
    errors::CaptureStateError,
    events::CaptureEvent,
    setup::labels::parse_label_csv,
    state::{capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData},
};
// si tu veux un Result typé :

#[derive(Serialize)]
pub struct LabelCsvImportResult {
    pub imported: usize,
    pub graph_data: GraphData,
}

#[command]
pub fn add_label(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    mac: String,
    ip: String,
    label: String,
) -> Result<(), CaptureStateError> {
    {
        let mut guard = matrix.lock()?;
        guard.add_label(mac.clone(), ip.clone(), label.clone());
    }

    let graph_update = {
        let mut guard = graph.lock()?;
        guard.update_node_label(&mac, &ip, label)
    };

    let event_channel = {
        let guard = capture_state.lock()?;
        guard.on_event.clone()
    };

    if let (Some(update), Some(on_event)) = (graph_update, event_channel)
        && let Err(error) = on_event.send(CaptureEvent::Graph { update: &update })
    {
        eprintln!("Erreur d'envoi du GraphUpdate label: {error}");
    }

    Ok(())
}

#[command]
pub fn get_label_list(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<Vec<String>, CaptureStateError> {
    let guard = matrix.lock()?;
    Ok(guard.get_label_list())
}

#[command]
pub fn import_label_csv(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    path: String,
) -> Result<LabelCsvImportResult, String> {
    let csv_data = std::fs::read_to_string(&path)
        .map_err(|error| format!("Impossible de lire le CSV labels : {error}"))?;
    let labels = parse_label_csv(&csv_data)?;
    let imported = labels.len();

    let graph_data = {
        let mut matrix_guard = matrix.lock().map_err(|error| error.to_string())?;
        matrix_guard.replace_labels(labels);

        let mut graph_guard = graph.lock().map_err(|error| error.to_string())?;
        graph_guard.apply_labels(|mac, ip| matrix_guard.get_label(mac, ip));
        graph_guard.get_all_graph_data()
    };

    Ok(LabelCsvImportResult {
        imported,
        graph_data,
    })
}
