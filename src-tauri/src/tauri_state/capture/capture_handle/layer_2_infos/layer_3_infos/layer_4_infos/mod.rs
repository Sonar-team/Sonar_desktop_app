use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use serde::Serialize;

mod layer_7_infos;
use layer_7_infos::get_layer7_infos;

#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct Layer4Infos {
    pub port_source: Option<String>,
    pub port_destination: Option<String>,
    pub l_7_protocol: Option<String>,
}

struct TcpHandler;
struct UdpHandler;
// struct IcmpHandler;
// struct Icmpv6Handler;
// struct Ipv6FragHandler;
// struct HopoptHandler;

trait HandleLayer4 {
    fn get_layer_4_infos(data: &[u8]) -> Layer4Infos;
}

impl HandleLayer4 for TcpHandler {
    fn get_layer_4_infos(data: &[u8]) -> Layer4Infos {
        if let Some(tcp_packet) = TcpPacket::new(data) {
            // println!("TCP packet detected: port source :{:?} - port destination :{:?}",
            //     tcp_packet.to_immutable().get_source(),
            //     tcp_packet.to_immutable().get_destination());

            //println!("{:?}", &layer7_info);
            Layer4Infos {
                port_source: Some(tcp_packet.get_source().to_string()),
                port_destination: Some(tcp_packet.get_destination().to_string()),
                l_7_protocol: get_layer7_infos(tcp_packet.payload()), // TCP does not inherently contain Layer 7 protocol information
            }
        } else {
            Default::default()
        }
    }
}

impl HandleLayer4 for UdpHandler {
    fn get_layer_4_infos(data: &[u8]) -> Layer4Infos {
        if let Some(udp_packet) = UdpPacket::new(data) {
            // println!("UDP packet detected: port source :{:?} - port destination :{:?}",
            //     udp_packet.to_immutable().get_source(),
            //     udp_packet.to_immutable().get_destination());

            //println!("{:?}", &layer7_info);
            Layer4Infos {
                port_source: Some(udp_packet.get_source().to_string()),
                port_destination: Some(udp_packet.get_destination().to_string()),
                l_7_protocol: get_layer7_infos(udp_packet.payload()), // UDP does not inherently contain Layer 7 protocol information
            }
        } else {
            Default::default()
        }
    }
}

pub fn get_layer_4_infos(proto: IpNextHeaderProtocol, data: &[u8]) -> Layer4Infos {
    match proto {
        IpNextHeaderProtocols::Tcp => TcpHandler::get_layer_4_infos(data),
        IpNextHeaderProtocols::Udp => UdpHandler::get_layer_4_infos(data),
        _ => {
            // General case for all other EtherTypes

            Default::default()
        }
    }
}
