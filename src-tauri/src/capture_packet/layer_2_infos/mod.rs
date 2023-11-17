use std::fmt;

use pnet::packet::ethernet::EthernetPacket;
use pnet::util::MacAddr;

use layer_3_infos::{get_layer_3_infos, Layer3Infos};
mod layer_3_infos;

#[derive(Debug, Default)]
pub struct PacketInfos {
    mac_address_source: MacAddr,
    mac_address_destination: MacAddr,
    interface: String,
    l_3_protocol: String,
    layer_3_infos: Layer3Infos,
}

impl PacketInfos {
    pub fn new(interface_name: &String, ethernet_packet: &EthernetPacket<'_>) -> PacketInfos {
        PacketInfos {
            mac_address_source: ethernet_packet.get_source(),
            mac_address_destination: ethernet_packet.get_destination(),
            interface: interface_name.to_string(),
            l_3_protocol: ethernet_packet.get_ethertype().to_string(),
            layer_3_infos: get_layer_3_infos(ethernet_packet),
        }
    }
}

impl fmt::Display for PacketInfos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Implement the formatting for PacketInfos here
        writeln!(f, "MAC Source: {}", self.mac_address_source)?;
        writeln!(f, "MAC Destination: {}", self.mac_address_destination)?;
        writeln!(f, "L2 Interface: {}", self.interface)?;
        writeln!(f, "L 3 proto: {}", self.l_3_protocol)?;
        writeln!(
            f,
            "ip_source: {}",
            self.layer_3_infos.ip_source.as_deref().unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "ip_destination: {}",
            self.layer_3_infos
                .ip_destination
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "port_destination: {}",
            self.layer_3_infos
                .layer_4_infos
                .port_destination
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "port_source: {}",
            self.layer_3_infos
                .layer_4_infos
                .port_source
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "L 4 proto: {}",
            self.layer_3_infos.l_4_protocol.as_deref().unwrap_or("N/A")
        )?;
        // Format other fields as needed
        Ok(())
    }
}
