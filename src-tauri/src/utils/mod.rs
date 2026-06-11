use pcap::Device;

pub fn lookup_default_device() -> Result<Device, String> {
    Device::lookup()
        .map_err(|err| format!("device lookup failed: {err}"))?
        .ok_or_else(|| "no device available".to_string())
}

pub fn return_device_lookup() -> String {
    match lookup_default_device() {
        Ok(device) => {
            println!("Using device {}", device.name);
            device.name
        }
        Err(err) => {
            eprintln!("Using device unavailable ({err})");
            String::new()
        }
    }
}
