//! Erreurs de validation des fichiers de labels CSV.

/// Champ invalide : (n° de ligne, valeur fautive, ligne brute).
pub type LabelInvalidField = (usize, String, String);
/// Ligne invalide : (n° de ligne, contenu affiché).
pub type LabelInvalidRow = (usize, String);
/// Conflit entre deux lignes : (ligne A, ligne B, ip, valeur A, valeur B,
/// ligne brute A, ligne brute B).
pub type LabelConflict = (usize, usize, String, String, String, String, String);

/// Erreur bloquante de validation d'un fichier de labels. Le frontend
/// affiche chaque variante dans un dialogue dédié (`ConflictDialog`).
#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("Adresse MAC invalide: {invalid_mac:?} / Adresse IP invalide: {invalid_ip:?}")]
    InvalidMacIpFormat {
        invalid_mac: Vec<LabelInvalidField>,
        invalid_ip: Vec<LabelInvalidField>,
    },
    #[error(
        "Format de fichier invalide. Attendu : au moins mac, ip, label ; colonnes suivantes fusionnées dans le label. Trouvé : {invalid_lines:?}"
    )]
    InvalidRowsFormat { invalid_lines: Vec<LabelInvalidRow> },
    // Les conflits de labels ne bloquent plus l'import (résolus « premier
    // gagné » et rapportés via LabelImportReport). Variante conservée pour la
    // compatibilité de l'API d'erreur et le futur module d'arbitrage.
    #[error(
        "Conflits détectés : IP -> Mac : {same_ip_diff_mac:?}, IP -> Label : {same_ip_diff_label:?}"
    )]
    #[allow(dead_code)]
    LabelLinesConflicts {
        same_ip_diff_mac: Vec<LabelConflict>,
        same_ip_diff_label: Vec<LabelConflict>,
    },
    /// Édition refusée par le store (ligne introuvable, collision de clé) —
    /// panneau de gestion des labels (#157).
    #[error("édition de label refusée : {0}")]
    EditRejected(String),
}

/// Représentation sérialisable de [`LabelError`] (forme `{ kind, message }`).
#[derive(serde::Serialize, ts_rs::TS)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum LabelErrorKind {
    InvalidMacIpFormat(Vec<LabelInvalidField>, Vec<LabelInvalidField>),
    InvalidRowsFormat(Vec<LabelInvalidRow>),
    LabelLinesConflicts(Vec<LabelConflict>, Vec<LabelConflict>),
    EditRejected(String),
}
