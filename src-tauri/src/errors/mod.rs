//! Erreurs de l'application. [`CaptureStateError`] agrège les erreurs de
//! chaque domaine (capture, export, import, labels) et se sérialise vers le
//! frontend sous la forme discriminée `{ kind, message }` attendue par
//! `src/errors/capture.ts`.

use capture_error::{CaptureError, CaptureErrorKind};
use serde::Serialize;

use crate::errors::{
    export::{ExportError, ExportErrorKind},
    import::{PcapImportError, PcapImportErrorKind},
    label::{LabelError, LabelErrorKind},
};

pub mod capture_error;
pub mod export;
pub mod import;
pub mod label;

/// Erreur agrégée retournée par les commandes Tauri : chaque variante
/// enveloppe l'erreur d'un domaine (IO, verrou empoisonné, capture, export,
/// import PCAP, labels, Tauri).
#[derive(Debug, thiserror::Error)]
pub enum CaptureStateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("the mutex was poisoned")]
    PoisonError(String),
    /// Transition refusée par la machine d'état de capture (ex. démarrage
    /// pendant qu'une capture tourne déjà).
    #[error("transition de capture refusée : {from} → {to}")]
    InvalidTransition { from: String, to: String },
    #[error(transparent)]
    Capture(#[from] CaptureError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Import(#[from] PcapImportError),
    #[error(transparent)]
    Label(#[from] LabelError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

/// Représentation sérialisable de [`CaptureStateError`] : forme discriminée
/// `{ kind, message }` consommée telle quelle par le frontend.
#[derive(serde::Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum CaptureStateErrorKind {
    Io(String),
    PoisonError(String),
    InvalidTransition(String),
    Capture(CaptureErrorKind),
    Export(ExportErrorKind),
    Import(PcapImportErrorKind),
    Label(LabelErrorKind),
    Tauri(String),
}

impl Serialize for CaptureStateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let kind = match self {
            Self::Io(e) => CaptureStateErrorKind::Io(e.to_string()),
            Self::PoisonError(e) => CaptureStateErrorKind::PoisonError(e.clone()),
            Self::InvalidTransition { .. } => {
                CaptureStateErrorKind::InvalidTransition(self.to_string())
            }
            Self::Capture(e) => {
                // Convert `CaptureError` into `CaptureErrorKind`
                let kind = match e {
                    CaptureError::InvalidConfig(msg) => {
                        CaptureErrorKind::InvalidConfig(msg.clone())
                    }
                    CaptureError::ConfigPersistence(msg) => {
                        CaptureErrorKind::ConfigPersistence(msg.clone())
                    }
                    CaptureError::InterfaceNotFound(msg) => {
                        CaptureErrorKind::InterfaceNotFound(msg.clone())
                    }
                    CaptureError::DeviceListError(e) => {
                        CaptureErrorKind::DeviceListError(e.to_string())
                    }
                    CaptureError::CaptureInitError(e) => {
                        CaptureErrorKind::CaptureInitError(e.to_string())
                    }
                    CaptureError::ChannelSendError(e) => {
                        CaptureErrorKind::ChannelSendError(e.to_string())
                    }
                    CaptureError::EventSendError(e) => {
                        CaptureErrorKind::EventSendError(e.to_string())
                    }
                    CaptureError::UnsupportedLinkType(label) => {
                        CaptureErrorKind::UnsupportedLinkType(label.clone())
                    }
                    CaptureError::MixedLinkType(message) => {
                        CaptureErrorKind::MixedLinkType(message.clone())
                    } //   CaptureError::FilterError(e) => CaptureErrorKind::FilterError(e.to_string()),
                };
                CaptureStateErrorKind::Capture(kind)
            }
            Self::Export(e) => {
                let kind = match e {
                    ExportError::EmptyPath => ExportErrorKind::EmptyPath,
                    ExportError::Io(e) => ExportErrorKind::Io(e.to_string()),
                    ExportError::Csv(e) => ExportErrorKind::Csv(e.to_string()),
                    ExportError::Zip(e) => ExportErrorKind::Zip(e.to_string()),
                    ExportError::PoisonError(e) => ExportErrorKind::PoisonError(e.clone()),
                    ExportError::LogNotFound => ExportErrorKind::LogNotFound,
                };
                CaptureStateErrorKind::Export(kind)
            }
            Self::Import(e) => {
                let kind = match e {
                    PcapImportError::MissingInput(operation) => {
                        PcapImportErrorKind::MissingInput(operation.clone())
                    }
                    PcapImportError::OpenFileError(msg, msgg) => {
                        PcapImportErrorKind::OpenFileError(msg.clone(), msgg.clone())
                    }
                    PcapImportError::ReadPacketError(file, msg) => {
                        PcapImportErrorKind::ReadPacketError(file.clone(), msg.clone())
                    }
                    PcapImportError::UnsupportedLinkType(file, label) => {
                        PcapImportErrorKind::UnsupportedLinkType(file.clone(), label.clone())
                    }
                    PcapImportError::Cancelled(operation) => {
                        PcapImportErrorKind::Cancelled(operation.clone())
                    }
                };
                CaptureStateErrorKind::Import(kind)
            }
            Self::Label(e) => {
                let kind = match e {
                    LabelError::InvalidMacIpFormat {
                        invalid_mac,
                        invalid_ip,
                    } => {
                        LabelErrorKind::InvalidMacIpFormat(invalid_mac.clone(), invalid_ip.clone())
                    }
                    LabelError::LabelLinesConflicts {
                        same_ip_diff_mac,
                        same_ip_diff_label,
                    } => LabelErrorKind::LabelLinesConflicts(
                        same_ip_diff_mac.clone(),
                        same_ip_diff_label.clone(),
                    ),
                    LabelError::InvalidRowsFormat { invalid_lines } => {
                        LabelErrorKind::InvalidRowsFormat(invalid_lines.clone())
                    }
                    LabelError::EditRejected(reason) => {
                        LabelErrorKind::EditRejected(reason.clone())
                    }
                };
                CaptureStateErrorKind::Label(kind)
            }
            Self::Tauri(e) => CaptureStateErrorKind::Tauri(e.to_string()),
        };
        kind.serialize(serializer)
    }
}

