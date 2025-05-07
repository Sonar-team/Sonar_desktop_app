use thiserror::Error;

/// Enum représentant les différentes erreurs pouvant survenir lors de l'écriture de paquets vers un fichier CSV ou Excel.
#[derive(Debug, Error, serde::Serialize)]
pub enum ExportError {
    /// Erreur d'entrée/sortie avec un message explicatif.
    #[error("Erreur d'E/S : {0}")]
    Io(String),

    /// Erreur lors de la manipulation de fichiers CSV avec un message explicatif.
    #[error("Erreur CSV : {0}")]
    Csv(String),

    // /// Erreur de conversion UTF-8 avec un message explicatif.
    // #[error("Erreur de conversion UTF-8 : {0}")]
    // Utf8Error(String),
    /// Erreur lors de la manipulation de fichiers Excel avec un message explicatif.
    #[error("Erreur Excel : {0}")]
    Xlsx(String),
}
