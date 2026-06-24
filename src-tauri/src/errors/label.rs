#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("Adresse MAC invalide: {invalid_mac:?} / Adresse IP invalide: {invalid_ip:?}")]
    InvalidMacIpFormat {
        invalid_mac: Vec<String>,
        invalid_ip: Vec<String>,
    },
    #[error("Format de fichier invalide. Attendu : mac, ip, label, trouvé : {invalid_lines:?}")]
    InvalidRowsFormat { invalid_lines: Vec<String> },
    #[error(
        "Conflits détectés : IP -> Mac : {same_ip_diff_mac:?}, IP -> Label : {same_ip_diff_label:?}"
    )]
    LabelLinesConflicts {
        same_ip_diff_mac: Vec<(String, String, String)>,
        same_ip_diff_label: Vec<(String, String, String)>,
    },
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum LabelErrorKind {
    InvalidMacIpFormat(Vec<String>, Vec<String>),
    InvalidRowsFormat(Vec<String>),
    LabelLinesConflicts(
        Vec<(String, String, String)>,
        Vec<(String, String, String)>,
    ),
}
