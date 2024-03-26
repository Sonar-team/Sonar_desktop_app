//! Module pour gérer l'état de Sonar.
//!
//! Ce module fournit les structures nécessaires pour maintenir l'état
//! actuel de l'application Sonar, en particulier pour suivre les trames réseau.

use crate::sniff::capture_packet::layer_2_infos::PacketInfos;

/// `SonarState` encapsule l'état global de l'application Sonar.
///
/// Cette structure est conçue pour stocker et gérer les informations sur les trames réseau
/// capturées, y compris le comptage de leurs occurrences.
///
/// # Structure
/// `SonarState` contient un `Arc<Mutex<Vec<(PacketInfos, u32)>>>`.
/// - `Arc` permet un accès thread-safe et partagé à l'état.
/// - `Mutex` garantit que l'accès à l'état est mutuellement exclusif,
///   empêchant les conditions de concurrence.
/// - `Vec<(PacketInfos, u32)>` stocke les trames réseau (`PacketInfos`) et
///   leur nombre d'occurrences (`u32`).
///
/// # Exemple
/// ```
/// use std::sync::{Mutex, Arc};
/// use std::collections::HashMap;
/// use crate::capture_packet::layer_2_infos::PacketInfos;
/// use crate::SonarState;
///
/// let state = SonarState(Arc::new(Mutex::new(Vec::new())));
/// // Utilisez `state` ici pour gérer les trames réseau et leur comptage
/// ```

pub struct SonarState {
    // Contient les trames réseau et leur nombre d'occurrences
    pub matrice: Vec<(PacketInfos, u32)>,
    // Indique si le filtrage des adresses IPv6 est activé
    pub filter_ipv6: bool,
}

impl Default for SonarState {
    fn default() -> Self {
        Self::new()
    }
}

impl SonarState {
    // Constructeur pour initialiser `SonarState`
    pub fn new() -> SonarState {
        SonarState {
            matrice: Vec::new(),
            filter_ipv6: true, // Par défaut, le filtrage IPv6 est désactivé
        }
    }

    // Méthode pour basculer l'état de `filter_ipv6`
    pub fn toggle_filter_ipv6(&mut self) {
        self.filter_ipv6 = !self.filter_ipv6; // Inverse l'état actuel
    }

    // Getter method for matrice
    pub fn get_matrice(&self) -> &Vec<(PacketInfos, u32)> {
        &self.matrice
    }

    pub fn update_matrice_with_packet(&mut self, new_packet: PacketInfos) {
        let mut is_found = false;

        for (existing_packet, count) in self.matrice.iter_mut() {
            // Logique pour déterminer si `new_packet` est "le même" que `existing_packet`.
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
            self.matrice.push((new_packet, 1));
        }
    }
}
