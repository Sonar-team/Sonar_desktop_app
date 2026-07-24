//! Commandes Tauri d'import/gestion des labels : lecture du store, import de
//! fichier CSV (avec résolution des conflits), arbitrage, et synchronisation
//! avec la matrice de flux.

use log::{error, info};
use std::sync::{Arc, Mutex};
use tauri::{State, ipc::Channel};

use crate::{
    errors::CaptureStateError,
    events::CaptureEvent,
    state::{
        capture::CaptureState,
        flow_matrix::FlowMatrix,
        graph::GraphData,
        labels_list::{LabelStore, PcInfoLabel},
    },
};

use super::super::event_channel;
use super::conflicts::{
    LabelConflictReport, LabelConflictStore, LabelImportReport, LabelResolution,
    dedup_labels_first_wins,
};
use super::csv::{is_header_row, read_label_rows};
use super::validation::verif_mac_ip_format_rows;

#[tauri::command(async)]
pub fn get_label_rows(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<Vec<(String, String, String)>, CaptureStateError> {
    let label_store = label_store.lock()?;
    Ok(label_store.get().clone())
}

/// Vide le store de labels. Le store étant la source de vérité unique
/// (#157), la matrice et le graphe sont resynchronisés : tous les
/// équipements sont désétiquetés (seuls les labels « pc sonar » subsistent).
#[tauri::command(async)]
pub fn clear_label_store(
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: State<'_, Arc<Mutex<PcInfoLabel>>>,
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<Vec<crate::state::graph::GraphUpdate>, CaptureStateError> {
    label_store.lock()?.clear();
    crate::commandes::flow_matrix::resync_and_notify(
        &matrix,
        &graph,
        &label_store,
        &pcinfo,
        &capture_state,
    )
}

/// Importe un fichier de labels CSV.
///
/// Le store est la source de vérité unique (#157) : il est remplacé par le
/// contenu du fichier puis matrice et graphe sont resynchronisés — un label
/// absent du fichier disparaît (l'ancienne sémantique « un import ne
/// désétiquette jamais » est abandonnée, la table du panneau EST l'état des
/// labels). Les labels « pc sonar » générés au démarrage subsistent.
#[allow(clippy::too_many_arguments)]
#[tauri::command(async)]
pub fn import_label_file(
    incoming_file_path: String,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: State<'_, Arc<Mutex<PcInfoLabel>>>,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<LabelImportReport, CaptureStateError> {
    let on_event = event_channel(&capture_state, on_event)?;

    // Lecture unique du fichier (#153) ; les erreurs de FORMAT restent
    // bloquantes et précèdent toute modification du store. Les conflits de
    // labels, eux, ne bloquent pas : on garde le premier label par clé
    // (mac, ip) et on enregistre les doublons écartés pour arbitrage.
    let mut rows = read_label_rows(&incoming_file_path)?;
    verif_mac_ip_format_rows(&rows)?;

    // Même principe que les imports PCAP/matrice : un `Started` en tête
    // pour porter `protocol_version` (#142). Sans DLT propre à un import de
    // labels (il ne touche pas au parsing de paquets), `link_type` reste
    // vide plutôt qu'une valeur inventée.
    if let Err(e) = on_event.send(CaptureEvent::Started {
        session_id: 0,
        device: "",
        buffer_size: 0,
        chan_capacity: 0,
        timeout: 0,
        snaplen: 0,
        link_type: "",
        protocol_version: crate::events::CAPTURE_EVENT_PROTOCOL_VERSION,
    }) {
        error!("Erreur lors de l'envoi de Started: {:?}", e);
    }

    let conflicts = {
        let mut label_store = label_store.lock()?;
        label_store.clear();

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
        // Resynchronisation complète (#157) : le store est LA source de
        // vérité, le miroir de la matrice est remplacé — un label absent du
        // fichier importé disparaît (fin de « un import ne désétiquette
        // jamais »). Les nœuds du graphe sont rafraîchis et notifiés un par
        // un : pas de snapshot complet, la disposition UI est préservée.
        let mut state_label = state_label.lock()?;
        let updates = {
            let mut graph_guard = graph.lock()?;
            let store_guard = label_store.lock()?;
            let pcinfo_guard = pcinfo.lock()?;
            crate::setup::labels::resync_labels(
                &pcinfo_guard,
                &store_guard,
                &mut state_label,
                &mut graph_guard,
            )
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
pub fn resolve_label_conflicts(
    resolutions: Vec<LabelResolution>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<LabelConflictStore>>>,
) -> Result<Vec<LabelConflictReport>, CaptureStateError> {
    // `CaptureState` n'est jamais imbriqué avec un état de données (#166).
    let on_event = capture_state.lock()?.on_event.clone();

    // Application transactionnelle (#153) : tous les verrous sont pris une
    // fois puis chaque arbitrage est appliqué dans la même section critique —
    // l'ancien appel par conflit depuis le front pouvait s'interrompre à
    // mi-chemin et laisser un état partiel.
    let (remaining, graph_updates) = {
        // Ordre global : matrice → graphe → labels → stores auxiliaires.
        let mut matrix = matrix.lock()?;
        let mut graph = graph.lock()?;
        let mut labels = label_store.lock()?;
        let mut store = conflict_store.lock()?;
        let mut graph_updates = Vec::new();

        for resolution in &resolutions {
            // Applique le label choisi à la matrice (résolution des flux) et
            // au store (table + réexport).
            matrix.add_label(
                resolution.mac.clone(),
                resolution.ip.clone(),
                resolution.label.clone(),
            );
            labels.set(&resolution.mac, &resolution.ip, &resolution.label);

            // Rafraîchit le nœud sans reconstruire le graphe. La notification
            // part après la transaction, sans aucun verrou partagé conservé.
            if let Some(update) =
                graph.update_node_label(&resolution.mac, &resolution.ip, resolution.label.clone())
            {
                graph_updates.push(update);
            }
        }

        // Retire les conflits résolus, renvoie ceux qui restent.
        store.conflicts.retain(|c| {
            !resolutions
                .iter()
                .any(|resolution| resolution.mac == c.mac && resolution.ip == c.ip)
        });
        (store.conflicts.clone(), graph_updates)
    };

    // Best-effort : une erreur d'envoi ne remet pas en cause la transaction ;
    // le front se resynchronisera sur l'état backend cohérent.
    if let Some(on_event) = on_event {
        for update in &graph_updates {
            if let Err(e) = on_event.send(CaptureEvent::Graph { update }) {
                error!("Erreur d'envoi du GraphUpdate arbitrage: {e}");
            }
        }
    }

    Ok(remaining)
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
    use super::*;

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
}
