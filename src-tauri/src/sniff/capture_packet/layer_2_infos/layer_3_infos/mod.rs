//! # Traitement des paquets de la couche 3
//!
//! Ce module implémente la logique pour extraire les informations de la couche 3 (et supérieures)
//! à partir de paquets Ethernet, en prenant en charge plusieurs protocoles comme IPv4, IPv6, ARP, et VLAN.

use pnet::packet::{
    arp::ArpPacket,
    ethernet::{
        EtherTypes::{self},
        EthernetPacket,
    },
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    vlan::VlanPacket,
    Packet,
};

mod layer_4_infos;
use layer_4_infos::{get_layer_4_infos, Layer4Infos};
use serde::Serialize;

/// Représente les informations extraites de la couche 3 d'un paquet réseau.
#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct Layer3Infos {
    pub ip_source: Option<String>,
    pub ip_destination: Option<String>,
    pub l_4_protocol: Option<String>,
    pub layer_4_infos: Layer4Infos,
}

// Définitions des handlers pour chaque type de paquet pris en charge.
struct Ipv4Handler;
struct Ipv6Handler;
struct ArpHandler;
struct VlanHandler;
struct PppoeDiscoveryHandler;

/// Trait définissant la fonctionnalité pour extraire les informations de la couche 3.
trait HandlePacket {
    fn get_layer_3(data: &[u8]) -> Layer3Infos;
}

impl HandlePacket for Ipv4Handler {
    /// Traite les paquets IPv4 pour extraire les informations de la couche 3 et 4.
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(ipv4_packet) = Ipv4Packet::new(data) {
            // //println!(
            //     "Layer 3: IPv4 packet: source {} destination {} => {} {}",
            //     ipv4_packet.get_source(),
            //     ipv4_packet.get_destination(),
            //     ipv4_packet.get_next_level_protocol(),
            //     ipv4_packet.get_total_length()
            // );
            //handle_next_proto_util(data, ipv4_packet.get_next_level_protocol());
            Layer3Infos {
                ip_source: Some(ipv4_packet.get_source().to_string()),
                ip_destination: Some(ipv4_packet.get_destination().to_string()),
                l_4_protocol: Some(ipv4_packet.get_next_level_protocol().to_string()),
                layer_4_infos: get_layer_4_infos(ipv4_packet.get_next_level_protocol(), data),
            }
        } else {
            Default::default()
        }
    }
}

impl HandlePacket for Ipv6Handler {
    /// Traite les paquets IPv6 pour extraire les informations de la couche 3 et 4.
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(ipv6_packet) = Ipv6Packet::new(data) {
            // println!(
            //     "Layer 3: IPv6 packet: source {} destination {} => {} {}",
            //     ipv6_packet.get_source(),
            //     ipv6_packet.get_destination(),
            //     ipv6_packet.get_next_header(),
            //     ipv6_packet.get_payload_length()
            // );
            Layer3Infos {
                ip_source: Some(ipv6_packet.get_source().to_string()),
                ip_destination: Some(ipv6_packet.get_destination().to_string()),
                l_4_protocol: Some(ipv6_packet.get_next_header().to_string()),
                layer_4_infos: get_layer_4_infos(ipv6_packet.get_next_header(), data),
            }
            //handle_next_proto_util(data, ipv6_packet.get_next_header());
        } else {
            // Handle the case when the data is not a valid IPv4 packet
            Default::default()
        }
    }
}

impl HandlePacket for ArpHandler {
    /// Traite les paquets ARP pour extraire les informations de la couche 3.
    fn get_layer_3(data: &[u8]) -> Layer3Infos {
        if let Some(arp_packet) = ArpPacket::new(data) {
            // println!(
            //     "Layer 2: arp packet: source {} destination {} => {:?} {} {} {:?} {} {}",
            //     arp_packet.get_sender_hw_addr(),
            //     arp_packet.get_target_hw_addr(),
            //     arp_packet.get_operation(),
            //     arp_packet.get_target_proto_addr(),
            //     arp_packet.get_sender_proto_addr(),
            //     arp_packet.get_hardware_type(),
            //     arp_packet.get_proto_addr_len(),
            //     arp_packet.packet().len()
            // );
            Layer3Infos {
                ip_source: Some(arp_packet.get_target_proto_addr().to_string()),
                ip_destination: Some(arp_packet.get_target_proto_addr().to_string()),
                l_4_protocol: Default::default(),
                layer_4_infos: Layer4Infos {
                    port_source: None,
                    port_destination: None,
                },
            }
        } else {
            // Handle the case when the data is not a valid IPv4 packet
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
                    ip_source: None, // PPPoE packets do not have IP source/destination
                    ip_destination: None,
                    l_4_protocol: Default::default(),
                    layer_4_infos: Layer4Infos {
                        port_source: None,
                        port_destination: None,
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

/// Fonction d'entrée pour traiter un paquet Ethernet et extraire les informations de la couche 3 en fonction du type EtherType.
pub fn get_layer_3_infos(ethernet_packet: &EthernetPacket<'_>) -> Layer3Infos {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv6 => Ipv6Handler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Ipv4 => Ipv4Handler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Arp => ArpHandler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Vlan => VlanHandler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::PppoeDiscovery => PppoeDiscoveryHandler::get_layer_3(ethernet_packet.payload()),
        _ => {
            // General case for all other EtherTypes
            println!(
                "Layer 3 - Unknown or unsupported packet type: {}",
                ethernet_packet.get_ethertype()
            );
            Default::default()
        }
    }
}
