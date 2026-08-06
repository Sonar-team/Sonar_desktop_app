//! Projet SONAR (`.sonar`) : un relevé complet dans un seul fichier (#159).
//!
//! Le format est une archive ZIP contenant des entrées à noms fixes :
//! - `manifest.json` — version de schéma, version de l'application, DLT et
//!   comptages, pour valider avant toute mutation et migrer plus tard ;
//! - `matrice.csv` — la matrice au format SFMS existant (même writer
//!   déterministe que l'export CSV, préambule `#SFMS` inclus) ;
//! - `labels.csv` — les labels résolus (même writer que l'export labels) ;
//! - `capture.json` — configuration de capture et filtre BPF de la session.
//!
//! L'ouverture ne réécrit aucune logique d'état : elle compose les chemins
//! d'import existants (`import_matrix_files` pour le swap transactionnel
//! matrice+graphe, `import_label_file` pour le remplacement du store de
//! labels et la resynchronisation), qui portent déjà l'exclusion mutuelle
//! avec la capture (#139/#167) et l'émission des événements IPC (#142).

use std::{
    fs::{self, File},
    io::{Read, Write, copy},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, command, ipc::Channel};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    errors::{CaptureStateError, export::ExportError, import::PcapImportError},
    events::CaptureEvent,
    state::{
        capture::{CaptureState, capture_config::CaptureConfig},
        flow_matrix::FlowMatrix,
        graph::GraphData,
        labels_list::{LabelStore, PcInfoLabel},
    },
};

/// Version du schéma de projet. À incrémenter à chaque évolution du contenu
/// de l'archive, avec un test de migration depuis chaque version antérieure.
/// La v2 attendue : identité d'actif contextualisée (#154).
pub const PROJECT_SCHEMA_VERSION: u32 = 1;

const MANIFEST_ENTRY: &str = "manifest.json";
const MATRIX_ENTRY: &str = "matrice.csv";
const LABELS_ENTRY: &str = "labels.csv";
const CAPTURE_ENTRY: &str = "capture.json";

/// Carte d'identité du projet, écrite en tête d'archive et validée avant
/// toute mutation de l'état à l'ouverture.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProjectManifest {
    pub schema_version: u32,
    pub app_version: String,
    pub sfms_version: String,
    /// Nom du DLT du relevé (`None` : relevé vide, DLT pas encore fixé).
    pub dlt: Option<String>,
    pub matrix_rows: usize,
    pub label_rows: usize,
    pub saved_at_epoch_secs: u64,
}

/// Réglages de session embarqués dans le projet. `CaptureConfig` est
/// revalidée à l'ouverture (bornes `MIN_*`/`MAX_*`) : un projet édité à la
/// main ne peut pas injecter une configuration hors bornes.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectCaptureSettings {
    config: CaptureConfig,
    filter: Option<String>,
}

/// Enregistre le relevé courant comme projet `.sonar` à `path`.
/// Générique sur le runtime pour être testable avec le harnais mock de Tauri.
#[command(async)]
pub fn save_project<R: tauri::Runtime>(
    app: AppHandle<R>,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    path: String,
) -> Result<(), CaptureStateError> {
    if path.trim().is_empty() {
        return Err(CaptureStateError::Export(ExportError::EmptyPath));
    }

    let (manifest, revision) = write_snapshot(
        &app.package_info().version.to_string(),
        matrice.inner(),
        capture_state.inner(),
        Path::new(&path),
    )?;
    // Enregistrement explicite de l'utilisateur : blanchit le suivi dirty
    // jusqu'à la révision snapshotée (l'autosave, lui, ne blanchit pas).
    capture_state.lock()?.mark_clean_at(revision);
    info!(
        "✅ Projet enregistré vers {path} ({} flux, {} labels)",
        manifest.matrix_rows, manifest.label_rows
    );
    Ok(())
}

