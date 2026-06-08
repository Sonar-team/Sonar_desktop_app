use hex::decode;
use packet_parser::PacketFlow;
use packet_parser::parse::application::protocols::dhcpv6::Dhcpv6Packet;
use std::convert::TryFrom;
fn main() {
    println!("=======================================================");
    println!("🧪 ENVIRONNEMENT DE TEST DHCPv6 (PARSER & ERRORS)");
    println!("=======================================================\n");

    // 1️⃣ Test: Parse un packet DHCPv6 complet encapsulé dans Ethernet -> IPv6 -> UDP
    println!("--- 1. Test Intégration complète (Couche 2 à Couche 7) ---");
    // Les champs longueurs (IPv6 et UDP) ont été corrigés à 0x001A (26 octets) pour concorder avec la taille réelle des options passées
    let full_dhcpv6_hex = "33330001000200112233445586dd60000000001a1140fe80000000000000021122fffe334455ff02000000000000000000000001000202220223001ae578011234560001000a00030001001122334455";
    let full_packet = decode(full_dhcpv6_hex).expect("Failed to decode hex");

    match PacketFlow::try_from(full_packet.as_slice()) {
        Ok(flow) => {
            println!("✅ Full PacketFlow parsé avec succès !");
            println!("   Application Protocol détecté: {:?}", flow.application);
            if let Some(app) = flow.application
                && app.application_protocol == "DHCPv6"
            {
                println!("   [C'est bien du DHCPv6!]");
            }
        }
        Err(e) => eprintln!("❌ Erreur inattendue: {:?}", e),
    }

    // 2️⃣ Test: Tester le module DHCPv6 directement avec un payload valid
    println!("\n--- 2. Test Unitaire Module (Payload Valide - Type 1 SOLICIT) ---");
    // Type (1 octet), Transaction ID (3 octets) = 01 12 34 56
    // Options = 0001 000a00030001001122334455
    let dhcp_payload_hex = "011234560001000a00030001001122334455";
    let valid_payload = decode(dhcp_payload_hex).unwrap();
    match Dhcpv6Packet::try_from(valid_payload.as_slice()) {
        Ok(packet) => {
            println!("✅ Payload DHCPv6 décodé :");
            println!("   > Type de Message : {}", packet.message_type);
            println!("   > ID Transaction  : {:06X}", packet.transaction_id);
            println!("   > Options brutes  : {:02X?}", packet.options);
        }
        Err(e) => eprintln!("❌ Erreur: {:?}", e),
    }

    // 3️⃣ Test: Tester l'erreur InvalidPacketLength
    println!("\n--- 3. Test Gestion d'Erreur (Payload trop court) ---");
    let short_payload_hex = "011234"; // Seulement 3 octets, doit échouer < 4
    let short_payload = decode(short_payload_hex).unwrap();
    match Dhcpv6Packet::try_from(short_payload.as_slice()) {
        Ok(_) => println!("❌ N'aurait pas dû réussir !"),
        Err(e) => println!("✅ Erreur capturée avec succès : {:?}", e),
    }

    // 4️⃣ Test: Tester le support des Agents de Relais
    println!("\n--- 4. Test d'un Agent de Relais (Type 12) ---");
    // Type 12 (0x0C) Relay-forward
    let relay_payload_hex = "0c000000000000000000000000000000";
    let relay_payload = decode(relay_payload_hex).unwrap();
    match Dhcpv6Packet::try_from(relay_payload.as_slice()) {
        Ok(packet) => {
            println!("✅ Payload Relais DHCPv6 (Type 12) décodé avec succès !");
            println!("   > Type de Message : {}", packet.message_type);
        }
        Err(e) => println!("❌ Erreur inattendue : {:?}", e),
    }

    // 5️⃣ Test: Tester l'erreur InvalidMessageType
    println!("\n--- 5. Test Gestion d'Erreur (Type de message inexistant) ---");
    // Type 14 (0x0E) n'est pas autorisé
    let invalid_type_hex = "0e1234560000";
    let invalid_payload = decode(invalid_type_hex).unwrap();
    match Dhcpv6Packet::try_from(invalid_payload.as_slice()) {
        Ok(_) => println!("❌ N'aurait pas dû réussir !"),
        Err(e) => println!("✅ Erreur capturée avec succès : {:?}", e),
    }
}
