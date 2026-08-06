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
            CaptureState,
            capture_config::CaptureConfig,
            capture_handle::CaptureHandle,
            capture_status::{CapturePhase, CaptureStatus},
            reap_terminated_capture, spawn_pipeline_reaper,
        },
        flow_matrix::FlowMatrix,
        graph::GraphData,
        labels_list::LabelStore,
    },
};

/// Démarre une capture live : recharge les labels dans la matrice, lance les
/// threads de capture puis attache le channel d'événements. Refuse
/// explicitement toute transition concurrente (capture déjà en cours ou en
/// train de démarrer/s'arrêter).
#[command(async)]
pub fn start_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    app: AppHandle,
    on_event: Channel<CaptureEvent<'static>>,
    state_label: State<'_, Arc<Mutex<FlowMatrix>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
) -> Result<CaptureStatus, CaptureStateError> {
    // Un pipeline arrêté de lui-même (erreur pcap, canal IPC cassé) laisse un
    // handle mort : on le récolte pour que ce démarrage ne réponde pas
    // « déjà en cours » à tort. La jointure se fait avant tout verrouillage
    // de la matrice et hors du verrou CaptureState (#166).
    if reap_terminated_capture(state.inner())? {
        info!("Pipeline précédent terminé : handle récolté avant redémarrage");
    }

    // Réserver `Starting` sous le seul verrou CaptureState. Pendant la suite
    // (labels, ouverture pcap, matrice), stop/import voient cette phase et
    // sont refusés au lieu de former un ordre de verrous inverse.
    let (session_id, config, filter) = {
        let mut state_lock = state.lock()?;
        let session_id = state_lock.begin_start()?;
        (
            session_id,
            state_lock.config.clone(),
            state_lock.filter.clone(),
        )
    };
    info!("Démarrage de la session de capture {session_id}");

    let start_result = (|| -> Result<_, CaptureStateError> {
        let mut state_label = state_label.lock()?;
        labels_to_matrix(label_store, &mut state_label)?;
        update_labels_in_state(&app, &mut state_label)?;

        // Contexte du relevé (#154) : seule l'interface d'écoute est
        // renseignée, parce qu'elle est la seule que SONAR connaisse sans
        // rien demander. Le site n'est pas déductible, et le nom de machine
        // est délibérément laissé vide : #96 traite la fuite du hostname
        // dans les fichiers produits comme une menace à couvrir, et un
        // export part chez le client.
        //
        // Aucune saisie, aucun dialogue : la métadonnée existe pour la
        // baseline et le diff temporel à venir (axe A de VISION.md, #164),
        // sans rien coûter au parcours d'aujourd'hui.
        state_label.context.interface =
            sonar_flows_core::sfms::SurveyContext::normalized_field(Some(&config.device_name));

        let mut capture = CaptureHandle::new(session_id);
        capture.start(config, app, on_event.clone(), filter, &mut state_label)?;
        let completion = capture.take_completion_receiver().ok_or_else(|| {
            CaptureStateError::InvalidTransition {
                from: "pipeline démarré sans coordinateur".to_string(),
                to: CapturePhase::Running.to_string(),
            }
        })?;
        Ok((capture, completion))
    })();

    match start_result {
        Ok((capture, completion)) => {
            // Le channel n'est mémorisé qu'après un démarrage réussi : un
            // échec ne laisse pas de canal fantôme attaché à l'état.
            let status = {
                let mut state_lock = state.lock()?;
                state_lock.complete_start(session_id, capture, on_event)?;
                // Une capture démarrée produit un relevé : à marquer modifié
                // dès maintenant (#159) — le pipeline n'a pas à toucher cet
                // état à chaque paquet.
                state_lock.mark_dirty();
                state_lock.status()
            };
            // Le coordinateur ne peut courir qu'après l'attachement. Si les
            // workers se sont déjà arrêtés, leurs notifications sont restées
            // dans le canal borné et seront consommées immédiatement.
            spawn_pipeline_reaper(Arc::downgrade(state.inner()), session_id, completion);
            Ok(status)
        }
        Err(e) => {
            state.lock()?.abort_start(session_id)?;
            Err(e)
        }
    }
}

