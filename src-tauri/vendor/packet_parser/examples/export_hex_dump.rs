use packet_parser::convert::Packet;

fn main() {
    let hex_dump_data = "feaa81e86d1efeaa818ec864080045500034000000003d06206b36e6700dac140a0201bbc1087d7f02aa4e2b998e80100081748300000101080a9373c9c207ef14e3";

    let packet = Packet::from(hex_dump_data);

    // Générer un fichier .pcap
    if let Err(e) = packet.packet_to_pcap() {
        eprintln!("❌ Erreur lors de l'export du fichier pcap : {}", e);
    }
}
