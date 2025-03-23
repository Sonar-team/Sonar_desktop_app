use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Impossible de lister les interfaces réseau : {0}")]
    DeviceListError(#[from] pcap::Error),
}
