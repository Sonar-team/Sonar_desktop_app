//! # Traitement des paquets de la couche 3
//!
//! Ce module implémente la logique pour extraire les informations de la couche 3 (et supérieures)
//! à partir de paquets Ethernet, en prenant en charge plusieurs protocoles comme IPv4, IPv6, ARP, et VLAN.
//!
//! ## Exemple d'utilisation
//!
//! ```rust
//! use pnet::packet::ethernet::EthernetPacket;
//! use layer_3_infos::get_layer_3_infos;
//!
//! // Exemple d'utilisation de la fonction get_layer_3_infos avec un paquet Ethernet
//! let ethernet_packet_data: &[u8] = &[/* données du paquet Ethernet */];
//! if let Some(ethernet_packet) = EthernetPacket::new(ethernet_packet_data) {
//!     let layer_3_infos = get_layer_3_infos(&ethernet_packet);
//!     println!("Layer 3 Infos: {:?}", layer_3_infos);
//! }
//! ```
//!
//! ## Structures
//!
//! - [`Layer3Infos`](struct.Layer3Infos.html): Représente les informations extraites de la couche 3 d'un paquet réseau.
//!
//! ## Traits
//!
//! - [`HandlePacket`](trait.HandlePacket.html): Trait définissant la fonctionnalité pour extraire les informations de la couche 3.
//!
//! ## Implémentations de Trait
//!
//! - [`HandlePacket`](trait.HandlePacket.html) est implémenté pour les types suivants:
//!     - [`Ipv4Handler`](struct.Ipv4Handler.html)
//!     - [`Ipv6Handler`](struct.Ipv6Handler.html)
//!     - [`ArpHandler`](struct.ArpHandler.html)
//!     - [`VlanHandler`](struct.VlanHandler.html)
//!     - [`PppoeDiscoveryHandler`](struct.PppoeDiscoveryHandler.html)
//!
//! ## Handlers de Paquets
//!
//! Les handlers de paquets sont des structures définies dans ce module et implémentent le trait [`HandlePacket`](trait.HandlePacket.html)
//! pour chaque type de paquet pris en charge.
//!
//! ## Fonctions
//!
//! - [`get_layer_3_infos`](fn.get_layer_3_infos.html): Fonction d'entrée pour traiter un paquet Ethernet et extraire les informations de la couche 3.
//!
//! ## Handlers de Paquets
//!
//! Les handlers de paquets sont des structures définies dans ce module et implémentent le trait [`HandlePacket`](trait.HandlePacket.html)
//! pour chaque type de paquet pris en charge.

use pnet::packet::{
    arp::ArpPacket,
    ethernet::{EtherTypes, EthernetPacket},
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    vlan::VlanPacket,
    Packet,
};

pub mod ip_type;
mod layer_4_infos;

use ip_type::IpType;
use layer_4_infos::{get_layer_4_infos, Layer4Infos};
use profinet_rt::ProfinetPacket;
use serde::Serialize;

use parse_mrp_packet::parse_mrp_data;

/// Représente les informations extraites de la couche 3 d'un paquet réseau.
#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct Layer3Infos {
    pub ip_source: Option<String>,
    pub ip_source_type: Option<IpType>,
    pub ip_destination: Option<String>,
    pub ip_destination_type: Option<IpType>,
    pub l_4_protocol: Option<String>,
    pub layer_4_infos: Layer4Infos,
}

// Définitions des handlers pour chaque type de paquet pris en charge...
struct Ipv4Handler;
struct Ipv6Handler;
struct ArpHandler;
struct VlanHandler;
struct PppoeDiscoveryHandler;
struct LldpHandler;
struct ProfinetHandler;
struct MrpHandler;

/// Trait définissant la fonctionnalité pour extraire les informations de la couche 3.
trait HandlePacket {
    fn get_layer_3(data: &[u8]) -> Layer3Infos;
}

impl HandlePacket for ProfinetHandler {
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        let ethernet_packet = EthernetPacket::new(data).unwrap();
        if let Some(_profinet_packet) = ProfinetPacket::new(ethernet_packet.payload()) {
            Layer3Infos {
                ip_source: Some(ethernet_packet.get_source().to_string()),
                ip_source_type: Some(IpType::Private), // Définissez un type approprié
                ip_destination: Some(ethernet_packet.get_destination().to_string()),
                ip_destination_type: Some(IpType::Private), // Définissez un type approprié
                l_4_protocol: None,
                layer_4_infos: Layer4Infos {
                    port_source: None,
                    port_destination: None,
                    l_7_protocol: None,
                },
            }
        } else {
            Default::default()
            
        }
    }
}

