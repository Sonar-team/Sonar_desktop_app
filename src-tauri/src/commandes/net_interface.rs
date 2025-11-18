use log::info;
use pcap::Device;
use tauri::command;
use crate::dto::NetDevice;

use crate::errors::capture_error::CaptureError;

#[command(async)]
pub fn get_devices_list() -> Result<Vec<NetDevice>, CaptureError> {
    info!("Récupération des interfaces réseau");

    let devices = Device::list()?;
    Ok(devices.into_iter().map(NetDevice::from).collect())
}


