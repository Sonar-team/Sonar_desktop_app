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
#[command(async)]
pub fn save_project(
    app: AppHandle,
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    path: String,
) -> Result<(), CaptureStateError> {
    if path.trim().is_empty() {
        return Err(CaptureStateError::Export(ExportError::EmptyPath));
    }

    // Verrous courts : snapshots seulement, toute l'I/O se fait hors verrou
    // (même discipline que `export_csv` — le processing thread verrouille la
    // matrice à chaque paquet).
    let (rows, link_type, labels) = {
        let matrix = matrice.lock()?;
        (
            matrix.to_flat_vec(),
            matrix.link_type,
            matrix.export_labels(),
        )
    };
    let settings = {
        let state = capture_state.lock()?;
        ProjectCaptureSettings {
            config: state.config.clone(),
            filter: state.filter.clone(),
        }
    };

    let manifest = ProjectManifest {
        schema_version: PROJECT_SCHEMA_VERSION,
        app_version: app.package_info().version.to_string(),
        sfms_version: sonar_flows_core::sfms::SFMS_VERSION.to_string(),
        dlt: link_type.map(sonar_flows_core::sfms::link_type_name),
        matrix_rows: rows.len(),
        label_rows: labels.len(),
        saved_at_epoch_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };

    write_project_archive(
        Path::new(&path),
        &manifest,
        &rows,
        link_type,
        &labels,
        &settings,
    )?;
    info!(
        "✅ Projet enregistré vers {path} ({} flux, {} labels)",
        manifest.matrix_rows, manifest.label_rows
    );
    Ok(())
}

/// Ouvre un projet `.sonar` : valide le manifest puis REMPLACE le relevé
/// courant (matrice, graphe, labels, config) par celui du projet.
#[command(async)]
#[allow(clippy::too_many_arguments)]
pub fn open_project(
    app: AppHandle,
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

    info!(
        "✅ Projet ouvert depuis {path} (schéma v{}, app {})",
        extracted.manifest.schema_version, extracted.manifest.app_version
    );
    Ok(())
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
    use crate::state::flow_matrix::FlowMatrixRow;

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
        assert!(
            matrix_text.starts_with("#SFMS version=1 dlt=ETHERNET"),
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
