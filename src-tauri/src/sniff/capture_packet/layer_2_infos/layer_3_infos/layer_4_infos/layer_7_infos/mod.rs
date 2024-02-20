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
    if data[UDP_HEADER_LENGTH + BOOTP_HEADER_LENGTH..UDP_HEADER_LENGTH + BOOTP_HEADER_LENGTH + 4]
        == DHCP_MAGIC_COOKIE
    {
        true
    } else {
        false
    }
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
    // Très basique; les requêtes DNS ont généralement un QDCOUNT > 0 après les 12 premiers octets.
    // Ceci est hautement heuristique et non fiable pour une identification précise.
    data.len() > 12 && data[4] == 0 && data[5] > 0
}
