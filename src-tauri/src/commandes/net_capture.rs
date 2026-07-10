//! Commandes du cycle de vie de la capture live : démarrage/arrêt,
//! configuration, filtre BPF et remise à zéro de l'état.

use std::sync::{Arc, Mutex};

use log::info;
use tauri::{AppHandle, State, command, ipc::Channel};

use crate::{
    commandes::import::labels_to_matrix,
    errors::CaptureStateError,
    events::CaptureEvent,
    setup::labels::update_labels_in_state,
    state::{
        capture::{
            CaptureState, capture_config::CaptureConfig, capture_handle::CaptureHandle,
            capture_status::CaptureStatus,
        },
        flow_matrix::FlowMatrix,
        graph::GraphData,
        labels_list::LabelStore,
    },
};

/// Démarre une capture live : recharge les labels dans la matrice, attache le
/// channel d'événements puis lance les threads de capture. Sans effet si une
/// capture tourne déjà (renvoie le statut courant).
#[command(async)]
pub fn start_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    app: AppHandle,
    on_event: Channel<CaptureEvent<'static>>,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<CaptureStatus, CaptureStateError> {
    let mut state_label = state_label.lock()?;
    labels_to_matrix(label_store, &mut state_label)?;
    update_labels_in_state(&app, &mut state_label)?;
    let mut state_lock = state.lock()?;
    // Un pipeline arrêté de lui-même (erreur pcap, canal IPC cassé) laisse un
    // handle mort : on le récolte pour que ce démarrage ne réponde pas
    // « déjà en cours » à tort.
    if state_lock.reap_terminated_capture() {
        info!("Pipeline précédent terminé : handle récolté avant redémarrage");
    }
    if state_lock.capture.is_some() {
        println!("Déjà en cours.");
        return Ok(state_lock.status.clone());
    }
    let mut capture = CaptureHandle::new();
    state_lock.on_event = Some(on_event.clone());
    capture.start(
        state_lock.config.clone(),
        app,
        on_event,
        state_lock.filter.clone(),
    )?;
    state_lock.capture = Some(capture);
    state_lock.status.toggle();

    Ok(state_lock.status.clone())
}

/// Variante headless de [`start_capture`] : démarre la capture sans channel
/// d'événements (aucun frontend à notifier).
pub fn start_capture_core(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    app: AppHandle,
) -> Result<CaptureStatus, CaptureStateError> {
    let mut st = state.lock()?;

    // Même récolte qu'en mode fenêtré : un pipeline mort ne doit pas bloquer
    // le redémarrage.
    if st.reap_terminated_capture() {
        info!("Pipeline précédent terminé : handle récolté avant redémarrage");
    }
    if st.capture.is_some() {
        return Ok(st.status.clone());
    }

    let mut capture = CaptureHandle::new();

    // Variante start sans event : start_no_event()
    capture.start_no_event(st.config.clone(), app, st.filter.clone())?;

    st.capture = Some(capture);
    st.status.toggle();

    Ok(st.status.clone())
}

/// Arrête la capture en cours (threads stoppés, channel détaché) et renvoie
/// le nouveau statut.
#[command(async)]
pub fn stop_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError> {
    let mut app = state.lock()?;
    stop_capture_inner(&mut app, on_event)
}

/// Cœur de [`stop_capture`], séparé pour être testable sans `State` Tauri.
fn stop_capture_inner(
    state: &mut CaptureState,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError> {
    if let Some(capture) = state.capture.take() {
        let stop_result = capture.stop(on_event);
        // `stop()` a joint les threads quoi qu'il arrive : le statut est
        // normalisé AVANT de propager une éventuelle erreur d'envoi de
        // `Stopped`, sinon le backend resterait marqué « en cours » sans
        // capture et le démarrage suivant serait marqué arrêté.
        state.status.is_running = false;
        state.on_event = None;
        stop_result?;
    } else {
        println!("Aucun thread à arrêter.");
    }
    Ok(state.status.clone())
}

/// Valide puis applique une nouvelle configuration de capture, persistée sur
/// disque pour les prochains démarrages.
#[command(async, rename_all = "snake_case")]
pub fn config_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    app_handle: AppHandle,
    device_name: String,
    buffer_size: i32,
    chan_capacity: i32,
    timeout: i32,
    snaplen: i32,
) -> Result<CaptureConfig, CaptureStateError> {
    let mut app = state.lock()?; // Gestion d'erreur ici
    let mut next_config = app.config.clone();
    next_config.setup(device_name, buffer_size, chan_capacity, timeout, snaplen)?;
    next_config.save_persisted(&app_handle)?;
    app.config = next_config;
    info!(
        "[get_config_capture] app.config {:?}",
        app.config.device_name
    );
    info!(
        "[get_config_capture] app.config {:?}",
        app.config.buffer_size
    );
    Ok(app.config.clone())
}

/// Configuration de capture courante.
#[command(async)]
pub fn get_config_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureConfig, CaptureStateError> {
    let app = state.lock()?; // Gestion d'erreur ici

    Ok(app.config.clone())
}

/// Vide la matrice de flux et le graphe (bouton reset du frontend).
#[command(async)]
pub fn reset_capture(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
) -> Result<(), CaptureStateError> {
    graph.lock()?.clear();
    matrix.lock()?.clear();
    Ok(())
}

/// Enregistre le filtre BPF à appliquer au prochain démarrage de capture.
#[command(async)]
pub fn set_filter(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    filter: String,
) -> Result<(), CaptureStateError> {
    info!("[set_filter] filter: {}", filter);
    let mut app = state.lock()?;
    app.filter = Some(filter);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le canal IPC échoue à l'envoi (frontend mort) : l'erreur doit être
    /// propagée, mais PAS avant la normalisation du statut backend.
    #[test]
    fn stop_normalizes_status_even_when_stopped_event_fails() {
        let mut state = CaptureState::new();
        state.capture = Some(CaptureHandle::terminated_for_tests());
        state.status.is_running = true;
        state.on_event = Some(Channel::new(|_| Ok(())));
        let dead_channel = Channel::new(|_| Err(std::io::Error::other("canal mort").into()));

        let result = stop_capture_inner(&mut state, dead_channel);

        assert!(result.is_err(), "l'erreur d'envoi de Stopped est propagée");
        assert!(!state.status.is_running, "statut normalisé malgré l'erreur");
        assert!(state.capture.is_none(), "le handle est libéré");
        assert!(state.on_event.is_none(), "le channel mort est détaché");
    }

    #[test]
    fn stop_succeeds_and_normalizes_with_a_live_channel() {
        let mut state = CaptureState::new();
        state.capture = Some(CaptureHandle::terminated_for_tests());
        state.status.is_running = true;

        let status = stop_capture_inner(&mut state, Channel::new(|_| Ok(()))).unwrap();

        assert!(!status.is_running);
        assert!(state.capture.is_none());
    }

    #[test]
    fn stop_without_capture_returns_current_status() {
        let mut state = CaptureState::new();
        let status = stop_capture_inner(&mut state, Channel::new(|_| Ok(()))).unwrap();
        assert!(!status.is_running);
    }
}
