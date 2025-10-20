use pcap::{Capture, Device, Error};

pub fn setup_capture(device: Device, buffer_size: i32) -> Result<Capture<pcap::Active>, Error> {
    Capture::from_device(device)?
        .promisc(false)
        .immediate_mode(true)
        .buffer_size(buffer_size)
        .open()
        
}
