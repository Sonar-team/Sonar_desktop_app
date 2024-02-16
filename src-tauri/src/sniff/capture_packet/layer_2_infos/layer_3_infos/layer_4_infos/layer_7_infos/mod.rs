use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
struct DhcpHandler;

trait PacketPayload {
    fn get_protocol(data: &[u8]) -> String;
}

// Implémentez ce trait pour les tranches de données (data slices) qui pourraient contenir des paquets DHCP
impl PacketPayload for &[u8] {
    fn get_protocol(data: &[u8]) -> String {
        println!("data: {:?}", data);
        if is_dhcp_packet(data) {
            "DHCP".to_string()
        } else if is_dns_packet(data) {
            "DNS".to_string()
        } else if is_http_packet(data) {
            "HTTP".to_string()
        } else if is_ssh_packet(data) {
            "SSH".to_string()
        } else if is_modbus_packet(data) {
            "modbus".to_string()
        } else {
            "Unknown Protocol".to_string()
        }
    }
}

pub fn get_protocol(data: &[u8]) -> String {
    <&[u8] as PacketPayload>::get_protocol(data)
}

fn is_dhcp_packet(data: &[u8]) -> bool {
    const DHCP_MIN_LENGTH: usize = 244;
    const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];
    
    if data.len() < DHCP_MIN_LENGTH {
        return false;
    }

    if data[236..240] == DHCP_MAGIC_COOKIE {
        true
    } else {
        false
    }
}

fn is_modbus_packet(data: &[u8]) -> bool {
    // Vérifiez si la longueur des données est suffisante pour un en-tête Modbus TCP
    if data.len() < 7 {
        return false;
    }

    // Vérifiez si le champ protocole est zéro (octets à l'indice 2 et 3)
    if data[2] != 0 || data[3] != 0 {
        return false;
    }

    // Extrait la longueur du PDU à partir de l'en-tête et vérifie si elle correspond
    // à la taille réelle des données moins l'en-tête
    let pdu_length = ((data[4] as usize) << 8) | (data[5] as usize);
    if pdu_length != data.len() - 6 {
        return false;
    }

    // Si toutes les conditions ci-dessus sont remplies, les données pourraient être un paquet Modbus TCP
    true
}

fn is_http_packet(data: &[u8]) -> bool {
    // Convertit les données en chaîne pour une recherche simplifiée.
    // Attention : cela ne fonctionnera pas bien avec des données binaires non-ASCII.
    if let Ok(data_str) = std::str::from_utf8(data) {
        data_str.starts_with("GET ") || data_str.starts_with("POST ") || data_str.starts_with("HTTP/1.")
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
    // Très basique; les requêtes DNS ont généralement un QDCOUNT > 0 après les 12 premiers octets.
    // Ceci est hautement heuristique et non fiable pour une identification précise.
    data.len() > 12 && data[4] == 0 && data[5] > 0
}