use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use std::thread;

use layer_2_infos::PacketInfos;
mod layer_2_infos;

pub fn all_interfaces() {
    let interfaces = datalink::interfaces();
    let mut handles = vec![];

    for interface in interfaces {
        let handle = thread::spawn(move || {
            capture_packets(interface);
        });
        handles.push(handle);
    }
    // Wait for all threads to complete
    for handle in handles {
        match handle.join() {
            Ok(_) => (), // Thread completed without panicking
            Err(e) => eprintln!("A thread panicked: {:?}", e),
        }
    }
}

pub fn one_interface(interface: &str) {
    println!("L'interface choisie est: {}", interface);
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface;
    let interfaces = datalink::interfaces();
    let captured_interface = match interfaces.into_iter().find(interface_names_match) {
        Some(interface) => interface,
        None => {
            panic!("No such interface '{}'", interface);
        }
    };
    capture_packets(captured_interface);
}

fn capture_packets(interface: datalink::NetworkInterface) {
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type: {}", &interface),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };

    println!("Start thread reading packet on interface: {}", &interface);
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    println!("---");
                    let packet_info = PacketInfos::new(&interface.name, &ethernet_packet);
                    println!("{}", packet_info);
                }
            }
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}
