use std::fmt;
use std::fmt::{Display, Formatter};

use pnet::packet::ethernet::EtherTypes::DECnet;
use serde::Serialize;
use pnet::packet::{dhcp::DhcpPacket, dns::DnsPacket};

#[derive(Default, Debug)]
pub struct Layer7Info<'a> {
    pub dns_info: Option<DnsPacket<'a>>,
    pub dhcp_info: Option<DhcpPacket<'a>>,
}


// impl Layer7Info {
//     pub fn to_string(&self) -> String {
//         format!("{}: {}", self.protocol, self.layer7_info)

//     }
// }



trait PacketPayload {
    fn get_protocol(data: &[u8]) -> Option<String>;
}

// Implémentez ce trait pour les tranches de données (data slices) qui pourraient contenir des paquets DHCP
impl PacketPayload for &[u8] {
    fn get_protocol(data: &[u8]) -> Option<String> {
        //println!("data: {:?}", data);
        if is_dns_packet(data) {
            Some("DNS".to_string())
        } else if  is_dhcp_packet(data){
            Some("DHCP".to_string())
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
    DhcpPacket::new(data).is_some()
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
    DnsPacket::new(data).is_some()
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
    if first_byte & 0x80 == 0 {
        // Si le bit le plus significatif est 0, ce n'est pas un long header
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


#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
struct DhcpHandler;

trait HandleLayer7 {
    fn get_layer7_infos(data: &[u8]) -> Layer7Info;
}

// impl HandleLayer7 for DhcpHandler {
//     fn get_layer7_infos(data: &[u8]) -> Layer7Info {
//         if let Some(dhcp_packet) = DhcpPacket::new(data) {
//             println!("DHCP packet detected: {:?}", dhcp_packet.get_flags());
//             Layer7Info {
//                 protocol: "DHCP".to_string(),
//                 //layer7_info: format!("{:?}", dhcp_packet),
//             }
//         } else {
//             Default::default()
//         }
//     }
// }

struct DnsHandler;

// impl HandleLayer7 for DnsHandler {
//     fn get_layer7_infos(data: &[u8]) -> Layer7Info {
//         if let Some(dns_packet) = DnsPacket::new(data) {
//             println!("DNS packet detected: id: {}", 
//                 dns_packet.get_id(),
             
//             );
//             Layer7Info {
//                 protocol: "DNS".to_string(),
//                 //layer7_info: format!("{:?}", dns_packet),
//             }
//         } else {
//             Default::default()
//         }
//     }
// }

pub fn get_layer7_infos(data: &[u8]) -> String {
    let dns_info = if let Some(dns_packet) = DnsPacket::new(data) {
        println!("DNS packet detected: {:?}", dns_packet.get_id());
        Some(dns_packet)
    } else {
        None
    };

    let dhcp_info = if let Some(dhcp_packet) = DhcpPacket::new(data) {
        println!("DHCP packet detected: {:?}", dhcp_packet.get_flags());
        Some(dhcp_packet)
    } else {
        None
    };

    let layer7 = Layer7Info {
        dns_info,
        dhcp_info,
        
    };
    println!("layer 7 infos: {:?} {:?}", layer7.dns_info.is_some(), layer7.dhcp_info.is_some());
    String::from("ok")
}
// if let Some(protocol) = get_protocol(data) {
//     match protocol.as_str() {
//         "DNS" => DnsHandler::get_layer7_infos(data),
//         "DHCP" => DhcpHandler::get_layer7_infos(data),
        
//         _ => {
//             // General case for all other EtherTypes
//             //println!("layer 7 - Unknown or unsupported packet type: {:?}", data);
//             Default::default()
//         },
        
//     }
// } else {
//     // General case for all other EtherTypes
//     //println!("layer 7 - Unknown or unsupported packet type: {:?}", data);
//     Default::default()
// }