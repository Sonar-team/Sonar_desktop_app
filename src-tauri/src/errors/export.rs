#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Chemin de fichier vide")]
    EmptyPath,
    #[error("Erreur d’E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("Erreur CSV: {0}")]
    Csv(#[from] csv::Error),
    #[error("the mutex was poisoned")]
    PoisonError(String),
    #[error("Le dossier de logs est introuvable.")]
    LogNotFound,
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ExportErrorKind {
    EmptyPath,
    Io(String),
    Csv(String),
    PoisonError(String),
    LogNotFound,
}

impl<T> From<std::sync::PoisonError<T>> for ExportError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        ExportError::PoisonError(err.to_string())
    }
}
