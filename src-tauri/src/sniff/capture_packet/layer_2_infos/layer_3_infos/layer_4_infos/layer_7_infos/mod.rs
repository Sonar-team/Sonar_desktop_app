use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
struct DhcpHandler;

trait PacketPayload {
    fn get_protocol(data: &[u8]) -> Option<String>;
}

// Implémentez ce trait pour les tranches de données (data slices) qui pourraient contenir des paquets DHCP
impl PacketPayload for &[u8] {
    fn get_protocol(data: &[u8]) -> Option<String> {
        println!("data: {:?}", data);
        if is_dhcp_packet(data) {
            Some("DHCP".to_string())
        } else if is_dns_packet(data) {
            Some("DNS".to_string())
        } else if is_http_packet(data) {
            Some("HTTP".to_string())
        } else if is_ssh_packet(data) {
            Some("SSH".to_string())
        } else if is_modbus_packet(data) {
            Some("modbus".to_string())
        } else if is_quic_packet(data) {
            Some("quic".to_string())
        } else if is_tls_v1_2_packet(data) {
            Some("tlsv1.2".to_string())
        } else {
            None
        }
    }
}

pub fn get_protocol(data: &[u8]) -> Option<String> {
    <&[u8] as PacketPayload>::get_protocol(data)
}

fn is_dhcp_packet(data: &[u8]) -> bool {
    const DHCP_MIN_LENGTH: usize = 244;
    const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];
    const UDP_HEADER_LENGTH: usize = 8;
    const BOOTP_HEADER_LENGTH: usize = 236;

    // Ports UDP pour DHCP
    const DHCP_SERVER_PORT: u16 = 67;
    const DHCP_CLIENT_PORT: u16 = 68;

    if data.len() < DHCP_MIN_LENGTH + UDP_HEADER_LENGTH {
        return false;
    }

    // Extraire les ports source et destination (les 2 premiers champs du header UDP)
    let src_port = ((data[0] as u16) << 8) | (data[1] as u16);
    let dst_port = ((data[2] as u16) << 8) | (data[3] as u16);

    // Vérifier si les ports correspondent à ceux utilisés par DHCP
    if src_port != DHCP_SERVER_PORT && dst_port != DHCP_CLIENT_PORT {
        return false;
    }

    // Vérifier la présence du Magic Cookie à la position correcte
    data[UDP_HEADER_LENGTH + BOOTP_HEADER_LENGTH..UDP_HEADER_LENGTH + BOOTP_HEADER_LENGTH + 4] == DHCP_MAGIC_COOKIE
}

fn is_modbus_packet(data: &[u8]) -> bool {
    const MODBUS_TCP_HEADER_LENGTH: usize = 7;

    if data.len() < MODBUS_TCP_HEADER_LENGTH {
        return false;
    }

    if data[2] != 0 || data[3] != 0 {
        return false;
    }

    let pdu_length = ((data[4] as usize) << 8) | (data[5] as usize);
    if pdu_length != data.len() - 6 {
        return false;
    }

    let function_code = data[7];
    if function_code == 0 || function_code > 127 {
        return false;
    }

    true
}

fn is_http_packet(data: &[u8]) -> bool {
    // Convertit les données en chaîne pour une recherche simplifiée.
    // Attention : cela ne fonctionnera pas bien avec des données binaires non-ASCII.
    if let Ok(data_str) = std::str::from_utf8(data) {
        data_str.starts_with("GET ")
            || data_str.starts_with("POST ")
            || data_str.starts_with("HTTP/1.")
    } else {
        false
    }
}

fn is_ssh_packet(data: &[u8]) -> bool {
    // Recherche de la bannière SSH initiale. Cela ne fonctionnera que pour le premier paquet d'une connexion SSH.
    if let Ok(data_str) = std::str::from_utf8(data) {
        data_str.starts_with("SSH-")
    } else {
        false
    }
}

fn is_dns_packet(data: &[u8]) -> bool {
    const DNS_HEADER_LENGTH: usize = 12;
    const QTYPE_LENGTH: usize = 2;
    const QCLASS_LENGTH: usize = 2;

    // Vérifier la longueur minimale pour contenir l'en-tête DNS
    if data.len() < DNS_HEADER_LENGTH {
        return false;
    }

    // Extraire qdcount (nombre de questions)
    let qdcount = ((data[4] as u16) << 8) | data[5] as u16;

    // Vérifier si qdcount est supérieur à 0
    if qdcount == 0 {
        return false;
    }

    // Vérifier le bit QR dans les flags pour s'assurer qu'il s'agit d'une requête
    let qr = (data[2] & 0b10000000) >> 7;
    if qr != 0 {
        return false;
    }

    // Commencer l'analyse après l'en-tête DNS
    let mut offset = DNS_HEADER_LENGTH;

    // Analyser chaque question
    for _ in 0..qdcount {
        // Trouver la fin du nom de domaine (0x00 marque la fin)
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }

        // Vérifier la présence de QTYPE et QCLASS après le nom de domaine
        if offset + 1 + QTYPE_LENGTH + QCLASS_LENGTH > data.len() {
            return false;
        }

        // Passer au-delà du QTYPE et du QCLASS pour la question suivante
        offset += 1 + QTYPE_LENGTH + QCLASS_LENGTH;
    }

    // Si toutes les vérifications sont passées, c'est probablement un paquet DNS
    true
}


fn is_quic_packet(data: &[u8]) -> bool {
    // Longueur minimale pour un en-tête QUIC
    const QUIC_MIN_HEADER_LENGTH: usize = 20;

    // Vérifier la longueur minimale
    if data.len() < QUIC_MIN_HEADER_LENGTH {
        return false;
    }

    // Vérifier le premier octet (flag)
    // Les paquets QUIC commencent par un octet spécifique
    let first_byte = data[0];
    if first_byte & 0x80 == 0 {  // Si le bit le plus significatif est 0, ce n'est pas un long header
        return false;
    }

    // Vérifier les bits réservés dans le long header (2e et 3e bits du premier octet)
    if first_byte & 0b00001100 != 0 {
        return false;
    }

    // Extraire la version
    let version = ((data[1] as u32) << 24)
                | ((data[2] as u32) << 16)
                | ((data[3] as u32) << 8)
                | data[4] as u32;

    // Liste des versions QUIC connues
    let known_versions = [0x00000001, 0x00000002, 0x00000003]; // Remplacer par les versions QUIC actuelles

    // Vérifier si la version est connue
    if !known_versions.contains(&version) {
        return false;
    }

    // Vous pouvez ajouter ici des vérifications supplémentaires si nécessaire

    true
}

fn is_tls_v1_2_packet(data: &[u8]) -> bool {
    // Vérifier la longueur minimale pour un header TLS
    if data.len() < 5 {
        return false;
    }

    let record_type = data[0];
    let version_major = data[1];
    let version_minor = data[2];

    // Vérifier le type de record (0x16 pour Handshake, 0x17 pour Application Data, etc.)
    // et la version du protocole (0x0303 pour TLS 1.2)
    record_type == 0x16 && version_major == 0x03 && version_minor == 0x03
}


