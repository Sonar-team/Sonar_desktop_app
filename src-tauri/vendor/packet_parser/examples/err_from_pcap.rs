use packet_parser::PacketFlow;
use pcap::Capture;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pcap_file_path =
        Path::new("/home/erdt-cyber/rust/ICS-Security-Tools/pcaps/ModbusTCP/mb2.pcap");

    let mut cap = Capture::from_file(pcap_file_path)?;
    let mut id = 0;
    loop {
        let packet = match cap.next_packet() {
            Ok(p) => p,
            Err(pcap::Error::NoMorePackets) => break,
            Err(e) => {
                eprintln!("pcap read error: {e}");
                continue;
            }
        };

        let parsed = PacketFlow::try_from(packet.data);

        match parsed {
            Ok(flow) => {
                id += 1;
                let _owned = flow.to_owned();
            }
            Err(e) => {
                id += 1;
                eprintln!("parse error: {e} for packet {id}");
            }
        }
    }

    Ok(())
}
