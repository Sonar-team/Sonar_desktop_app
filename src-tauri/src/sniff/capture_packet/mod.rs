//! Module de capture des paquets réseau pour le projet Sonar.
//!
//! Fournit des fonctionnalités pour capturer le trafic réseau à travers une ou toutes les interfaces réseau.
//! Utilise `pnet` pour la capture des paquets et `tauri` pour l'intégration avec l'interface utilisateur.

use log::{error, info};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use std::sync::mpsc;
use std::thread;

use tauri::{Manager, State};
pub(crate) mod layer_2_infos;

use crate::tauri_state::SonarState;

use self::layer_2_infos::PacketInfos;

/// Capture le trafic réseau sur toutes les interfaces disponibles.
///
/// # Arguments
///
/// * `app` - Handle vers l'application Tauri, utilisé pour interagir avec l'interface utilisateur.
/// * `state` - État global de l'application, contenant les données capturées.

pub fn all_interfaces(app: tauri::AppHandle, state: State<SonarState>) {
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel::<PacketInfos>();

    let state_clone = state.0.clone();

    thread::spawn(move || {
        for new_packet in rx {
            let mut state_locked = state_clone.lock().expect("Failed to lock the mutex");
            
            let mut is_found = false;
            for (existing_packet, count) in state_locked.iter_mut() {
                // Définissez ici la logique pour déterminer si `new_packet` est "le même" que `existing_packet`.
                // Cela pourrait dépendre des adresses MAC, des adresses IP, du protocole, etc.
                if existing_packet.mac_address_source == new_packet.mac_address_source &&
                   existing_packet.mac_address_destination == new_packet.mac_address_destination &&
                   existing_packet.interface == new_packet.interface &&
                   existing_packet.l_3_protocol == new_packet.l_3_protocol &&
                   existing_packet.layer_3_infos == new_packet.layer_3_infos {
                    // Un paquet correspondant a été trouvé, incrémentez son compteur
                    *count += 1;
                    existing_packet.packet_size += new_packet.packet_size;
                    is_found = true;
                    break;
                }
            }

            if !is_found {
                // Si aucun paquet correspondant n'a été trouvé, ajoutez `new_packet` comme une nouvelle entrée
                state_locked.push((new_packet, 1));
            }
        }
    });

    // threads qui ecoute les trames
    let interfaces = datalink::interfaces();
    for interface in interfaces {
        let app2 = app.clone();
        let tx_clone = tx.clone();
        let handle = thread::spawn(move || {
            capture_packets(app2, interface, tx_clone);
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

/// Capture le trafic réseau sur une interface spécifique.
///
/// # Arguments
///
/// * `app` - Handle vers l'application Tauri.
/// * `interface` - Nom de l'interface réseau sur laquelle effectuer la capture.
/// * `state` - État global de l'application.
pub fn one_interface(app: tauri::AppHandle, interface: &str, state: State<SonarState>) {
    info!("L'interface choisie est: {}", interface);

    // thread fifo
    let (tx, rx) = mpsc::channel();

    // Clone the state for the thread
    let state_clone = state.0.clone();

    // Spawn a thread to process packets
    thread::spawn(move || {
        for packet in rx {
            let mut vector = state_clone.lock().expect("Failed to lock the mutex");

            // Vérification de l'existence du paquet dans le vecteur et mise à jour de son comptage et de sa taille
            let mut found = false;
            for (packet_info, count) in vector.iter_mut() {
                if *packet_info == packet {
                    *count += 1;
                    packet_info.packet_size += packet.packet_size;
                    found = true;
                    break;
                }
            }

            if !found {
                // Si le paquet n'existe pas déjà, ajout avec un compteur initialisé à 1
                vector.push((packet, 1));
            }
        }
    });

    let interface_names_match = |iface: &NetworkInterface| iface.name == interface;
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
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            error!("Type de canal non géré : {}", &interface);
            panic!("Type de canal non géré : {}", &interface)
        }
        Err(e) => {
            error!(
                "Une erreur s'est produite lors de la création du canal de liaison de données: {}",
                &interface
            );
            panic!(
                "Une erreur s'est produite lors de la création du canal de liaison de données: {}",
                e
            )
        }
    };
    let main_window = app.get_window("main").unwrap();

    info!(
        "Démarrage du thread de lecture de paquets sur l'interface :{}",
        &interface
    );
    loop {
        match rx.next() {
            Ok(packet) => {
                if let Some(ethernet_packet) = EthernetPacket::new(packet) {
                    let packet_info = PacketInfos::new(&interface.name, &ethernet_packet);
                    //println!("{packet_info}");
                    if let Err(err) = main_window.emit("frame", &packet_info) {
                        error!("Failed to emit event: {}", err);
                    }
                    if let Err(err) = tx.send(packet_info) {
                        error!("Failed to send packet to queue: {}", err);
                    }
                }
            }
            Err(e) => {
                error!("An error occurred while reading: {}", e);
                break;
            }
        }
    }
}

// fn process_packet(
//     state: tauri::State<SonarState>,
//     observed_packets: &mut HashSet<String>,
//     info: PacketInfos,
//     app: tauri::AppHandle
// ) {
//     //println!("{}", state);
//     let main_window = app.get_window("main").unwrap();
//     let mut ips = vec![info.layer_3_infos.ip_source.clone(), info.layer_3_infos.ip_source.clone()];
//     ips.sort();
//     let key = format!("{:?}-{:?}", ips[0], ips[1]);
//     if !observed_packets.contains(&key) {
//         //println!("New unique packet: {:?}", &info);
//         observed_packets.insert(key);

//         main_window.emit("matrice", &info).expect("Failed to emit event");
//         // Add the packet info to the vector
//         state.push_to_hash_map(info);
//         //println!("{} packets captured", state);

//     }
// }
