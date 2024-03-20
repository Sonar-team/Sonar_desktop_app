use std::{collections::HashMap, sync::Mutex};
use log::error;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use crate::{sniff::capture_packet::layer_2_infos::{layer_3_infos::ip_type::IpType, PacketInfos}, tauri_state::SonarState};


#[derive(Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

#[derive(Serialize, Clone)]
struct Node {
    name: String,
    color: String,
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



    fn add_edge(&mut self, packet: &PacketInfos) {
        let is_source_ip_private = matches!(packet.layer_3_infos.ip_source_type, Some(IpType::Private));
        let is_target_ip_private = matches!(packet.layer_3_infos.ip_destination_type, Some(IpType::Private));
    
        if let (Some(source_ip), true) = (packet.layer_3_infos.ip_source.clone(), is_source_ip_private) {
            if let (Some(target_ip), true) = (packet.layer_3_infos.ip_destination.clone(), is_target_ip_private) {
    
                // Déterminez la couleur basée sur si l'adresse IP se termine par '1'
                let source_color = if source_ip.ends_with('1') { "red".to_string() } else { "blue".to_string() };
                let target_color = if target_ip.ends_with('1') { "red".to_string() } else { "blue".to_string() };
    
                self.nodes.entry(source_ip.clone()).or_insert_with(|| Node { 
                    name: source_ip.clone(),
                    color: source_color,
                });
                self.nodes.entry(target_ip.clone()).or_insert_with(|| Node { 
                    name: target_ip.clone(),
                    color: target_color,
                });
    
                let label = packet.l_3_protocol.clone();
    
                let edge_exists = self.edges.values().any(|e| {
                    (e.source == source_ip && e.target == target_ip && e.label == label) ||
                    (e.source == target_ip && e.target == source_ip && e.label == label)
                });
    
                if !edge_exists {
                    let edge_name = format!("edge{}", self.edge_counter);
                    self.edges.insert(
                        edge_name,
                        Edge {
                            source: source_ip,
                            target: target_ip,
                            label,
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

