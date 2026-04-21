use hex::decode;
use packet_parser::{DataLink, MacAddress};

fn main() {
    let hex_dump_data = "feaa81e86d1efeaa818ec864080045500034000000003d06206b36e6700dac140a0201bbc1087d7f02aa4e2b998e80100081748300000101080a9373c9c207ef14e3";

    // Convertir la chaîne hexadécimale en un Vec<u8>
    let packet = decode(hex_dump_data).expect("Conversion hexadécimale échouée");

    // Afficher les octets du paquet en hexadécimal
    println!("{:X?}", packet);

    // Convertir en DataLink (supposé implémenter TryFrom<&[u8]>)
    match DataLink::try_from(packet.as_slice()) {
        Ok(datalink) => println!("{}", datalink),
        Err(e) => eprintln!("Erreur : {:?}", e),
    }
    let hex_dump_data = "feaa81e86d1efeaa818ec8640800";

    // Convertir la chaîne hexadécimale en un Vec<u8>
    let packet = decode(hex_dump_data).expect("Conversion hexadécimale échouée");

    // Afficher les octets du paquet en hexadécimal
    println!("{:X?}", packet);

    // Convertir en DataLink (supposé implémenter TryFrom<&[u8]>)
    match DataLink::try_from(packet.as_slice()) {
        Ok(datalink) => println!("{}", datalink),
        Err(e) => eprintln!("Erreur : {:?}", e),
    }

    let hex_dump_data = "feaa81e86d1e";
    let packet = decode(hex_dump_data).expect("Conversion hexadécimale échouée");

    let mac = MacAddress::try_from(packet.as_slice()).unwrap();
    println!("{}", mac.display_with_oui());
}
