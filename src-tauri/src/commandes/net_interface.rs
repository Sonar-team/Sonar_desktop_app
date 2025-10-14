use log::info;
use pcap::Device;
use tauri::command;

use crate::errors::capture_error::CaptureError;

#[command(async)]
pub fn get_devices_list() -> Result<Vec<String>, CaptureError> {
    info!("Récupération des interfaces réseau");

    let devices = Device::list()?;
    let names: Vec<String> = devices.iter().map(|d| d.name.clone()).collect();
    Ok(names)
}