/// Snapshot du relevé sous verrous courts puis écriture de l'archive vers
/// `dest` (même discipline que `export_csv` : toute l'I/O hors verrou).
/// Retourne le manifest écrit et la révision au moment du snapshot. Ne
/// touche PAS au suivi dirty : l'appelant décide.
fn write_snapshot(
    app_version: &str,
    matrice: &Arc<Mutex<FlowMatrix>>,
    capture_state: &Arc<Mutex<CaptureState>>,
    dest: &Path,
) -> Result<(ProjectManifest, u64), CaptureStateError> {
    let (rows, link_type, labels) = {
        let matrix = matrice.lock()?;
        (
            matrix.to_flat_vec(),
            matrix.link_type,
            matrix.export_labels(),
        )
    };
    // La révision est capturée avec le snapshot : des modifications arrivées
    // pendant l'écriture resteront comptées comme non enregistrées (#159).
    let (settings, revision) = {
        let state = capture_state.lock()?;
        (
            ProjectCaptureSettings {
                config: state.config.clone(),
                filter: state.filter.clone(),
            },
            state.current_revision(),
        )
    };

    let manifest = ProjectManifest {
        schema_version: PROJECT_SCHEMA_VERSION,
        app_version: app_version.to_string(),
        sfms_version: sonar_flows_core::sfms::SFMS_VERSION.to_string(),
        dlt: link_type.map(sonar_flows_core::sfms::link_type_name),
        matrix_rows: rows.len(),
        label_rows: labels.len(),
        saved_at_epoch_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };

    write_project_archive(dest, &manifest, &rows, link_type, &labels, &settings)?;
    Ok((manifest, revision))
}

/// Ouvre un projet `.sonar` : valide le manifest puis REMPLACE le relevé
/// courant (matrice, graphe, labels, config) par celui du projet.
#[command(async)]
#[allow(clippy::too_many_arguments)]
pub fn open_project<R: tauri::Runtime>(
    app: AppHandle<R>,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    graph: State<'_, Arc<Mutex<GraphData>>>,
    label_store: State<'_, Arc<Mutex<LabelStore>>>,
    pcinfo: State<'_, Arc<Mutex<PcInfoLabel>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    conflict_store: State<'_, Arc<Mutex<super::import::LabelConflictStore>>>,
    on_event: Channel<CaptureEvent<'static>>,
    path: String,
) -> Result<(), CaptureStateError> {
    let extracted = extract_project_archive(Path::new(&path))?;

    // Le swap transactionnel matrice+graphe, l'exclusion avec la capture et
    // les événements IPC sont ceux de l'import de matrice existant.
    super::import::import_matrix_files(
        vec![extracted.matrix_csv.to_string_lossy().into_owned()],
        matrice.clone(),
        graph.clone(),
        label_store.clone(),
        capture_state.clone(),
        on_event.clone(),
    )?;

    // Les labels du projet remplacent le store (source de vérité #157),
    // appliqués après la matrice pour que la resynchronisation porte sur le
    // relevé fraîchement chargé.
    if let Some(labels_csv) = &extracted.labels_csv {
        super::import::import_label_file(
            labels_csv.to_string_lossy().into_owned(),
            label_store,
            pcinfo,
            matrice,
            graph,
            capture_state.clone(),
            conflict_store,
            on_event,
        )?;
    }

    // La configuration embarquée est revalidée ; une config invalide (projet
    // édité à la main, bornes qui ont changé) n'empêche pas l'ouverture du
    // relevé — elle est simplement ignorée avec trace.
    if let Some(settings) = extracted.capture {
        match settings.config.validate() {
            Ok(()) => {
                {
                    let mut state = capture_state.lock()?;
                    state.config = settings.config.clone();
                    state.filter = settings.filter;
                }
                if let Err(err) = settings.config.save_persisted(&app) {
                    warn!("[open_project] persistance de la config impossible: {err:?}");
                }
            }
            Err(err) => warn!("[open_project] config du projet ignorée: {err:?}"),
        }
    }

    // L'état courant correspond exactement au fichier projet : rien à
    // perdre (les imports ci-dessus ont marqué dirty, on annule ici).
    capture_state.lock()?.mark_clean();

    info!(
        "✅ Projet ouvert depuis {path} (schéma v{}, app {})",
        extracted.manifest.schema_version, extracted.manifest.app_version
    );
    Ok(())
}

