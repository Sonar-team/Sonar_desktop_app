use crate::{
    sniff::capture_packet::layer_2_infos::layer_3_infos::ip_type::IpType, tauri_state::PacketKey,
};

use serde::Serialize;
use std::collections::HashMap;

/// Récupère et sérialise les données de trafic réseau en une représentation de graph.
///
/// Cette fonction tente d'acquérir un verrou sur l'état partagé contenant les informations des paquets
/// et sérialise ces données en une chaîne JSON. Cela permet une transmission facile des données
/// pour la visualisation ou l'analyse ultérieure.
///
/// # Structures
///
/// `GraphData` représente les données de graph avec des nœuds et des arêtes.
/// `Node` représente un nœud dans le graph.
/// `Edge` représente une arête entre deux nœuds dans le graph.
/// `GraphBuilder` est utilisé pour construire les données de graph.
///
/// # Fonctionnement
///
/// La fonction `get_graph_data` construit les données de graph à partir des paquets de trafic réseau
/// et les sérialise en JSON.
///
/// # Exemple
///
/// Supposons que vous ayez un état partagé `shared_state` initialisé et passé à cette fonction :
///
/// ```ignore
/// let result = get_graph_data(shared_state);
/// match result {
///     Ok(json_string) => println!("Données sérialisées : {}", json_string),
///     Err(e) => eprintln!("Erreur : {}", e),
/// }
/// ```

#[derive(Serialize)]
pub struct GraphData {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

#[derive(Serialize, Clone)]
struct Node {
    name: String,
    color: String,
    mac: String,
}
#[derive(Serialize, Clone)]
struct Edge {
    source: String,
    target: String,
    label: String,
}

pub struct GraphBuilder {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
    edge_counter: u32,
}

impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            edge_counter: 1,
        }
    }

    // Function to determine the color of a node based on its IP type
    fn determine_color(ip_type: &Option<IpType>) -> String {
        match ip_type {
            Some(IpType::Private) => "#D4D3DC".to_string(), // Light gray for private IPs
            Some(IpType::Public) => "#317AC1".to_string(),  // Blue for public IPs
            _ => "#FF5733".to_string(), // Red for others (e.g., multicast or loopback, if they ever get through)
        }
    }

    // Fonction pour vérifier si une arête existe déjà
    pub fn edge_exists(&self, source_ip: &String, target_ip: &String, label: &String) -> bool {
        self.edges.values().any(|e| {
            (e.source == *source_ip && e.target == *target_ip && e.label == *label)
                || (e.source == *target_ip && e.target == *source_ip && e.label == *label)
        })
    }

    pub fn add_edge(&mut self, packet: &PacketKey) {
        if let (Some(source_ip), Some(target_ip)) = (
            &packet.layer_3_infos.ip_source,
            &packet.layer_3_infos.ip_destination,
        ) {
            let is_source_ip_private_or_public_ipv4 = matches!(
                packet.layer_3_infos.ip_source_type,
                Some(IpType::Private | IpType::Public)
            );
            let is_target_ip_private_or_public_ipv4 = matches!(
                packet.layer_3_infos.ip_destination_type,
                Some(IpType::Private | IpType::Public)
            );

            if is_source_ip_private_or_public_ipv4 && is_target_ip_private_or_public_ipv4 {
                // Use IP type to determine color
                let source_color = Self::determine_color(&packet.layer_3_infos.ip_source_type);
                let target_color = Self::determine_color(&packet.layer_3_infos.ip_destination_type);

                // Ensure nodes for source and target IPs are created
                self.nodes.entry(source_ip.clone()).or_insert_with(|| Node {
                    name: source_ip.clone(),
                    color: source_color,
                    mac: packet.mac_address_source.clone(),
                });
                self.nodes.entry(target_ip.clone()).or_insert_with(|| Node {
                    name: target_ip.clone(),
                    color: target_color,
                    mac: packet.mac_address_destination.clone(),
                });

                let label = &packet.l_3_protocol;

                // Add edge if it doesn't exist yet
                if !self.edge_exists(source_ip, target_ip, label) {
                    let edge_name = format!("edge{}", self.edge_counter);
                    self.edges.insert(
                        edge_name,
                        Edge {
                            source: source_ip.clone(),
                            target: target_ip.clone(),
                            label: label.clone(),
                        },
                    );
                    self.edge_counter += 1;
                }
            }
        }
    }

    pub fn build_graph_data(&self) -> GraphData {
        GraphData {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

// Helper function to determine if an IP address is IPv4
// fn is_ipv4(ip: &String) -> bool {
//     ip.contains('.') && !ip.contains(':')
// }
