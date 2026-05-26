#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("Adresse MAC invalide: {mac}")]
    InvalidMacAddress { mac: String },
    #[error("Adresse IP invalide: {ip}")]
    InvalidIpAddress {ip: String}
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum LabelErrorKind {
    InvalidMacAddress(String),
    InvalidIpAddress(String)
}