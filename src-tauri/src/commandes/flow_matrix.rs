//! Commandes de gestion des labels (#157) : le [`LabelStore`] est la source
//! de vérité unique ; chaque mutation resynchronise le miroir de la matrice
//! et les nœuds du graphe (`setup::labels::resync_labels`), notifie le
//! channel de capture s'il existe et retourne les updates graphe à
//! l'appelant (le panneau les applique sans dépendre du channel).

use std::sync::{Arc, Mutex};
use tauri::{State, command};

use crate::{
    commandes::import::normalize_label_key,
    errors::CaptureStateError,
    events::CaptureEvent,
    setup::labels::resync_labels,
    state::{
        capture::CaptureState,
        flow_matrix::FlowMatrix,
        graph::{GraphData, GraphUpdate},
        labels_list::{LabelStore, PcInfoLabel},
    },
};

/// Ligne de label enrichie de son statut : `observed` indique qu'un endpoint
/// de la matrice courante correspond à la clé (label « actif » vs
/// « dormant » dans l'UI de gestion).
#[derive(Debug, Clone, serde::Serialize)]
pub struct LabelRowStatus {
    pub mac: String,
    pub ip: String,
    pub label: String,
    pub observed: bool,
}

/// Resynchronise miroir + graphe après une mutation du store et pousse les
/// updates sur le channel de capture s'il existe. Même ordre de verrouillage
/// que les imports (matrice -> graphe -> store) pour éviter un interblocage.
pub(crate) fn resync_and_notify(
    matrix: &State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: &State<'_, Arc<Mutex<GraphData>>>,
    label_store: &State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: &State<'_, Arc<Mutex<PcInfoLabel>>>,
    capture_state: &State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<Vec<GraphUpdate>, CaptureStateError> {
    let updates = {
        let mut matrix_guard = matrix.lock()?;
        let mut graph_guard = graph.lock()?;
        let store_guard = label_store.lock()?;
        let pcinfo_guard = pcinfo.lock()?;
        resync_labels(
            &pcinfo_guard,
            &store_guard,
            &mut matrix_guard,
            &mut graph_guard,
        )
    };

    // Best-effort : une erreur d'envoi ne remet pas en cause la mutation,
    // l'appelant reçoit de toute façon les updates dans la réponse.
    // Tous les appelants sont des mutations de labels : le relevé est marqué
    // modifié ici, une fois les verrous de données relâchés (#166, #159).
    let on_event = {
        let mut state = capture_state.lock()?;
        state.mark_dirty();
        state.on_event.clone()
    };
    if let Some(on_event) = on_event {
        for update in &updates {
            if let Err(e) = on_event.send(CaptureEvent::Graph { update }) {
                log::error!("Erreur d'envoi du GraphUpdate label: {e}");
            }
        }
    }

    Ok(updates)
}

/// Ajoute ou remplace le label de la clé `(mac, ip)` (édition depuis le
/// graphe ou ajout depuis le panneau de gestion).
#[allow(clippy::too_many_arguments)]
#[command]
pub fn add_label(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: State<'_, Arc<Mutex<PcInfoLabel>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    mac: String,
    ip: String,
    label: String,
) -> Result<Vec<GraphUpdate>, CaptureStateError> {
    let (mac, ip) = normalize_label_key(&mac, &ip)?;
    label_store.lock()?.set(&mac, &ip, label.trim());
    resync_and_notify(&matrix, &graph, &label_store, &pcinfo, &capture_state)
}

/// Modifie une ligne du store, clé comprise. Refuse si la nouvelle clé
/// écraserait une autre ligne (pas d'écrasement silencieux : l'utilisateur
/// arbitre).
#[allow(clippy::too_many_arguments)]
#[command(rename_all = "snake_case")]
pub fn update_label(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: State<'_, Arc<Mutex<PcInfoLabel>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    old_mac: String,
    old_ip: String,
    mac: String,
    ip: String,
    label: String,
) -> Result<Vec<GraphUpdate>, CaptureStateError> {
    let (mac, ip) = normalize_label_key(&mac, &ip)?;
    label_store
        .lock()?
        .update(&old_mac, &old_ip, &mac, &ip, label.trim())
        .map_err(crate::errors::label::LabelError::EditRejected)?;
    resync_and_notify(&matrix, &graph, &label_store, &pcinfo, &capture_state)
}

/// Supprime la ligne de clé `(mac, ip)` : l'équipement est désétiqueté
/// (matrice et graphe resynchronisés).
#[command]
pub fn delete_label(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: State<'_, Arc<Mutex<PcInfoLabel>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    mac: String,
    ip: String,
) -> Result<Vec<GraphUpdate>, CaptureStateError> {
    label_store.lock()?.remove(&mac, &ip);
    resync_and_notify(&matrix, &graph, &label_store, &pcinfo, &capture_state)
}

/// Lignes du store avec leur statut observé/dormant : une ligne est
/// « observée » si un endpoint réel de la matrice courante correspond à sa
/// clé (couple exact, ou MAC seule / IP seule pour les labels partiels).
#[command]
pub fn get_labels_with_status(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<Vec<LabelRowStatus>, CaptureStateError> {
    let endpoints = matrix.lock()?.observed_endpoints();
    let observed_macs: std::collections::HashSet<&str> =
        endpoints.iter().map(|(mac, _)| mac.as_str()).collect();
    let observed_ips: std::collections::HashSet<&str> =
        endpoints.iter().map(|(_, ip)| ip.as_str()).collect();

    let store = label_store.lock()?;
    Ok(store
        .get()
        .iter()
        .map(|(mac, ip, label)| {
            let observed = match (mac.is_empty(), ip.is_empty()) {
                (false, false) => endpoints.contains(&(mac.clone(), ip.clone())),
                (false, true) => observed_macs.contains(mac.as_str()),
                (true, false) => observed_ips.contains(ip.as_str()),
                (true, true) => false,
            };
            LabelRowStatus {
                mac: mac.clone(),
                ip: ip.clone(),
                label: label.clone(),
                observed,
            }
        })
        .collect())
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
