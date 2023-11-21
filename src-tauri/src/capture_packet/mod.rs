use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;

use tauri::Manager;

use layer_2_infos::PacketInfos;
mod layer_2_infos;

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

pub fn all_interfaces(app: tauri::AppHandle) {
    let interfaces = datalink::interfaces();
    let mut handles = vec![];
    let matice_set = Arc::new(Mutex::new(HashSet::new())); // Thread-safe HashSet
    for interface in interfaces {
        let app2 = app.clone();
        let matice_set_clone = Arc::clone(&matice_set); // Clone the Arc
        let handle = thread::spawn(move || {
            capture_packets(app2, interface, &matice_set_clone);
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
    let matice_set = Arc::new(Mutex::new(HashSet::new())); // Thread-safe HashSet
    let interface_names_match = |iface: &NetworkInterface| iface.name == interface;
    let interfaces = datalink::interfaces();
    let captured_interface = match interfaces.into_iter().find(interface_names_match) {
        Some(interface) => interface,
        None => {
            panic!("No such interface '{}'", interface);
        }
    };
    capture_packets(app, captured_interface, &matice_set);
}

fn capture_packets(app: tauri::AppHandle, interface: datalink::NetworkInterface, matice_set: &Arc<Mutex<HashSet<String>>>) {
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
                    //println!("{}", packet_info);
                    main_window.emit("frame", &packet_info).expect("Failed to emit event");
                    //process_packet(&matice_set, &packet_info)
                }
            }
            Err(e) => {
                panic!("An error occurred while reading: {}", e);
            }
        }
    }
}

fn process_packet(
    observed_packets: &Arc<Mutex<HashSet<String>>>,
    info: &PacketInfos
) {
    let mut observed_packets = observed_packets.lock().unwrap(); // Lock the mutex

    let mut ips = vec![info.layer_3_infos.ip_source.clone(), info.layer_3_infos.ip_destination.clone()];
    ips.sort(); 
    let key = format!("{:?}-{:?}", ips[0], ips[1]);

    if !observed_packets.contains(&key) {
        println!("New unique packet: {:?}", &info);
        observed_packets.insert(key);
    }
    else {
        // Count or other logic
    }
}

