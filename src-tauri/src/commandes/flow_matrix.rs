//! Commandes de gestion des labels de la matrice de flux (ajout manuel,
//! consultation).

use std::sync::{Arc, Mutex};
use tauri::{State, command};

use crate::{
    errors::CaptureStateError,
    events::CaptureEvent,
    state::{capture::CaptureState, flow_matrix::FlowMatrix, graph::GraphData},
};

/// Applique un label à la clé `(mac, ip)` : matrice de flux, nœud du graphe,
/// puis notification du frontend via le channel de la capture s'il existe.
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

/// Liste des labels connus de la matrice (valeurs seules, pour l'UI).
#[command]
pub fn get_label_list(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<Vec<String>, CaptureStateError> {
    let guard = matrix.lock()?;
    Ok(guard.get_label_list())
}

/// Labels réellement appliqués à la matrice de flux, au format `(mac, ip, label)`,
/// avec les champs manquants (MAC ou IP) complétés depuis la matrice — même
/// vue que l'export de labels.
#[command]
pub fn get_matrix_labels(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<Vec<(String, String, String)>, CaptureStateError> {
    let guard = matrix.lock()?;
    Ok(guard.export_labels())
}