impl<T> From<std::sync::PoisonError<T>> for CaptureStateError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        CaptureStateError::PoisonError(err.to_string())
    }
}

/// Les erreurs du cœur partagé gardent leur distinction ouverture/lecture
/// (préservée côté front par `PcapImportErrorKind`) ; le reste (CSV invalide,
/// IO…) passe par la variante `Io` avec son message d'origine.
impl From<sonar_flows_core::SonarCoreError> for CaptureStateError {
    fn from(err: sonar_flows_core::SonarCoreError) -> Self {
        use sonar_flows_core::SonarCoreError;
        match err {
            SonarCoreError::PcapOpen { path, message } => CaptureStateError::Import(
                PcapImportError::OpenFileError(path.display().to_string(), message),
            ),
            SonarCoreError::PcapRead { path, message } => CaptureStateError::Import(
                PcapImportError::ReadPacketError(path.display().to_string(), message),
            ),
            SonarCoreError::UnsupportedLinkType { path, label } => CaptureStateError::Import(
                PcapImportError::UnsupportedLinkType(path.display().to_string(), label),
            ),
            SonarCoreError::MissingInput => {
                CaptureStateError::Import(PcapImportError::MissingInput("cet import".to_string()))
            }
            SonarCoreError::Io(e) => CaptureStateError::Io(e),
            other => CaptureStateError::Io(std::io::Error::other(other.to_string())),
        }
    }
}

/// Contrat IPC généré (#142) : écrit les types TypeScript des erreurs et du
/// payload `Stats` dans `src/types/generated/`, consommés par
/// `src/errors/capture.ts`. Relancer via `cargo test export_ipc_bindings`
/// après toute modification de ces types ; la CI vérifie l'absence de dérive
/// (`git diff --exit-code src/types/generated`).
#[cfg(test)]
mod bindings {
    use ts_rs::TS;

