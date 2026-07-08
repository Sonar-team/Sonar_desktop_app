use std::path::PathBuf;

use crate::{Result, validate_batch_paths};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixMergeRequest {
    pub inputs: Vec<PathBuf>,
    pub output: PathBuf,
}

impl MatrixMergeRequest {
    pub fn new(inputs: Vec<PathBuf>, output: PathBuf) -> Result<Self> {
        validate_batch_paths(&inputs, &output)?;
        Ok(Self { inputs, output })
    }
}
