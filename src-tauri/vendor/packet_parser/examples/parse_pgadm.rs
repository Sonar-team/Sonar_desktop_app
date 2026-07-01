use hex::decode;
use packet_parser::PacketFlow;
use std::convert::TryFrom;
use packet_parser::parse::application::protocols::postgresql::PostgreSqlPacket;

fn main() {
    println!("=======================================================");
    println!("🧪 ENVIRONNEMENT DE TEST PostgreSQL (PARSER & ERRORS)");
    println!("=======================================================\n");

    // 1️⃣ Test: Parse un packet DHCPv6 complet encapsulé dans Ethernet -> IPv6 -> UDP
    println!("--- 1. Test Intégration complète (Couche 2 à Couche 7) ---");
    // Les champs longueurs (IPv6 et UDP) ont été corrigés à 0x001A (26 octets) pour concorder avec la taille réelle des options passées
    let full_pgsql_hex = "00106f1803fa00106f1ae4420800450000a2bd4c4000400670d1ac135a0aac135a07d92515389676eac995d00f45801803eab49200000101080a005a2f9c005a28f55000000051005345542053455353494f4e20434841524143544552495354494353204153205452414e53414354494f4e2049534f4c4154494f4e204c4556454c205245414420434f4d4d4954544544000000420000000c0000000000000000450000000900000000015300000004";
    let full_packet = decode(full_pgsql_hex).expect("Failed to decode hex");

    match PacketFlow::try_from(full_packet.as_slice()) {
    Ok(flow) => {
        println!("✅ Packet parsé");

        if let Some(transport) = &flow.transport {
            if let Some(payload) = transport.payload {
                match PostgreSqlPacket::try_from(payload) {
                    Ok(pg) => {
                        println!("{:#?}", pg);

                        for (i, msg) in pg.messages.iter().enumerate() {
                            println!("--------------------------------");
                            println!("Message {}", i);
                            println!("Type   : {:?}", msg.message_type);
                            println!("Length : {}", msg.length);
                            println!("Body   : {:#?}", msg.body);
                        }
                    }
                    Err(e) => {
                        println!("Erreur PostgreSQL : {:?}", e);
                    }
                }
            }
        }
    }
    Err(e) => println!("{:?}", e),
}
}