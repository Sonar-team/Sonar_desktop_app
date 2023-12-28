use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ipv6::{Ipv6Packet, MutableIpv6Packet};
use pnet::packet::{MutablePacket, Packet};
use pnet::packet::ethernet::{EthernetPacket, MutableEthernetPacket, EtherTypes};
use std::env;

fn main() {
    let interface_name = "lo";  // Using loopback interface

    let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();

    let source_ip = "your_source_ipv6_addr"; // Replace with your source IPv6 address
    let dest_ip = "your_destination_ipv6_addr"; // Replace with your destination IPv6 address

    // Create a new channel, dealing with layer 2 packets
    let (mut tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred when creating the datalink channel: {}", e)
    };

    let mut ethernet_buffer = [0u8; 42]; // Ethernet header + IPv6 header
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).unwrap();

    // Set the Ethernet fields
    ethernet_packet.set_ethertype(EtherTypes::Ipv6).set_source(interface.mac.unwrap());
    ethernet_packet.set_destination(interface.mac.unwrap()); // Set correct destination MAC

    let mut ipv6_buffer = [0u8; 40]; // IPv6 header
    let mut ipv6_packet = MutableIpv6Packet::new(&mut ipv6_buffer).unwrap();

    // Set the IPv6 fields
    ipv6_packet.set_version(6);
    ipv6_packet.set_next_header(0); // 0 indicates Hop-by-Hop Options
    ipv6_packet.set_payload_length(0);
    ipv6_packet.set_source(source_ip.parse().unwrap());
    ipv6_packet.set_destination(dest_ip.parse().unwrap());

    // Set the payload - in this case, we don't have additional payload

    // Construct the final packet
    ethernet_packet.set_payload(ipv6_packet.packet_mut());

    // Send the packet
    tx.send_to(ethernet_packet.packet(), None);
}
