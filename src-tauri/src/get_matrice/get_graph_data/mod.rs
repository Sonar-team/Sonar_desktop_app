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

    // Fonction pour déterminer la couleur d'un nœud basée sur son IP
    fn determine_color(ip: &String) -> String {
        if ip.ends_with(".1") {
            "#D4D3DC".to_string()
        } else {
            "#317AC1".to_string()
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
        let is_source_ip_private = matches!(packet.layer_3_infos.ip_source_type, Some(IpType::Private));
        let is_target_ip_private = matches!(packet.layer_3_infos.ip_destination_type, Some(IpType::Private));

        // Vérification supplémentaire pour s'assurer que les adresses IP sont bien IPv4 privées
        if is_source_ip_private && is_target_ip_private {
            if let (Some(source_ip), Some(target_ip)) = 
                (&packet.layer_3_infos.ip_source, &packet.layer_3_infos.ip_destination) {
                
                let source_color = Self::determine_color(source_ip);
                let target_color = Self::determine_color(target_ip);

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