    #[test]
    fn export_ipc_bindings() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/types/generated");
        // `large_int = number` : serde_json sérialise les u64 en nombres
        // JSON, pas en bigint — le type généré doit dire la vérité du fil.
        let cfg = ts_rs::Config::new()
            .with_out_dir(dir)
            .with_large_int("number".to_owned());
        // `export_all` exporte aussi les types imbriqués (kinds par domaine)
        // avec leurs imports croisés.
        super::CaptureStateErrorKind::export_all(&cfg).expect("export du contrat d'erreurs");
        crate::state::capture::capture_handle::messages::stats::StatsPayload::export_all(&cfg)
            .expect("export du contrat Stats");
        // Union complète des événements du Channel de capture (#142) : voir
        // `crate::events::contract` pour la fidélité au format réellement
        // émis par `crate::events::CaptureEvent`.
        crate::events::contract::CaptureEventContract::export_all(&cfg)
            .expect("export du contrat CaptureEvent");
    }
}

/// Fidélité JSON du contrat d'erreurs (#142). Contrairement à
/// `crate::events::contract`, il n'y a pas de miroir dupliqué à faire
/// dériver ici : `CaptureStateErrorKind` (et les `XxxKind` par domaine) sont
/// à la fois la cible de sérialisation ET le type `ts-rs` — un seul `match`
/// exhaustif construit l'un depuis l'autre (`impl Serialize for
/// CaptureStateError` ci-dessus), sans possibilité de double définition qui
/// diverge. Le risque restant est plus étroit (le tag `{kind,message}` et le
/// `camelCase` de premier niveau), ce que ces tests couvrent par domaine.
#[cfg(test)]
mod fidelity_tests {
    use super::*;
    use serde_json::json;

    fn assert_json(error: &CaptureStateError, expected: serde_json::Value, label: &str) {
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            expected,
            "{label} : sérialisation divergente du contrat attendu"
        );
    }

    #[test]
    fn io_and_poison_and_invalid_transition_match() {
        assert_json(
            &CaptureStateError::PoisonError("verrou capturé".to_string()),
            json!({"kind": "poisonError", "message": "verrou capturé"}),
            "poisonError",
        );
        assert_json(
            &CaptureStateError::InvalidTransition {
                from: "Idle".to_string(),
                to: "Running".to_string(),
            },
            json!({"kind": "invalidTransition", "message": "transition de capture refusée : Idle → Running"}),
            "invalidTransition",
        );
    }

    #[test]
    fn capture_domain_nests_under_capture_tag() {
        assert_json(
            &CaptureStateError::Capture(CaptureError::InvalidConfig(
                "snaplen invalide".to_string(),
            )),
            json!({"kind": "capture", "message": {"kind": "invalidConfig", "message": "snaplen invalide"}}),
            "capture.invalidConfig",
        );
        assert_json(
            &CaptureStateError::Capture(CaptureError::UnsupportedLinkType("DLT 999".to_string())),
            json!({"kind": "capture", "message": {"kind": "unsupportedLinkType", "message": "DLT 999"}}),
            "capture.unsupportedLinkType",
        );
    }

    #[test]
    fn export_domain_nests_under_export_tag() {
        assert_json(
            &CaptureStateError::Export(ExportError::EmptyPath),
            json!({"kind": "export", "message": {"kind": "emptyPath"}}),
            "export.emptyPath",
        );
        assert_json(
            &CaptureStateError::Export(ExportError::LogNotFound),
            json!({"kind": "export", "message": {"kind": "logNotFound"}}),
            "export.logNotFound",
        );
    }

    #[test]
    fn import_domain_nests_under_import_tag() {
        assert_json(
            &CaptureStateError::Import(PcapImportError::MissingInput("l'import PCAP".to_string())),
            json!({
                "kind": "import",
                "message": {"kind": "missingInput", "message": "l'import PCAP"},
            }),
            "import.missingInput",
        );
        assert_json(
            &CaptureStateError::Import(PcapImportError::OpenFileError(
                "capture.pcap".to_string(),
                "permission refusée".to_string(),
            )),
            json!({
                "kind": "import",
                "message": {"kind": "openFileError", "message": ["capture.pcap", "permission refusée"]},
            }),
            "import.openFileError",
        );
    }

    #[test]
    fn label_domain_nests_under_label_tag() {
        assert_json(
            &CaptureStateError::Label(LabelError::EditRejected("ligne introuvable".to_string())),
            json!({"kind": "label", "message": {"kind": "editRejected", "message": "ligne introuvable"}}),
            "label.editRejected",
        );
        assert_json(
            &CaptureStateError::Label(LabelError::InvalidMacIpFormat {
                invalid_mac: vec![(3, "zz:zz".to_string(), "3,zz:zz,1.2.3.4".to_string())],
                invalid_ip: vec![],
            }),
            json!({
                "kind": "label",
                "message": {
                    "kind": "invalidMacIpFormat",
                    "message": [[[3, "zz:zz", "3,zz:zz,1.2.3.4"]], []],
                },
            }),
            "label.invalidMacIpFormat",
        );
    }
}