/// Vrai si des données seraient perdues en quittant ou réinitialisant sans
/// enregistrer (#159). Consommé par les confirmations du frontend.
#[command(async)]
pub fn is_session_dirty(
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
) -> Result<bool, CaptureStateError> {
    Ok(capture_state.lock()?.is_dirty())
}

// --- Autosave et récupération après crash (#159) ---

/// Nom de l'archive d'autosave, réécrite en place (atomiquement) par le
/// thread d'autosave tant que le relevé change.
pub const AUTOSAVE_FILE: &str = "autosave.sonar";
/// Sentinelle posée au démarrage et retirée à la fermeture propre : encore
/// présente au démarrage suivant, elle signe un crash.
const SESSION_LOCK_FILE: &str = "session.lock";
/// Cadence de l'autosave. Le tick ne fait rien si la révision n'a pas bougé
/// ou si un import est en cours.
const AUTOSAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

/// Offre de récupération figée au démarrage, AVANT la pose de la sentinelle
/// de la session courante (sinon elle se détecterait elle-même).
pub struct RecoveryOffer(Option<PathBuf>);

/// Chemin de l'autosave de la session précédente si elle s'est terminée par
/// un crash, sinon `None`. Le frontend propose alors la récupération.
#[command(async)]
pub fn get_recovery_offer(offer: State<'_, RecoveryOffer>) -> Option<String> {
    offer
        .0
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
}

/// Dossier des fichiers d'autosave/sentinelle de l'application.
fn autosave_dir<R: tauri::Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    use tauri::Manager;
    match app.path().app_data_dir() {
        Ok(dir) => Some(dir.join("autosave")),
        Err(err) => {
            warn!("[autosave] dossier de données introuvable: {err}");
            None
        }
    }
}

/// Une sentinelle laissée par une session morte + un autosave lisible =
/// récupération à proposer. Fonction pure sur le contenu du dossier.
fn detect_recovery(dir: &Path) -> Option<PathBuf> {
    let autosave = dir.join(AUTOSAVE_FILE);
    (dir.join(SESSION_LOCK_FILE).exists() && autosave.is_file()).then_some(autosave)
}

/// À appeler une fois au setup : calcule l'offre de récupération, pose la
/// sentinelle de cette session et démarre le thread d'autosave. L'offre
/// retournée est à placer dans l'état managé.
pub fn init_session_persistence<R: tauri::Runtime>(app: &AppHandle<R>) -> RecoveryOffer {
    let Some(dir) = autosave_dir(app) else {
        return RecoveryOffer(None);
    };
    if let Err(err) = fs::create_dir_all(&dir) {
        warn!("[autosave] création {} impossible: {err}", dir.display());
        return RecoveryOffer(None);
    }

    let offer = detect_recovery(&dir);
    if offer.is_some() {
        info!("[autosave] session précédente non fermée : récupération proposée");
    }
    if let Err(err) = fs::write(dir.join(SESSION_LOCK_FILE), std::process::id().to_string()) {
        warn!("[autosave] sentinelle non posée: {err}");
    }

    spawn_autosaver(app.clone(), dir.join(AUTOSAVE_FILE));
    RecoveryOffer(offer)
}

/// À appeler à la sortie propre (`RunEvent::Exit`) : la sentinelle retirée,
/// le prochain démarrage ne proposera pas de récupération.
pub fn remove_session_lock<R: tauri::Runtime>(app: &AppHandle<R>) {
    if let Some(dir) = autosave_dir(app) {
        let _ = fs::remove_file(dir.join(SESSION_LOCK_FILE));
    }
}

