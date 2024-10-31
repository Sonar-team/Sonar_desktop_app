//! Module de capture des paquets réseau pour le projet Sonar.
//!
//! Fournit des fonctionnalités pour capturer le trafic réseau à travers une ou toutes les interfaces réseau.
//! Utilise `pnet` pour la capture des paquets et `tauri` pour l'intégration avec l'interface utilisateur.
//!
//! ## Fonctions
//!
//! - [`all_interfaces`](fn.all_interfaces.html): Capture le trafic réseau sur toutes les interfaces disponibles.
//! - [`one_interface`](fn.one_interface.html): Capture le trafic réseau sur une interface spécifique.
//! - [`capture_packets`](fn.capture_packets.html): Fonction interne pour démarrer la capture des paquets sur une interface donnée.
//!
//! ## Tests
//!
//! Ce module contient également des tests pour la fonction `update_matrice_with_packet` et la fonction `capture_packets`.

use log::{error, info};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use std::process::exit;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager, State};
pub(crate) mod layer_2_infos;

use crate::tauri_state::SonarState;

use self::layer_2_infos::PacketInfos;

// use std::sync::atomic::{AtomicBool, AtomicUsize};

// struct ThreadStatus {
//     is_alive: AtomicBool,
//     packets_processed: AtomicUsize,
//     last_error: Mutex<Option<String>>, // Using Mutex to allow mutable access across threads
// }

/// Capture le trafic réseau sur toutes les interfaces disponibles.
///
/// # Arguments
///
/// * `app` - Handle vers l'application Tauri, utilisé pour interagir avec l'interface utilisateur.
/// * `state` - État global de l'application, contenant les données capturées.

pub fn all_interfaces(app: AppHandle) {
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel::<PacketInfos>();

    let state_clone: Arc<Mutex<SonarState>> = Arc::clone(&app.state());

    // Creer un thread pour traiter les paquets

    thread::spawn(move || {
        for new_packet in rx {
            // 1. lock the state
            let mut locked_state = state_clone
                .lock()
                .map_err(|_| "Failed to lock state".to_string())
                .unwrap();
            //println!("  mutex matrice: {:?}", &locked_state.matrice);
            // 2. update the state
            locked_state.update_matrice_with_packet(new_packet);
            //println!("  mutex matrice updated: {:?}", &locked_state.matrice);
        }
    });

    // threads qui ecoute les trames
    let interfaces = datalink::interfaces();
    for interface in interfaces {
        // Vérifier si le nom de l'interface n'est pas 'lo' avant de créer un thread
        if interface.name != "lo" {
            let app2 = app.clone();
            let tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                capture_packets(app2, interface, tx_clone);
            });
            handles.push(handle);
        }
    }
    // Wait for all threads to complete
    for handle in handles {
        match handle.join() {
            Ok(_) => (), // Thread completed without panicking
            Err(e) => eprintln!("A thread panicked: {:?}", e),
        }
    }
}

/// Capture le trafic réseau sur une interface spécifique.
///
/// # Arguments
///
/// * `app` - Handle vers l'application Tauri.
/// * `interface` - Nom de l'interface réseau sur laquelle effectuer la capture.
/// * `state` - État global de l'application.
pub fn one_interface(app: tauri::AppHandle, interface: &str) {
    info!("L'interface choisie est: {}", interface);
    let app_clone = app.clone();
    // Création d'un canal de communication de type FIFO
    let (tx, rx) = mpsc::channel();

    // Démarrer un thread pour traiter les paquets
    thread::spawn(move || {
        for new_packet in rx {
            let state: State<'_, Arc<Mutex<SonarState>>> = app_clone.state(); // Acquire a lock
            let mut locked_state = state
                .lock()
                .map_err(|_| "Failed to lock state".to_string())
                .unwrap();

            locked_state.update_matrice_with_packet(new_packet);
        }
    });

    let interface_names_match = |iface: &NetworkInterface| iface.name == interface || iface.mac.unwrap_or_default().to_string() == interface;
    let interfaces = datalink::interfaces();

    let captured_interface = match interfaces.into_iter().find(interface_names_match) {
        Some(interface) => interface,
        None => {
            error!("Aucune interface de ce type: '{}'", interface);
            panic!("Aucune interface de ce type: '{}'", interface);
        }
    };
    capture_packets(app, captured_interface, tx);
}

