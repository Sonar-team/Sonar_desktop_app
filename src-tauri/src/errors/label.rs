#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("Nom(s) de fichier(s) en conflit: {files_names:?}")]
    FileNameConflicts { files_names: Vec<String> },
    #[error("Adresse MAC invalide: {invalid_mac:?} / Adresse IP invalide: {invalid_ip:?}")]
    InvalidFormats { invalid_mac: Vec<(String, String)>, invalid_ip: Vec<(String, String)> },
    #[error("Conflits détectés : IP -> Mac : {same_ip_diff_mac:?}, IP -> Label : {same_ip_diff_label:?}")]
    LabelLinesConflicts {same_ip_diff_mac: Vec<(String, String, String, String, String)>, same_ip_diff_label:Vec<(String, String, String, String, String)>}
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum LabelErrorKind {
    FileNameConflicts (Vec<String>),
    InvalidFormats (Vec<(String, String)>, Vec<(String, String)>),
    LabelLinesConflicts(Vec<(String, String, String, String, String)>, Vec<(String, String, String, String, String)>)
}