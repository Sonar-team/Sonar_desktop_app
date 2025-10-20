use thiserror::Error;

#[derive(Error, Debug)]
pub enum PcapImportError {
    #[error("Failed to open pcap file {0}: {1}")]
    OpenFileError(String, String),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum PcapImportErrorKind {
    OpenFileError(String, String),
}