impl HandlePacket for Ipv4Handler {
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(ipv4_packet) = Ipv4Packet::new(data) {
            // Convertir les adresses IP source et destination en String
            let source_ip_str = ipv4_packet.get_source().to_string();
            let destination_ip_str = ipv4_packet.get_destination().to_string();

            // Utiliser les chaînes pour déterminer le type d'adresse IP
            let ip_source_type = IpType::from_ip(&source_ip_str);
            let ip_destination_type = IpType::from_ip(&destination_ip_str);

            Layer3Infos {
                ip_source: Some(source_ip_str),
                ip_source_type: Some(ip_source_type), // Correction ici
                ip_destination: Some(destination_ip_str),
                ip_destination_type: Some(ip_destination_type), // Correction ici
                l_4_protocol: Some(ipv4_packet.get_next_level_protocol().to_string()),
                layer_4_infos: get_layer_4_infos(
                    ipv4_packet.get_next_level_protocol(),
                    ipv4_packet.payload(),
                ),
            }
        } else {
            Default::default()
        }
    }
}

impl HandlePacket for Ipv6Handler {
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(ipv6_packet) = Ipv6Packet::new(data) {
            let ip_source = ipv6_packet.get_source().to_string();
            let ip_destination = ipv6_packet.get_destination().to_string();

            // Déterminez le type d'IP pour l'adresse source et destination
            let ip_source_type = IpType::from_ip(&ip_source);
            let ip_destination_type = IpType::from_ip(&ip_destination);

            Layer3Infos {
                ip_source: Some(ip_source),
                ip_source_type: Some(ip_source_type),
                ip_destination: Some(ip_destination),
                ip_destination_type: Some(ip_destination_type),
                l_4_protocol: Some(ipv6_packet.get_next_header().to_string()),
                layer_4_infos: get_layer_4_infos(
                    ipv6_packet.get_next_header(),
                    ipv6_packet.payload(),
                ),
            }
        } else {
            Default::default()
        }
    }
}

impl HandlePacket for ArpHandler {
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(arp_packet) = ArpPacket::new(data) {
            // Convertir les adresses proto (IP) en String pour déterminer leur type
            let ip_source = arp_packet.get_sender_proto_addr().to_string();
            let ip_destination = arp_packet.get_target_proto_addr().to_string();

            // Déterminer le type d'IP pour les adresses source et destination
            // Notez que ARP est principalement utilisé avec des adresses IPv4,
            // donc une logique spéciale pour ARP n'est peut-être pas nécessaire
            let ip_source_type = IpType::from_ip(&ip_source);
            let ip_destination_type = IpType::from_ip(&ip_destination);

            Layer3Infos {
                ip_source: Some(ip_source),
                ip_source_type: Some(ip_source_type),
                ip_destination: Some(ip_destination),
                ip_destination_type: Some(ip_destination_type),
                // ARP n'utilise pas de protocole de couche 4, donc on laisse cela comme None
                l_4_protocol: None,
                layer_4_infos: Layer4Infos {
                    port_source: None,
                    port_destination: None,
                    l_7_protocol: None,
                },
            }
        } else {
            Default::default()
        }
    }
}

impl HandlePacket for VlanHandler {
    /// Traite les paquets VLAN pour extraire les informations de la couche 3 et 4, y compris le support QinQ.
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(outer_vlan_packet) = VlanPacket::new(data) {
            // Check if the encapsulated packet is also a VLAN packet (QinQ)
            if outer_vlan_packet.get_ethertype() == EtherTypes::Vlan {
                if let Some(inner_vlan_packet) = VlanPacket::new(outer_vlan_packet.payload()) {
                    // Handle the encapsulated packet inside the inner VLAN tag
                    let encapsulated_ether_type = inner_vlan_packet.get_ethertype();
                    let encapsulated_data = inner_vlan_packet.payload();

                    match encapsulated_ether_type {
                        EtherTypes::Ipv4 => Ipv4Handler::get_layer_3(encapsulated_data),
                        EtherTypes::Ipv6 => Ipv6Handler::get_layer_3(encapsulated_data),
                        // Handle other types or default...
                        _ => Default::default(),
                    }
                } else {
                    // Handle case where inner VLAN packet is not valid
                    Default::default()
                }
            } else {
                // Process single VLAN-tagged packet as before
                let encapsulated_ether_type = outer_vlan_packet.get_ethertype();
                let encapsulated_data = outer_vlan_packet.payload();

                match encapsulated_ether_type {
                    EtherTypes::Ipv4 => Ipv4Handler::get_layer_3(encapsulated_data),
                    EtherTypes::Ipv6 => Ipv6Handler::get_layer_3(encapsulated_data),
                    // Handle other types or default...
                    _ => Default::default(),
                }
            }
        } else {
            Default::default()
        }
    }
}