/// Arrête la capture en cours (threads stoppés, channel détaché) et renvoie
/// le nouveau statut.
#[command(async)]
pub fn stop_capture(
    state: State<'_, Arc<Mutex<CaptureState>>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError> {
    stop_capture_inner(state.inner(), on_event)
}

/// Cœur de [`stop_capture`], séparé pour être testable sans `State` Tauri.
/// Idempotent depuis `Idle`. Si le coordinateur autonome possède déjà le
/// handle en phase `Stopping`, l'appel concurrent est explicitement refusé :
/// il ne prétend jamais avoir joint des threads qu'un autre finaliseur joint.
fn stop_capture_inner(
    state: &Arc<Mutex<CaptureState>>,
    on_event: Channel<CaptureEvent<'static>>,
) -> Result<CaptureStatus, CaptureStateError> {
    let detached = {
        let mut locked = state.lock()?;
        match locked.begin_stop()? {
            Some(detached) => detached,
            None => {
                info!("Aucun thread à arrêter.");
                return Ok(locked.status());
            }
        }
    };

    // Point central de #166 : aucune jointure n'a lieu sous CaptureState.
    // La phase reste `Stopping`, donc un start/import concurrent est refusé.
    let termination = detached.handle.stop();

    // `stop()` a joint les threads quoi qu'il arrive : le statut est
    // normalisé AVANT de propager une éventuelle erreur d'envoi de `Stopped`,
    // sinon le backend resterait marqué « stopping » sans capture.
    let status = state.lock()?.complete_stop(detached.session_id)?;
    termination.publish(&on_event)?;
    Ok(status)
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
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<(), CaptureStateError> {
    // Le reset est une opération destructive au même titre qu'un import :
    // il prend la même réservation exclusive et est donc refusé si un import
    // est déjà en phase `Importing` (#167).
    let reset_guard = crate::state::capture::ImportGuard::acquire(
        capture_state.inner(),
        "réinitialisation du relevé",
    )?;
    reset_guard.verify_current("commit de la réinitialisation")?;

    // Ordre global partagé par les imports : matrice -> graphe.
    {
        let mut matrix = matrix.lock()?;
        let mut graph = graph.lock()?;
        matrix.clear();
        graph.clear();
        // `FlowMatrix::clear` ne touche pas au contexte de relevé : le vider
        // explicitement ici est indispensable (#154). Le parcours réel est
        // « arrêter, réinitialiser, se rebrancher sur un autre switch,
        // recapturer » — garder la qualification précédente étiquetterait le
        // nouveau point de mesure avec le nom de l'ancien, en silence.
        matrix.context = sonar_flows_core::sfms::SurveyContext::default();
    }
    // Relevé vide : plus rien à perdre, la confirmation avant fermeture ne
    // doit plus se déclencher (#159). Verrous de données relâchés d'abord (#166).
    capture_state.lock()?.mark_clean();
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
    use crossbeam::channel::bounded;
    use std::{sync::Barrier, thread, time::Duration};
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    fn reset_invoke_request() -> InvokeRequest {
        InvokeRequest {
            cmd: "reset_capture".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(any(windows, target_os = "android")) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(serde_json::json!({})),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    /// Le canal IPC échoue à l'envoi (frontend mort) : l'erreur doit être
    /// propagée, mais PAS avant la normalisation du statut backend.
    #[test]
    fn stop_normalizes_status_even_when_stopped_event_fails() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.capture = Some(CaptureHandle::terminated_for_tests(0));
            locked.phase = CapturePhase::Running;
            locked.on_event = Some(Channel::new(|_| Ok(())));
        }
        let dead_channel = Channel::new(|_| Err(std::io::Error::other("canal mort").into()));

        let result = stop_capture_inner(&state, dead_channel);

        assert!(result.is_err(), "l'erreur d'envoi de Stopped est propagée");
        let locked = state.lock().unwrap();
        assert_eq!(
            locked.phase,
            CapturePhase::Idle,
            "statut normalisé malgré l'erreur"
        );
        assert!(locked.capture.is_none(), "le handle est libéré");
        assert!(locked.on_event.is_none(), "le channel mort est détaché");
    }

    #[test]
    fn stop_succeeds_and_normalizes_with_a_live_channel() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.capture = Some(CaptureHandle::terminated_for_tests(0));
            locked.phase = CapturePhase::Running;
        }

        let status = stop_capture_inner(&state, Channel::new(|_| Ok(()))).unwrap();

        assert!(!status.is_running);
        assert_eq!(status.phase, CapturePhase::Idle);
        assert!(state.lock().unwrap().capture.is_none());
    }

    #[test]
    fn stopped_event_observes_idle_without_capture_state_lock_held() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.capture = Some(CaptureHandle::terminated_for_tests(0));
            locked.phase = CapturePhase::Running;
        }
        let event_saw_idle = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let state_for_event = Arc::clone(&state);
        let saw_idle = Arc::clone(&event_saw_idle);
        let on_event = Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                let event: serde_json::Value = serde_json::from_str(&json).unwrap();
                if event["event"] == "stopped" {
                    let locked = state_for_event
                        .try_lock()
                        .expect("Stopped est émis hors du verrou CaptureState");
                    saw_idle.store(
                        locked.phase == CapturePhase::Idle && locked.capture.is_none(),
                        std::sync::atomic::Ordering::Release,
                    );
                }
            }
            Ok(())
        });

        let status = stop_capture_inner(&state, on_event).unwrap();

        assert_eq!(status.phase, CapturePhase::Idle);
        assert!(event_saw_idle.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn stop_without_capture_returns_current_status() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        let status = stop_capture_inner(&state, Channel::new(|_| Ok(()))).unwrap();
        assert!(!status.is_running);
    }

    #[test]
    fn stop_is_refused_while_start_is_reserved() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        let session_id = state.lock().unwrap().begin_start().unwrap();

        let result = stop_capture_inner(&state, Channel::new(|_| Ok(())));

        assert!(
            matches!(result, Err(CaptureStateError::InvalidTransition { .. })),
            "une course start/stop doit être refusée, pas modifier l'état"
        );
        let mut locked = state.lock().unwrap();
        assert_eq!(locked.phase, CapturePhase::Starting);
        locked.abort_start(session_id).unwrap();
    }

    #[test]
    fn stop_is_refused_while_the_auto_reaper_owns_the_handle() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        {
            let mut locked = state.lock().unwrap();
            locked.session_id = 4;
            locked.phase = CapturePhase::Stopping;
            locked.capture = None;
        }

        let result = stop_capture_inner(&state, Channel::new(|_| Ok(())));

        assert!(
            matches!(result, Err(CaptureStateError::InvalidTransition { .. })),
            "l'arrêt concurrent doit signaler que le finaliseur est déjà propriétaire"
        );
        let locked = state.lock().unwrap();
        assert_eq!(locked.phase, CapturePhase::Stopping);
        assert!(locked.capture.is_none());
    }

    /// Reproduit l'ancien cycle ABBA de #166 de façon déterministe : le faux
    /// processing attend la matrice pendant que stop le joint. CaptureState
    /// doit rester disponible, sans quoi un start tenant la matrice formerait
    /// l'interblocage historique.
    #[test]
    fn stop_releases_capture_state_before_join_and_refuses_concurrent_start() {
        let state = Arc::new(Mutex::new(CaptureState::new()));
        let matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let worker_ready = Arc::new(Barrier::new(2));
        let (about_to_lock_tx, about_to_lock_rx) = bounded::<()>(1);

        let matrix_for_worker = Arc::clone(&matrix);
        let ready_for_worker = Arc::clone(&worker_ready);
        let handle = CaptureHandle::with_thread_for_tests(1, move |stop_flag| {
            ready_for_worker.wait();
            while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                thread::yield_now();
            }
            about_to_lock_tx.send(()).unwrap();
            let _matrix = matrix_for_worker.lock().unwrap();
        });
        worker_ready.wait();

        {
            let mut locked = state.lock().unwrap();
            locked.session_id = 1;
            locked.phase = CapturePhase::Running;
            locked.capture = Some(handle);
            locked.on_event = Some(Channel::new(|_| Ok(())));
        }

        let matrix_guard = matrix.lock().unwrap();
        let state_for_stop = Arc::clone(&state);
        let (stop_done_tx, stop_done_rx) = bounded(1);
        thread::spawn(move || {
            let result = stop_capture_inner(&state_for_stop, Channel::new(|_| Ok(())))
                .map(|status| status.phase.to_string())
                .map_err(|error| error.to_string());
            let _ = stop_done_tx.send(result);
        });

        about_to_lock_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("le faux processing atteint l'attente de la matrice");

        let state_available = match state.try_lock() {
            Ok(mut locked) => {
                assert_eq!(locked.phase, CapturePhase::Stopping);
                assert!(
                    locked.begin_start().is_err(),
                    "un start concurrent est refusé pendant Stopping"
                );
                true
            }
            Err(_) => false,
        };

        // Nettoyage avant toute assertion : même une régression ne laisse pas
        // les threads du runner suspendus.
        drop(matrix_guard);
        let stop_result = stop_done_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("stop doit terminer dans la borne après libération de la matrice");

        assert!(
            state_available,
            "CaptureState ne doit pas être conservé pendant join()"
        );
        assert_eq!(stop_result.unwrap(), "idle");
        assert_eq!(state.lock().unwrap().phase, CapturePhase::Idle);
    }

    /// Le contrôle frontend n'est pas une synchronisation : le backend doit
    /// refuser lui-même un reset qui perd la course face à un import et ne
    /// toucher à aucune donnée (#167).
    #[test]
    fn tauri_reset_is_refused_while_importing_without_clearing_data() {
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_files/20260703_NP_Matrice.csv");
        let rows = sonar_flows_core::csv::read_matrix_rows(&source).unwrap();
        let mut seeded_matrix = FlowMatrix::new();
        seeded_matrix.link_type = Some(packet_parser::LinkType::ETHERNET);
        sonar_flows_core::csv::merge_rows(
            &mut seeded_matrix,
            &rows,
            packet_parser::LinkType::ETHERNET,
        )
        .unwrap();
        let seeded_graph = GraphData::from_flow_matrix(&seeded_matrix);

        let matrix = Arc::new(Mutex::new(seeded_matrix));
        let graph = Arc::new(Mutex::new(seeded_graph));
        let capture_state = Arc::new(Mutex::new(CaptureState::new()));
        let before_matrix = matrix.lock().unwrap().to_flat_vec();
        let before_graph = serde_json::to_value(&*graph.lock().unwrap()).unwrap();
        let import_guard = crate::state::capture::ImportGuard::acquire(
            &capture_state,
            "import concurrent de test",
        )
        .unwrap();

        let app = mock_builder()
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&capture_state))
            .invoke_handler(tauri::generate_handler![reset_capture])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let error = get_ipc_response(&webview, reset_invoke_request())
            .expect_err("reset concurrent à Importing refusé");

        assert_eq!(error["kind"], "invalidTransition");
        assert_eq!(matrix.lock().unwrap().to_flat_vec(), before_matrix);
        assert_eq!(
            serde_json::to_value(&*graph.lock().unwrap()).unwrap(),
            before_graph
        );
        assert_eq!(capture_state.lock().unwrap().phase, CapturePhase::Importing);
        drop(import_guard);
        assert_eq!(capture_state.lock().unwrap().phase, CapturePhase::Idle);
    }

    /// Un reset repart d'un relevé non qualifié (#154).
    ///
    /// Parcours réel de l'opérateur : arrêter, réinitialiser, se rebrancher
    /// sur un autre switch, recapturer. `FlowMatrix::clear` ne touchant pas au
    /// contexte, le conserver étiquetterait le nouveau point de mesure avec le
    /// nom de l'ancien — sans que rien ne le signale, puisque le dialogue de
    /// qualification ne se rouvre que sur un contexte vide.
    #[test]
    fn tauri_reset_clears_the_survey_context() {
        let mut seeded_matrix = FlowMatrix::new();
        seeded_matrix.context = sonar_flows_core::sfms::SurveyContext {
            site: Some("Usine Nord".to_string()),
            sensor: Some("PC de relevé".to_string()),
            interface: Some("eth0".to_string()),
        };

        let matrix = Arc::new(Mutex::new(seeded_matrix));
        let graph = Arc::new(Mutex::new(GraphData::new()));
        let capture_state = Arc::new(Mutex::new(CaptureState::new()));

        let app = mock_builder()
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&capture_state))
            .invoke_handler(tauri::generate_handler![reset_capture])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        get_ipc_response(&webview, reset_invoke_request()).expect("reset accepté");

        assert!(
            matrix.lock().unwrap().context.is_empty(),
            "après reset, le relevé suivant doit être proposé à la qualification"
        );
    }
}
