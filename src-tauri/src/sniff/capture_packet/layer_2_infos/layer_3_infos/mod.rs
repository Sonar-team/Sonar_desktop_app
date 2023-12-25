use pnet::packet::{
    arp::ArpPacket,
    ethernet::{
        EtherTypes::{self},
        EthernetPacket,
    },
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    Packet,
};

mod layer_4_infos;
use layer_4_infos::{get_layer_4_infos, Layer4Infos};
use serde::Serialize;

#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct Layer3Infos {
    pub ip_source: Option<String>,
    pub ip_destination: Option<String>,
    pub l_4_protocol: Option<String>,
    pub layer_4_infos: Layer4Infos,
}

struct Ipv4Handler;
struct Ipv6Handler;
struct ArpHandler;

trait HandlePacket {
    fn get_layer_3(data: &[u8]) -> Layer3Infos;
}

impl HandlePacket for Ipv4Handler {
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

pub fn get_layer_3_infos(ethernet_packet: &EthernetPacket<'_>) -> Layer3Infos {
    match ethernet_packet.get_ethertype() {
        EtherTypes::Ipv6 => Ipv6Handler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Ipv4 => Ipv4Handler::get_layer_3(ethernet_packet.payload()),
        EtherTypes::Arp => ArpHandler::get_layer_3(ethernet_packet.payload()),
        _ => {
            // General case for all other EtherTypes
            println!(
                "Layer 3 - Unknown or unsupported packet type: {:?}",
                ethernet_packet.get_ethertype()
            );
            Default::default()
        }
    }
}
