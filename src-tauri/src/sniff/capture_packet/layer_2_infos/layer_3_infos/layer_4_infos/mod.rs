// use pnet::packet::icmp::IcmpPacket;
// use pnet::packet::icmpv6::Icmpv6Packet;
//use pnet::packet::dhcp::DhcpPacket;
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use serde::Serialize;

use log::info;
mod layer_7_infos;
use layer_7_infos::get_protocol;

#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct Layer4Infos {
    pub port_source: Option<String>,
    pub port_destination: Option<String>,
    pub l_7_protocol: Option<String>
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
            let layer7_info = get_protocol(tcp_packet.packet());
            Layer4Infos {
                port_source: Some(tcp_packet.get_source().to_string()),
                port_destination: Some(tcp_packet.get_destination().to_string()),
                l_7_protocol: Some(layer7_info),
            }
        } else {
            Default::default()
        }
    }
}

impl HandleLayer4 for UdpHandler {
    fn get_layer_4_infos(data: &[u8]) -> Layer4Infos {
        if let Some(udp_packet) = UdpPacket::new(data) {
            let layer7_info = get_protocol(udp_packet.packet());
            Layer4Infos {
                port_source: Some(udp_packet.get_source().to_string()),
                port_destination: Some(udp_packet.get_destination().to_string()),
                l_7_protocol: Some(layer7_info), // UDP does not inherently contain Layer 7 protocol information
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
        IpNextHeaderProtocols::Icmp => Default::default(),
        IpNextHeaderProtocols::Icmpv6 => Default::default(),
        IpNextHeaderProtocols::Igmp => Default::default(),
        IpNextHeaderProtocols::Ipv6Frag => Default::default(),
        IpNextHeaderProtocols::Hopopt => Default::default(),
        _ => {
            // General case for all other EtherTypes
            info!("layer 4 - Unknown or unsupported packet type: {}", proto);
            Default::default()
        }
    }
}
