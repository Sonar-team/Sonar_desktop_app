use crate::{
    sniff::capture_packet::layer_2_infos::{layer_3_infos::ip_type::IpType, PacketInfos},
    tauri_state::SonarState,
};
use log::error;
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct GraphData {
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

    // Function to determine the color of a node based on its IP type
    fn determine_color(ip_type: &Option<IpType>) -> String {
        match ip_type {
            Some(IpType::Private) => "#D4D3DC".to_string(), // Light gray for private IPs
            Some(IpType::Public) => "#317AC1".to_string(),  // Blue for public IPs
            _ => "#FF5733".to_string(), // Red for others (e.g., multicast or loopback, if they ever get through)
        }
    }

    // Fonction pour vérifier si une arête existe déjà
    fn edge_exists(&self, source_ip: &String, target_ip: &String, label: &String) -> bool {
        self.edges.values().any(|e| {
            (e.source == *source_ip && e.target == *target_ip && e.label == *label)
                || (e.source == *target_ip && e.target == *source_ip && e.label == *label)
        })
    }

    fn add_edge(&mut self, packet: &PacketInfos) {
        if let (Some(source_ip), Some(target_ip)) = (
            &packet.layer_3_infos.ip_source,
            &packet.layer_3_infos.ip_destination,
        ) {
            let is_source_ip_private_or_public_ipv4 = matches!(
                packet.layer_3_infos.ip_source_type,
                Some(IpType::Private | IpType::Public) if is_ipv4(source_ip)
            );
            let is_target_ip_private_or_public_ipv4 = matches!(
                packet.layer_3_infos.ip_destination_type,
                Some(IpType::Private | IpType::Public) if is_ipv4(target_ip)
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
    //println!("{:?}", serde_json::to_string(&graph_data).unwrap());

    serde_json::to_string(&graph_data).map_err(|e| {
        error!("Serialization error: {}", e);
        format!("Serialization error: {}", e)
    })
}

// Helper function to determine if an IP address is IPv4
fn is_ipv4(ip: &String) -> bool {
    ip.contains('.') && !ip.contains(':')
}