/// Conversion `sonar_flows_core::SonarCoreError -> CaptureStateError` : les
/// trois variantes PCAP distinguées (ouverture/lecture/DLT non supporté)
/// gardent leur identité côté front, le reste retombe sur `Io`.
#[cfg(test)]
mod sonar_core_error_conversion_tests {
    use super::*;
    use sonar_flows_core::SonarCoreError;
    use std::path::PathBuf;

    #[test]
    fn pcap_open_becomes_import_open_file_error() {
        let err: CaptureStateError = SonarCoreError::PcapOpen {
            path: PathBuf::from("capture.pcap"),
            message: "permission refusée".to_string(),
        }
        .into();
        match err {
            CaptureStateError::Import(PcapImportError::OpenFileError(path, message)) => {
                assert_eq!(path, "capture.pcap");
                assert_eq!(message, "permission refusée");
            }
            other => panic!("attendu Import(OpenFileError), reçu {other:?}"),
        }
    }

    #[test]
    fn pcap_read_becomes_import_read_packet_error() {
        let err: CaptureStateError = SonarCoreError::PcapRead {
            path: PathBuf::from("capture.pcap"),
            message: "paquet tronqué".to_string(),
        }
        .into();
        match err {
            CaptureStateError::Import(PcapImportError::ReadPacketError(path, message)) => {
                assert_eq!(path, "capture.pcap");
                assert_eq!(message, "paquet tronqué");
            }
            other => panic!("attendu Import(ReadPacketError), reçu {other:?}"),
        }
    }

    #[test]
    fn unsupported_link_type_is_preserved() {
        let err: CaptureStateError = SonarCoreError::UnsupportedLinkType {
            path: PathBuf::from("capture.pcap"),
            label: "DLT_RAW".to_string(),
        }
        .into();
        match err {
            CaptureStateError::Import(PcapImportError::UnsupportedLinkType(path, label)) => {
                assert_eq!(path, "capture.pcap");
                assert_eq!(label, "DLT_RAW");
            }
            other => panic!("attendu Import(UnsupportedLinkType), reçu {other:?}"),
        }
    }

    #[test]
    fn io_variant_passes_through() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "fichier introuvable");
        let err: CaptureStateError = SonarCoreError::Io(io_err).into();
        match err {
            CaptureStateError::Io(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            other => panic!("attendu Io, reçu {other:?}"),
        }
    }

    /// L'absence d'entrée garde une identité typée jusqu'au frontend.
    #[test]
    fn missing_input_becomes_typed_import_error() {
        let err: CaptureStateError = SonarCoreError::MissingInput.into();
        match err {
            CaptureStateError::Import(PcapImportError::MissingInput(operation)) => {
                assert_eq!(operation, "cet import");
            }
            other => panic!("attendu Import(MissingInput), reçu {other:?}"),
        }
    }
}
