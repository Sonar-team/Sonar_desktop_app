use std::{collections::HashMap, fmt, sync::Mutex};
use log::error;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use crate::{sniff::capture_packet::layer_2_infos::{layer_3_infos::ip_type::IpType, PacketInfos}, tauri_state::SonarState};

const BROADCAST_MAC: &str = "ff:ff:ff:ff:ff:ff";

#[derive(Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

#[derive(Serialize, Clone)]
struct Node {
    name: String,
}

#[derive(Serialize, Clone)]
struct Edge {
    source: String,
    target: String,
    label: String,
}

struct GraphBuilder {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
    edge_counter: u32,
}

impl GraphBuilder {
    fn new() -> Self {
        GraphBuilder {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            edge_counter: 1,
        }
    }

    fn add_node(&mut self, mac_address: String) {
        self.nodes.entry(mac_address.clone()).or_insert(Node { name: mac_address });
    }

    fn add_edge(&mut self, packet: &PacketInfos) {
        // Extrait les adresses MAC
        let source_mac = packet.mac_address_source.clone();
        let target_mac = packet.mac_address_destination.clone();
        
        // Vérifie si les adresses IP sont privées
        let is_source_ip_private = matches!(packet.layer_3_infos.ip_source_type, Some(IpType::Private));
        let is_target_ip_private = matches!(packet.layer_3_infos.ip_destination_type, Some(IpType::Private));
        
        // Construit les noms de nœud en fonction de la disponibilité et du type d'adresse IP
        let source_node_name = if is_source_ip_private {
            format!("{} ({})", source_mac, packet.layer_3_infos.ip_source.clone().unwrap_or_default())
        } else {
            source_mac.clone()
        };
        let target_node_name = if is_target_ip_private {
            format!("{} ({})", target_mac, packet.layer_3_infos.ip_destination.clone().unwrap_or_default())
        } else {
            target_mac.clone()
        };
    
        // Continue uniquement si les deux adresses IP sont privées
        if is_source_ip_private && is_target_ip_private {
            // Ajoute les nœuds avec les noms construits
            self.nodes.entry(source_node_name.clone()).or_insert(Node { name: source_node_name.clone() });
            self.nodes.entry(target_node_name.clone()).or_insert(Node { name: target_node_name.clone() });
        
            // Ajoute une arête entre les deux nœuds
            let label = packet.l_3_protocol.clone();
            let edge_name = format!("edge{}", self.edge_counter);
            self.edges.insert(
                edge_name,
                Edge {
                    source: source_node_name,
                    target: target_node_name,
                    label,
                },
            );
            self.edge_counter += 1;
        }
        // Si les adresses IP ne sont pas privées, la fonction se termine sans ajouter d'arête
    }
    

    fn build_graph_data(&self) -> GraphData {
        GraphData {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

pub fn get_graph_data(app: AppHandle) -> Result<String, String> {
    let state = app.state::<Mutex<SonarState>>();
    let state_guard = state.lock().unwrap();
    let matrice = state_guard.get_matrice();

    let mut graph_builder = GraphBuilder::new();

    for (packet, _) in matrice.iter() {
        graph_builder.add_edge(packet);
    }

    let graph_data = graph_builder.build_graph_data();
    serde_json::to_string(&graph_data).map_err(|e| {
        error!("Serialization error: {}", e);
        format!("Serialization error: {}", e)
    })
}