/// Fonction interne pour démarrer la capture des paquets sur une interface donnée.
///
/// # Arguments
///
/// * `app` - Handle vers l'application Tauri.
/// * `interface` - Interface réseau sur laquelle capturer les paquets.
/// * `tx` - Canal de transmission pour envoyer les informations de paquets capturés.

fn capture_packets(
    app: tauri::AppHandle,
    interface: datalink::NetworkInterface,
    tx: mpsc::Sender<PacketInfos>,
) {
    loop {
        let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => {
                println!("Type de canal non géré : {}", &interface);
                continue;
            }
            Err(e) => {
                println!(
                        "Une erreur s'est produite lors de la création du canal de liaison de données: {}. 
                        sudo setcap cap_net_raw,cap_net_admin=eip src-tauri/target/debug/sonar-desktop-app",
                        &e
                    );
                exit(1);
            }
        };
        let main_window = app.get_window("main").unwrap();

        println!(
            "Démarrage du thread de lecture de paquets sur l'interface :{}",
            &interface
        );

        loop {
            match rx.next() {
                Ok(packet) => {
                    //println!("      packet: {:?} ", packet);
                    if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                        // println!("      ethernet_packet: {:?} ", ethernet_packet);
                        let packet_info = PacketInfos::new(&interface.name, &ethernet_packet);
                        //println!("      capture_packet: {:?}", &packet_info);
                        let state: State<'_, Arc<Mutex<SonarState>>> = app.state(); // Acquire a lock
                        let state_guard = state.lock().unwrap();

                        let active_state = state_guard.actif;
                        if !active_state {
                            continue;
                        }
                        
                        //println!("{packet_info}");
                        if packet_info.l_3_protocol == "Ipv6" {
                            let filter_ipv6 = state_guard.filter_ipv6;
                            if !filter_ipv6 {
                                continue;
                            }
                        }
                        // afficher dans le composant bottom long
                        if let Err(err) = main_window.emit("frame", &packet_info) {
                            println!("Failed to emit event: {}", err);
                        }
                        // envoyer au thread qui met a jour la matrice
                        if let Err(err) = tx.send(packet_info) {
                            println!("Failed to send packet to queue: {}", err);
                        }
                    }
                }
                Err(e) => {
                    println!("Connexion perdu. redemarrage: {}", e);
                    thread::sleep(Duration::from_secs(6));
                    break;
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::{Arc, Mutex};

//     #[test]
//     fn test_update_matrice_with_packet() {
//         let state = Arc::new(Mutex::new(vec![]));
//         let buffer = vec![0u8; 64]; // Local buffer
//         let ethernet_packet = EthernetPacket::new(&buffer).unwrap();
//         let packet = PacketInfos::new(&String::from("eth0"), &ethernet_packet);

//         // Add a packet to the state and verify it
//         update_matrice_with_packet(state.clone(), packet.clone());
//         assert_eq!(state.lock().unwrap().len(), 1);

//         // Add the same packet again and verify that the count is incremented
//         update_matrice_with_packet(state.clone(), packet.clone());
//         assert_eq!(state.lock().unwrap().len(), 1);
//         assert_eq!(state.lock().unwrap()[0].1, 2);

//         // Add a different packet and verify that it's added as a new entry
//         let different_packet = PacketInfos::new(&String::from("eth2"), &ethernet_packet);
//         update_matrice_with_packet(state.clone(), different_packet.clone());
//         assert_eq!(state.lock().unwrap().len(), 2);
//     }

// #[test]
// fn test_capture_packets() {
//     // Create a mock channel
//     let (tx, rx) = mpsc::channel();
//     let tx_clone = tx.clone();
//     let app: tauri::Window;
//     // Spawn a thread to capture packets
//     let handle = thread::spawn(move || {
//         let interface = datalink::interfaces().into_iter().next().unwrap();
//         capture_packets(app.app_handle(),interface,tx_clone);
//     });

//     // Wait a short time to allow the capture thread to start
//     thread::sleep(Duration::from_secs(1));

//     // Send a mock packet through the channel and verify
//     let mock_eth_packet = mock_packet();
//     tx.send(PacketInfos::new(&String::from("eth0"), &mock_eth_packet)).unwrap();
//     let received_packet = rx.recv().unwrap();
//     assert_eq!(received_packet.interface.len(), 64);
//     assert_eq!(received_packet.l_3_protocol.len(), 64);
//     assert_eq!(received_packet.mac_address_destination.len(), 64);
//     assert_eq!(received_packet.mac_address_source.len(), 64);

//     // Clean up by joining the capture thread
//     handle.join().unwrap();
// }
// }
