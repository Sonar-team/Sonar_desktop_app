//! Commande d'énumération des interfaces réseau.

use crate::dto::NetDevice;
use log::info;
use pcap::Device;
use tauri::command;

use crate::errors::capture_error::CaptureError;

/// Liste les interfaces réseau visibles par pcap, converties en DTO
/// sérialisables pour le sélecteur d'interface du frontend.
#[command(async)]
pub fn get_devices_list() -> Result<Vec<NetDevice>, CaptureError> {
    info!("Récupération des interfaces réseau");

    let devices = Device::list()?;
    info!("Interfaces réseau: {:#?}", devices);
    Ok(devices.into_iter().map(NetDevice::from).collect())
}