/// Thread d'autosave : réécrit l'archive d'autosave quand la révision a
/// bougé depuis la dernière écriture, jamais pendant un import (le prochain
/// tick verra l'état stable d'après). Best-effort : une erreur est tracée
/// et le tick suivant réessaie.
fn spawn_autosaver<R: tauri::Runtime>(app: AppHandle<R>, autosave_path: PathBuf) {
    use tauri::Manager;
    std::thread::spawn(move || {
        let mut last_written: u64 = 0;
        loop {
            std::thread::sleep(AUTOSAVE_INTERVAL);

            let capture_state = app.state::<Arc<Mutex<CaptureState>>>();
            let (revision, importing) = match capture_state.lock() {
                Ok(state) => (
                    state.current_revision(),
                    state.phase == crate::state::capture::capture_status::CapturePhase::Importing,
                ),
                Err(err) => {
                    warn!("[autosave] verrou empoisonné, tick ignoré: {err}");
                    continue;
                }
            };
            if revision == last_written || importing {
                continue;
            }

            let matrice = app.state::<Arc<Mutex<FlowMatrix>>>();
            match write_snapshot(
                &app.package_info().version.to_string(),
                matrice.inner(),
                capture_state.inner(),
                &autosave_path,
            ) {
                Ok((manifest, snapshot_revision)) => {
                    last_written = snapshot_revision;
                    info!(
                        "[autosave] relevé sauvegardé ({} flux, révision {snapshot_revision})",
                        manifest.matrix_rows
                    );
                }
                Err(err) => warn!("[autosave] écriture échouée, retenté au prochain tick: {err:?}"),
            }
        }
    });
}

/// Écrit l'archive projet vers `dest` : entrées composées en staging via les
/// writers CSV existants, archive assemblée en `.part` puis renommée — une
/// écriture interrompue ne laisse jamais d'archive partielle au chemin final.
fn write_project_archive(
    dest: &Path,
    manifest: &ProjectManifest,
    rows: &[crate::state::flow_matrix::FlowMatrixRow],
    link_type: Option<packet_parser::LinkType>,
    labels: &[(String, String, String)],
    settings: &ProjectCaptureSettings,
) -> Result<(), CaptureStateError> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let staging = unique_temp_dir("save");
    let result = (|| -> Result<(), CaptureStateError> {
        let matrix_csv = staging.join(MATRIX_ENTRY);
        FlowMatrix::write_rows_to_csv(rows, link_type, &matrix_csv.to_string_lossy())?;
        let labels_csv = staging.join(LABELS_ENTRY);
        FlowMatrix::write_label_rows_to_csv(labels, &labels_csv.to_string_lossy())?;

        let manifest_json = serde_json::to_string_pretty(manifest)
            .map_err(|err| ExportError::Io(std::io::Error::other(err)))?;
        let capture_json = serde_json::to_string_pretty(settings)
            .map_err(|err| ExportError::Io(std::io::Error::other(err)))?;

        let tmp_dest = dest.with_extension("part");
        let write = (|| -> Result<(), CaptureStateError> {
            let file = File::create(&tmp_dest)?;
            let mut zip = ZipWriter::new(file);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            for (name, content) in [
                (MANIFEST_ENTRY, manifest_json),
                (CAPTURE_ENTRY, capture_json),
            ] {
                zip.start_file(name, options).map_err(ExportError::Zip)?;
                zip.write_all(content.as_bytes())?;
            }
            for (name, path) in [(MATRIX_ENTRY, &matrix_csv), (LABELS_ENTRY, &labels_csv)] {
                zip.start_file(name, options).map_err(ExportError::Zip)?;
                let mut src = File::open(path)?;
                copy(&mut src, &mut zip)?;
            }
            zip.finish().map_err(ExportError::Zip)?;
            Ok(())
        })();
        if let Err(err) = write {
            let _ = fs::remove_file(&tmp_dest);
            return Err(err);
        }
        fs::rename(&tmp_dest, dest)?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&staging);
    result
}

/// Contenu extrait d'une archive projet, prêt à passer aux chemins d'import.
/// Le dossier temporaire vit tant que la valeur existe.
#[derive(Debug)]
struct ExtractedProject {
    manifest: ProjectManifest,
    matrix_csv: PathBuf,
    labels_csv: Option<PathBuf>,
    capture: Option<ProjectCaptureSettings>,
    /// Supprime le dossier temporaire à la sortie de portée, succès ou non.
    _temp_dir: TempDirGuard,
}