impl HandlePacket for PppoeDiscoveryHandler {
    /// Traite les paquets PPPoE Discovery pour extraire les informations pertinentes.
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        // Here, you would parse the PPPoE Discovery packet.
        // This is a simplified example, as actual parsing would be more complex.
        if let Some(ethernet_packet) = EthernetPacket::new(data) {
            if ethernet_packet.get_ethertype() == EtherTypes::PppoeDiscovery {
                Layer3Infos {
                    ip_source: None,           // PPPoE packets do not have IP source/destination
                    ip_destination_type: None, // PPPoE packets do not have IP destination
                    ip_destination: None,
                    ip_source_type: None,
                    l_4_protocol: Default::default(),
                    layer_4_infos: Layer4Infos {
                        port_source: None,
                        port_destination: None,
                        l_7_protocol: None,
                    },
                }
            } else {
                Default::default()
            }
        } else {
            Default::default()
        }
    }
}

impl HandlePacket for LldpHandler {
    fn get_layer_3(_data: &[u8]) -> Layer3Infos {
        // Ici, vous devez implémenter la logique pour parser les paquets LLDP.
        // Comme LLDP est un protocole de la couche 2, les informations spécifiques de la couche 3 peuvent ne pas être disponibles.
        // Vous pourriez vouloir extraire des informations comme le nom de l'appareil, le port, la description, etc., et les stocker d'une manière qui a du sens pour votre application.

        Layer3Infos {
            // Ajustez selon les informations que vous pouvez extraire des paquets LLDP.
            ip_source: None,
            ip_destination_type: None,
            ip_destination: None,
            ip_source_type: None,
            l_4_protocol: None,
            layer_4_infos: Layer4Infos {
                port_source: None,
                port_destination: None,
                l_7_protocol: None,
            },
        }
    }
}

impl HandlePacket for MrpHandler {
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(_mrp_packet) = parse_mrp_data(data) {
            //println!("MRP packet: {:?}", mrp_packet);
            
            Layer3Infos { 
                ip_source: None,
                ip_destination_type: None,
                ip_destination: None,
                ip_source_type: None,
                l_4_protocol: None,
                layer_4_infos: Layer4Infos {
                    port_source: None,
                    port_destination: None,
                    l_7_protocol: None,
                },
            }
        } else {
            Default::default()
        }
    }
}

/// Fonction d'entrée pour traiter un paquet Ethernet et extraire les informations de la couche 3 en fonction du type EtherType.
pub fn get_layer_3_infos(ethernet_packet: &EthernetPacket<'_>) -> Layer3Infos {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv6 => Ipv6Handler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Ipv4 => Ipv4Handler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Arp => ArpHandler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Vlan => VlanHandler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::PppoeDiscovery => PppoeDiscoveryHandler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Lldp => LldpHandler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Profinet_rt => ProfinetHandler::get_layer_3(ethernet_packet.packet()), // Ajout pour Profinet
        EtherTypes::Mrp => MrpHandler::get_layer_3(ethernet_packet.payload()), // Ajout pour Mrp
        // EtherTypes::Ipx => todo!("Handle IPX packets"),
        // EtherTypes::AppleTalk => todo!("Handle AppleTalk packets"),
        // EtherTypes::Mpls => todo!("Handle MPLS packets"),
        // EtherTypes::MplsMcast => todo!("Handle MPLS multicast packets"),
        // EtherTypes::PBridge => todo!("Handle Provider Bridging packets"),
        // EtherTypes::Qnx => todo!("Handle QNX packets"),
        // EtherTypes::PppoeSession => todo!("Handle PPPoE Session packets"),
        // EtherTypes::Trill => todo!("Handle TRILL packets"),
        // EtherTypes::CobraNet => todo!("Handle CobraNet packets"),
        // EtherTypes::Aarp => todo!("Handle AppleTalk ARP packets"),
        // EtherTypes::QinQ => todo!("Handle QinQ packets"),
        // EtherTypes::FlowControl => todo!("Handle Flow Control packets"),
        _ => {
            // General case for all other EtherTypes
            println!(
                "Layer 3 - Unknown or unsupported packet type: {}, payload: {:?}, source: {}, eth_type {}, eth_type_nbr: {}",
                ethernet_packet.get_ethertype(), 
                ethernet_packet.payload(), 
                ethernet_packet.get_source(), 
                ethernet_packet.get_ethertype().to_string(), 
                ethernet_packet.get_ethertype().0
            );
            Default::default()
        }
    }
}
