use std::sync::{Arc, Mutex};

use log::info;
use tauri::{AppHandle, State, command, ipc::Channel};

use crate::{
    errors::CaptureStateError,
    events::CaptureEvent,
    state::{
        capture::{
            CaptureState, capture_config::CaptureConfig, capture_handle::CaptureHandle,
            capture_status::CaptureStatus,
        },
        flow_matrix::FlowMatrix,
        graph::GraphData,
    },
};

#[command(async)]
pub fn start_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    app: AppHandle,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError> {
    let mut state_lock = state.lock()?;

    if state_lock.capture.is_some() {
        println!("Déjà en cours.");
        return Ok(state_lock.status.clone());
    }
    let capture = CaptureHandle::new();
    capture.start(state_lock.config.get_config(), app, on_event)?;
    state_lock.capture = Some(capture);
    state_lock.status.toggle();

    Ok(state_lock.status.clone())
}

#[command(async)]
pub fn stop_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError> {
    let mut app = state.lock()?;
    if let Some(capture) = app.capture.take() {
        capture.stop(on_event); // Suppose que stop() ne retourne pas d'erreur
        app.status.toggle();
    } else {
        println!("Aucun thread à arrêter.");
    }
    Ok(app.status.clone())
}

#[command(async, rename_all = "snake_case")]
pub fn config_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    device_name: String,
    buffer_size: i32,
    chan_capacity: i32,
    timeout: i32
) -> Result<CaptureConfig, CaptureStateError> {
    let mut app = state.lock()?; // Gestion d'erreur ici
    app.config.setup(device_name, buffer_size, chan_capacity, timeout);
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

#[command(async)]
pub fn get_config_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<CaptureConfig, CaptureStateError> {
    let app = state.lock()?; // Gestion d'erreur ici

    Ok(app.config.clone())
}

#[command(async)]
pub fn reset_capture(
    matrix: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
) {
    graph.lock().unwrap().clear();
    matrix.lock().unwrap().clear();
}
