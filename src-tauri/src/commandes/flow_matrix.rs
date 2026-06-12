use std::sync::{Arc, Mutex};
use tauri::{State, command};

use crate::{
    errors::CaptureStateError,
    events::CaptureEvent,
    state::{capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData},
};
// si tu veux un Result typé :

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
