//! Reusable Sonar domain logic.
//!
//! This crate must stay independent from Tauri so it can be shared by the
//! desktop app, the CLI, tests, and eventually external Rust consumers.

pub mod csv;
pub mod graph;
pub mod matrix;
pub mod packet;
#[cfg(feature = "pcap")]
pub mod pcap;

pub use error::{Result, SonarCoreError};

pub mod error {
    use std::path::PathBuf;

    #[derive(Debug, thiserror::Error)]
    pub enum SonarCoreError {
        #[error("at least one input file is required")]
        MissingInput,

        #[error("output path is required")]
        MissingOutput,

        #[error("input file does not exist: {0}")]
        MissingInputFile(PathBuf),

        #[error("{path}: {message}")]
        InvalidCsv { path: PathBuf, message: String },

        #[error(transparent)]
        Io(#[from] std::io::Error),

        #[cfg(feature = "pcap")]
        #[error("{path}: {message}")]
        Pcap { path: PathBuf, message: String },
    }

    pub type Result<T> = std::result::Result<T, SonarCoreError>;
}

pub(crate) fn validate_batch_paths(
    inputs: &[std::path::PathBuf],
    output: &std::path::Path,
) -> Result<()> {
    if inputs.is_empty() {
        return Err(SonarCoreError::MissingInput);
    }

    if output.as_os_str().is_empty() {
        return Err(SonarCoreError::MissingOutput);
    }

    for input in inputs {
        if !input.exists() {
            return Err(SonarCoreError::MissingInputFile(input.clone()));
        }
    }

    Ok(())
}
