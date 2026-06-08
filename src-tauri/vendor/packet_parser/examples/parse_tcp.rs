use hex::decode;
use packet_parser::{DataLink, Internet, Transport};

fn main() {
    let hex_dump_data = "0004170258b778e7d1e0025e08004500002879944000800600008d51000a8d510056ed1101f64f2ee0fc7c8069b6501400001b1d0000";

    // Convertir la chaîne hexadécimale en un Vec<u8>
    let packet = decode(hex_dump_data).expect("Conversion hexadécimale échouée");

    println!("{:X?}", packet);

    let datalink = match DataLink::try_from(packet.as_slice()) {
        Ok(datalink) => {
            println!("{}", datalink);
            datalink
        }
        Err(e) => {
            eprintln!("Erreur DataLink : {:?}", e);
            return;
        }
    };

    let internet = match Internet::try_from(datalink.payload) {
        Ok(internet) => {
            println!("{}", internet);
            internet
        }
        Err(e) => {
            eprintln!("Erreur Internet : {:?}", e);
            return;
        }
    };

    match Transport::try_from(internet.payload) {
        Ok(transport) => {
            println!("{}", transport);
        }
        Err(e) => {
            eprintln!("Erreur Transport : {:?}", e);
        }
    }

    match Transport::try_from_parts(internet.payload_protocol, internet.payload) {
        Ok(transport) => {
            println!("{}", transport);
        }
        Err(e) => {
            eprintln!("Erreur Transport : {:?}", e);
        }
    }
}
