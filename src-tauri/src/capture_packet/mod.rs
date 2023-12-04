use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use std::thread;

use tauri::Manager;

use layer_2_infos::PacketInfos;
mod layer_2_infos;

pub fn all_interfaces(app: tauri::AppHandle) {
    let interfaces = datalink::interfaces();
    let mut handles = vec![];
    

    for interface in interfaces {
        let app2 = app.clone();
        let handle = thread::spawn(move || {
            capture_packets(app2, interface);
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

pub fn one_interface(app: tauri::AppHandle, interface: &str) {
    println!("L'interface choisie est: {}", interface);
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface;
    let interfaces = datalink::interfaces();
    let captured_interface = match interfaces.into_iter().find(interface_names_match) {
        Some(interface) => interface,
        None => {
            panic!("No such interface '{}'", interface);
        }
    };
    capture_packets(app, captured_interface);
}

fn capture_packets(app: tauri::AppHandle, interface: datalink::NetworkInterface) {
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type: {}", &interface),
        Err(e) => panic!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    };
    let main_window = app.get_window("main").unwrap();

    println!("Start thread reading packet on interface: {}", &interface);
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    println!("---");
                    let packet_info = PacketInfos::new(&interface.name, &ethernet_packet);
                    println!("{}", packet_info);
                    main_window.emit("frame", packet_info).expect("Failed to emit event");

                }
            }
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

pub fn get_interfaces() -> Vec<String> {
    
    let interfaces = datalink::interfaces();
    println!("Fetching network interfaces");

    let mut names: Vec<String> = interfaces.iter().map(|iface| {
        let name = iface.name.clone();
        println!("Found interface: {}", name);
        name
    }).collect();
    let all = String::from("all");
    names.push(all);

    names
}