#[derive(Debug)]
struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Ouvre et valide une archive projet : manifest lisible, version de schéma
/// supportée, matrice présente. Seules les entrées à noms fixes sont
/// extraites — aucun nom d'entrée de l'archive n'est utilisé comme chemin,
/// ce qui exclut toute traversée (« zip slip »).
fn extract_project_archive(src: &Path) -> Result<ExtractedProject, CaptureStateError> {
    let open_error = |message: String| {
        CaptureStateError::Import(PcapImportError::OpenFileError(
            src.to_string_lossy().into_owned(),
            message,
        ))
    };

    let file = File::open(src).map_err(|err| open_error(format!("ouverture impossible: {err}")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| open_error(format!("ce n'est pas une archive projet valide: {err}")))?;

    let manifest_text = read_entry(&mut archive, MANIFEST_ENTRY)
        .map_err(|err| open_error(format!("{MANIFEST_ENTRY} illisible: {err}")))?
        .ok_or_else(|| open_error(format!("{MANIFEST_ENTRY} absent — pas un projet SONAR")))?;
    let manifest: ProjectManifest = serde_json::from_str(&manifest_text)
        .map_err(|err| open_error(format!("{MANIFEST_ENTRY} invalide: {err}")))?;
    if manifest.schema_version > PROJECT_SCHEMA_VERSION {
        return Err(open_error(format!(
            "projet au schéma v{} (créé par SONAR {}), cette version ne lit que v{} au plus — \
             mettez SONAR à jour pour l'ouvrir",
            manifest.schema_version, manifest.app_version, PROJECT_SCHEMA_VERSION
        )));
    }

    let matrix_text = read_entry(&mut archive, MATRIX_ENTRY)
        .map_err(|err| open_error(format!("{MATRIX_ENTRY} illisible: {err}")))?
        .ok_or_else(|| open_error(format!("{MATRIX_ENTRY} absent de l'archive")))?;
    let labels_text = read_entry(&mut archive, LABELS_ENTRY)
        .map_err(|err| open_error(format!("{LABELS_ENTRY} illisible: {err}")))?;
    let capture = read_entry(&mut archive, CAPTURE_ENTRY)
        .map_err(|err| open_error(format!("{CAPTURE_ENTRY} illisible: {err}")))?
        .and_then(|text| match serde_json::from_str(&text) {
            Ok(settings) => Some(settings),
            Err(err) => {
                warn!("[open_project] {CAPTURE_ENTRY} invalide, ignoré: {err}");
                None
            }
        });

    let temp_dir = unique_temp_dir("open");
    let matrix_csv = temp_dir.join(MATRIX_ENTRY);
    fs::write(&matrix_csv, matrix_text).inspect_err(|_| {
        let _ = fs::remove_dir_all(&temp_dir);
    })?;
    let labels_csv = match labels_text {
        Some(text) => {
            let path = temp_dir.join(LABELS_ENTRY);
            fs::write(&path, text).inspect_err(|_| {
                let _ = fs::remove_dir_all(&temp_dir);
            })?;
            Some(path)
        }
        None => None,
    };

    Ok(ExtractedProject {
        manifest,
        matrix_csv,
        labels_csv,
        capture,
        _temp_dir: TempDirGuard(temp_dir),
    })
}

/// Lit une entrée de l'archive par son nom fixe ; `None` si absente.
fn read_entry(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> Result<Option<String>, zip::result::ZipError> {
    match archive.by_name(name) {
        Ok(mut entry) => {
            let mut content = String::new();
            entry
                .read_to_string(&mut content)
                .map_err(zip::result::ZipError::Io)?;
            Ok(Some(content))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(err) => Err(err),
    }
}

/// Dossier temporaire unique par processus et par appel : deux sauvegardes
/// concurrentes ne partagent jamais leur staging.
fn unique_temp_dir(label: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("sonar_project_{label}_{}_{n}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commandes::import::import_matrix_files;
    use crate::state::flow_matrix::FlowMatrixRow;
    use tauri::{
        ipc::{CallbackFn, InvokeBody, InvokeResponseBody},
        test::{INVOKE_KEY, get_ipc_response, mock_builder, mock_context, noop_assets},
        webview::InvokeRequest,
    };

    fn invoke_request(cmd: &str, body: serde_json::Value) -> InvokeRequest {
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: if cfg!(any(windows, target_os = "android")) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        }
    }

    /// Critère d'acceptation n°1 de #159, joué via les vraies commandes
    /// IPC : un relevé importé puis enregistré comme projet se rouvre avec
    /// une matrice identique après un « redémarrage » (état vidé), et le
    /// suivi dirty suit chaque étape (import → modifié, save/open → propre).
    #[test]
    fn tauri_round_trip_save_open_restores_the_same_survey() {
        let matrix = Arc::new(Mutex::new(FlowMatrix::new()));
        let graph = Arc::new(Mutex::new(crate::state::graph::GraphData::new()));
        let labels = Arc::new(Mutex::new(LabelStore::new()));
        let pcinfo = Arc::new(Mutex::new(PcInfoLabel::new()));
        let capture_state = Arc::new(Mutex::new(CaptureState::new()));
        let conflicts = Arc::new(Mutex::new(
            crate::commandes::import::LabelConflictStore::default(),
        ));

        let app = mock_builder()
            .channel_interceptor(|_webview, _callback, _index, _body| true)
            .manage(Arc::clone(&matrix))
            .manage(Arc::clone(&graph))
            .manage(Arc::clone(&labels))
            .manage(Arc::clone(&pcinfo))
            .manage(Arc::clone(&capture_state))
            .manage(Arc::clone(&conflicts))
            .invoke_handler(tauri::generate_handler![
                save_project,
                open_project,
                is_session_dirty
            ])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let read_dirty = |webview: &tauri::WebviewWindow<_>| -> bool {
            match get_ipc_response(
                webview,
                invoke_request("is_session_dirty", serde_json::json!({})),
            )
            .expect("is_session_dirty doit répondre")
            {
                InvokeResponseBody::Json(json) => serde_json::from_str(&json).unwrap(),
                other => panic!("réponse inattendue: {other:?}"),
            }
        };

        assert!(!read_dirty(&webview), "état initial propre");

        // 1) Relevé peuplé par le VRAI chemin d'import (appel direct de la
        // fonction de commande : les macros IPC d'un autre module ne sont
        // pas visibles ici, la logique exercée est la même).
        use tauri::Manager;
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files/20260703_NP_Matrice.csv");
        import_matrix_files(
            vec![source.to_string_lossy().into_owned()],
            app.state(),
            app.state(),
            app.state(),
            app.state(),
            Channel::new(|_| Ok(())),
        )
        .expect("l'import de la matrice de test doit réussir");

        let rows_before = matrix.lock().unwrap().to_flat_vec();
        assert!(
            !rows_before.is_empty(),
            "la fixture doit peupler la matrice"
        );
        let nodes_before = graph.lock().unwrap().nodes.len();
        assert!(read_dirty(&webview), "un import marque le relevé modifié");

        // 2) Enregistrement du projet.
        let dir = unique_temp_dir("test_e2e");
        let project_path = dir.join("relevé de test.sonar");
        get_ipc_response(
            &webview,
            invoke_request(
                "save_project",
                serde_json::json!({ "path": project_path.to_string_lossy() }),
            ),
        )
        .expect("save_project doit réussir");
        assert!(
            !read_dirty(&webview),
            "un enregistrement blanchit le relevé"
        );

        // 3) « Redémarrage » : état vidé puis projet rouvert.
        matrix.lock().unwrap().clear();
        graph.lock().unwrap().clear();
        labels.lock().unwrap().clear();
        get_ipc_response(
            &webview,
            invoke_request(
                "open_project",
                serde_json::json!({
                    "path": project_path.to_string_lossy(),
                    "onEvent": "__CHANNEL__:2",
                }),
            ),
        )
        .expect("open_project doit réussir");

        assert_eq!(
            matrix.lock().unwrap().to_flat_vec(),
            rows_before,
            "la matrice rouverte doit être identique à celle enregistrée"
        );
        assert_eq!(
            graph.lock().unwrap().nodes.len(),
            nodes_before,
            "le graphe est reconstruit avec les mêmes nœuds"
        );
        assert!(
            !read_dirty(&webview),
            "un projet fraîchement ouvert n'a rien à perdre"
        );

        fs::remove_dir_all(&dir).ok();
    }

    fn sample_row(port_destination: u16) -> FlowMatrixRow {
        FlowMatrixRow {
            mac_source: "aa:bb:cc:00:00:01".to_string(),
            mac_destination: "aa:bb:cc:00:00:11".to_string(),
            vlan_id: None,
            protocol_data_link: "IPv4".to_string(),
            ip_source: "192.168.50.1".to_string(),
            ip_source_type: "Privée".to_string(),
            label_source: None,
            ip_destination: "192.168.50.11".to_string(),
            ip_destination_type: "Privée".to_string(),
            label_destination: Some("capteur".to_string()),
            port_source: Some(5247),
            port_destination: Some(port_destination),
            protocol_transport: Some("UDP".to_string()),
            application_protocol: Some("Unknown".to_string()),
            count: 1,
            total_bytes: 101,
            last_seen: "2024-07-03 09:46:40.400000 UTC".to_string(),
            encap_id: String::new(),
            origin: String::new(),
        }
    }

    fn sample_settings() -> ProjectCaptureSettings {
        ProjectCaptureSettings {
            config: CaptureConfig {
                device_name: "eth0".to_string(),
                buffer_size: 18_000_000,
                chan_capacity: 10_000,
                timeout: 25,
                snaplen: 65536,
            },
            filter: Some("udp".to_string()),
        }
    }

    fn sample_manifest(schema_version: u32) -> ProjectManifest {
        ProjectManifest {
            schema_version,
            app_version: "test".to_string(),
            sfms_version: sonar_flows_core::sfms::SFMS_VERSION.to_string(),
            dlt: Some("ETHERNET".to_string()),
            matrix_rows: 2,
            label_rows: 1,
            saved_at_epoch_secs: 1_754_000_000,
        }
    }

    fn write_sample_project(dest: &Path, schema_version: u32) {
        let link_type = sonar_flows_core::sfms::link_type_from_text("ETHERNET");
        let rows = vec![sample_row(40000), sample_row(40001)];
        let labels = vec![(
            "aa:bb:cc:00:00:11".to_string(),
            "192.168.50.11".to_string(),
            "capteur".to_string(),
        )];
        write_project_archive(
            dest,
            &sample_manifest(schema_version),
            &rows,
            link_type,
            &labels,
            &sample_settings(),
        )
        .unwrap();
    }

    #[test]
    fn round_trip_preserves_manifest_matrix_labels_and_settings() {
        let dir = unique_temp_dir("test_roundtrip");
        let dest = dir.join("relevé projet.sonar");

        write_sample_project(&dest, PROJECT_SCHEMA_VERSION);

        assert!(dest.is_file(), "l'archive doit exister au chemin final");
        assert!(
            !dest.with_extension("part").exists(),
            "aucun fichier temporaire .part ne doit rester après une sauvegarde réussie"
        );

        let extracted = extract_project_archive(&dest).unwrap();
        assert_eq!(extracted.manifest, sample_manifest(PROJECT_SCHEMA_VERSION));

        let matrix_text = fs::read_to_string(&extracted.matrix_csv).unwrap();
        // La version vient de la crate : un bump de SFMS_VERSION ne doit pas
        // exiger de retoucher ce test, seulement de le voir rester vrai.
        let expected_preamble = format!(
            "#SFMS version={} dlt=ETHERNET",
            sonar_flows_core::sfms::SFMS_VERSION
        );
        assert!(
            matrix_text.starts_with(&expected_preamble),
            "la matrice embarquée doit porter le préambule SFMS, trouvé: {}",
            matrix_text.lines().next().unwrap_or_default()
        );
        assert_eq!(matrix_text.matches("192.168.50.11").count(), 2);

        let labels_text = fs::read_to_string(extracted.labels_csv.as_ref().unwrap()).unwrap();
        assert!(labels_text.contains("aa:bb:cc:00:00:11"));
        assert!(labels_text.contains("capteur"));

        let settings = extracted.capture.as_ref().unwrap();
        assert_eq!(settings.config.device_name, "eth0");
        assert_eq!(settings.filter.as_deref(), Some("udp"));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn future_schema_version_is_refused_before_any_mutation() {
        let dir = unique_temp_dir("test_future_schema");
        let dest = dir.join("futur.sonar");

        write_sample_project(&dest, PROJECT_SCHEMA_VERSION + 1);

        let err = extract_project_archive(&dest).unwrap_err();
        let message = format!("{err:?}");
        assert!(
            message.contains("mettez SONAR à jour"),
            "le refus doit expliquer la marche à suivre, trouvé: {message}"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn garbage_file_is_refused_as_invalid_archive() {
        let dir = unique_temp_dir("test_garbage");
        let dest = dir.join("pas-un-zip.sonar");
        fs::write(&dest, b"ceci n'est pas une archive").unwrap();

        let err = extract_project_archive(&dest).unwrap_err();
        assert!(matches!(
            err,
            CaptureStateError::Import(PcapImportError::OpenFileError(_, _))
        ));

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn archive_without_matrix_entry_is_refused() {
        let dir = unique_temp_dir("test_no_matrix");
        let dest = dir.join("sans-matrice.sonar");

        let file = File::create(&dest).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file(MANIFEST_ENTRY, options).unwrap();
        zip.write_all(
            serde_json::to_string(&sample_manifest(PROJECT_SCHEMA_VERSION))
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        zip.finish().unwrap();

        let err = extract_project_archive(&dest).unwrap_err();
        let message = format!("{err:?}");
        assert!(
            message.contains(MATRIX_ENTRY),
            "le refus doit nommer l'entrée manquante, trouvé: {message}"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn recovery_needs_both_sentinel_and_autosave() {
        let dir = unique_temp_dir("test_recovery");

        assert!(
            detect_recovery(&dir).is_none(),
            "dossier vide : rien à récupérer"
        );

        fs::write(dir.join(SESSION_LOCK_FILE), b"42").unwrap();
        assert!(
            detect_recovery(&dir).is_none(),
            "sentinelle sans autosave : rien à proposer"
        );

        fs::write(dir.join(AUTOSAVE_FILE), b"zip").unwrap();
        assert_eq!(
            detect_recovery(&dir),
            Some(dir.join(AUTOSAVE_FILE)),
            "sentinelle + autosave : récupération proposée"
        );

        fs::remove_file(dir.join(SESSION_LOCK_FILE)).unwrap();
        assert!(
            detect_recovery(&dir).is_none(),
            "fermeture propre (sentinelle retirée) : pas de récupération"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_labels_and_capture_entries_are_tolerated() {
        let dir = unique_temp_dir("test_minimal");
        let dest = dir.join("minimal.sonar");

        let file = File::create(&dest).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file(MANIFEST_ENTRY, options).unwrap();
        zip.write_all(
            serde_json::to_string(&sample_manifest(PROJECT_SCHEMA_VERSION))
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        zip.start_file(MATRIX_ENTRY, options).unwrap();
        zip.write_all(sonar_flows_core::sfms::format_preamble(None).as_bytes())
            .unwrap();
        zip.finish().unwrap();

        let extracted = extract_project_archive(&dest).unwrap();
        assert!(extracted.labels_csv.is_none());
        assert!(extracted.capture.is_none());

        fs::remove_dir_all(&dir).ok();
    }
}
