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
//! Ce module contient également des tests pour la fonction `update_state_with_packet` et la fonction `capture_packets`.

use log::{error, info};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::EthernetPacket;
use std::sync::{mpsc, Arc, Mutex};
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
            update_state_with_packet(state_clone.clone(), new_packet);
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
        for new_packet in rx {
            update_state_with_packet(state_clone.clone(), new_packet);
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

/// Met à jour l'état avec les informations d'un nouveau paquet.
///
/// Cette fonction prend un état partagé et un `PacketInfos` en tant que nouveau paquet.
/// Elle parcourt l'état pour trouver un paquet existant qui correspond au nouveau paquet
/// (basé sur certaines propriétés comme les adresses MAC source et destination, l'interface,
/// le protocole de couche 3, etc.). Si un tel paquet est trouvé, la fonction met à jour
/// le compteur et la taille du paquet pour cet élément. Sinon, elle ajoute le nouveau paquet
/// comme une nouvelle entrée dans l'état.
///
/// # Arguments
///
/// * `state` - Un `Arc<Mutex<Vec<(PacketInfos, u32)>>>` qui représente l'état partagé contenant
///             les paquets capturés et leurs compteurs.
/// * `new_packet` - Un `PacketInfos` représentant le nouveau paquet capturé à ajouter ou mettre à jour dans l'état.
///
/// # Panics
///
/// Panique si le verrou sur l'état ne peut pas être acquis.
///
fn update_state_with_packet(state: Arc<Mutex<Vec<(PacketInfos, u32)>>>, new_packet: PacketInfos) {
    let mut state_locked = state.lock().expect("Failed to lock the mutex");

    let mut is_found = false;
    for (existing_packet, count) in state_locked.iter_mut() {
        // Définissez ici la logique pour déterminer si `new_packet` est "le même" que `existing_packet`.
        // Cela pourrait dépendre des adresses MAC, des adresses IP, du protocole, etc.
        if existing_packet.mac_address_source == new_packet.mac_address_source
            && existing_packet.mac_address_destination == new_packet.mac_address_destination
            && existing_packet.interface == new_packet.interface
            && existing_packet.l_3_protocol == new_packet.l_3_protocol
            && existing_packet.layer_3_infos == new_packet.layer_3_infos
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_update_state_with_packet() {
        let state = Arc::new(Mutex::new(vec![]));
        let buffer = vec![0u8; 64]; // Local buffer
        let ethernet_packet = EthernetPacket::new(&buffer).unwrap();
        let packet = PacketInfos::new(&String::from("eth0"), &ethernet_packet);

        // Add a packet to the state and verify it
        update_state_with_packet(state.clone(), packet.clone());
        assert_eq!(state.lock().unwrap().len(), 1);

        // Add the same packet again and verify that the count is incremented
        update_state_with_packet(state.clone(), packet.clone());
        assert_eq!(state.lock().unwrap().len(), 1);
        assert_eq!(state.lock().unwrap()[0].1, 2);

        // Add a different packet and verify that it's added as a new entry
        let different_packet = PacketInfos::new(&String::from("eth2"), &ethernet_packet);
        update_state_with_packet(state.clone(), different_packet.clone());
        assert_eq!(state.lock().unwrap().len(), 2);
    }

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
}
