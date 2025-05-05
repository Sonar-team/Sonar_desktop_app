use pcap;

pub fn return_device_lookup() -> String {
    let device = pcap::Device::lookup()
        .expect("device lookup failed")
        .expect("no device available");
    println!("Using default device {}", device.name);
    device.name
}
