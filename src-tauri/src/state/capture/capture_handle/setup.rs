use pcap::{Capture, Error};

pub fn setup_capture(config: (String, i32, i32, i32, i32)) -> Result<Capture<pcap::Active>, Error> {
    Capture::from_device(config.0.as_str())?
        .promisc(true)
        .snaplen(config.4)
        .immediate_mode(false)
        .timeout(config.3)
        .buffer_size(config.1)
        .open()
}
