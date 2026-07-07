pub type LabelInvalidField = (usize, String, String);
pub type LabelInvalidRow = (usize, String);
pub type LabelConflict = (usize, usize, String, String, String, String, String);

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
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum LabelErrorKind {
    InvalidMacIpFormat(Vec<LabelInvalidField>, Vec<LabelInvalidField>),
    InvalidRowsFormat(Vec<LabelInvalidRow>),
    LabelLinesConflicts(Vec<LabelConflict>, Vec<LabelConflict>),
}
