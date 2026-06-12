#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("Adresse MAC invalide: {invalid_mac:?} / Adresse IP invalide: {invalid_ip:?}")]
    InvalidMacIpFormat {
        invalid_mac: Vec<(String, String)>,
        invalid_ip: Vec<(String, String)>,
    },
    #[error("Format de fichier invalide. Attendu : mac, ip, label, trouvé : {invalid_lines:?}")]
    InvalidFileFormat { invalid_lines: Vec<String> },
    #[error(
        "Conflits détectés : IP -> Mac : {same_ip_diff_mac:?}, IP -> Label : {same_ip_diff_label:?}"
    )]
    LabelLinesConflicts {
        same_ip_diff_mac: Vec<(String, String, String, String, String)>,
        same_ip_diff_label: Vec<(String, String, String, String, String)>,
    },
    #[error("Trop de fichiers sélectionnés: {files_count} (maximum 1)")]
    TooManyFiles { files_count: usize },
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum LabelErrorKind {
    InvalidMacIpFormat(Vec<(String, String)>, Vec<(String, String)>),
    InvalidFileFormat(Vec<String>),
    LabelLinesConflicts(
        Vec<(String, String, String, String, String)>,
        Vec<(String, String, String, String, String)>,
    ),
    TooManyFiles(usize),
}
