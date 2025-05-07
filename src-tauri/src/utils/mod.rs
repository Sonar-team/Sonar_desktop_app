use pcap::Device;

pub fn return_device_lookup() -> String {
    let device = Device::lookup()
        .expect("device lookup failed")
        .expect("no device available");
    println!("Using default device {}", device.name);
    device.name
}
