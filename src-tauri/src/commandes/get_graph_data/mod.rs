use serde::Serialize;
use std::collections::HashMap;

use crate::tauri_state::{
    capture::capture_handle::layer_2_infos::layer_3_infos::ip_type::IpType, matrice::PacketKey,
};

#[derive(Serialize)]
pub struct GraphData {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

#[derive(Serialize, Clone)]
struct Node {
    name: String,
    color: &'static str,    // Static for color constants
    mac: String,
}

#[derive(Serialize, Clone)]
struct Edge {
    source: String,
    target: String,
    label: String,
    source_port: Option<String>,
    destination_port: Option<String>,
}

pub struct GraphBuilder {
    nodes: HashMap<String, Node>,
    edges: Vec<Edge>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, packet: &PacketKey) {
        if let (Some(source_ip), Some(target_ip)) = (
            packet.layer_3_infos.ip_source.as_deref(),
            packet.layer_3_infos.ip_destination.as_deref(),
        ) {
            // Skip if either IP is not valid
            if !self.is_valid_ip(&packet.layer_3_infos.ip_source_type) || 
               !self.is_valid_ip(&packet.layer_3_infos.ip_destination_type) {
                return;
            }

            // Get or create nodes
            let source_color = self.determine_color(&packet.layer_3_infos.ip_source_type);
            let target_color = self.determine_color(&packet.layer_3_infos.ip_destination_type);

            self.nodes.entry(source_ip.to_string()).or_insert_with(|| Node {
                name: source_ip.to_string(),
                color: source_color,
                mac: packet.mac_address_source.clone(),
            });

            self.nodes.entry(target_ip.to_string()).or_insert_with(|| Node {
                name: target_ip.to_string(),
                color: target_color,
                mac: packet.mac_address_destination.clone(),
            });

            // Get protocol label
            let label = packet.layer_3_infos.layer_4_infos.l_7_protocol.as_deref()
                .or_else(|| packet.layer_3_infos.l_4_protocol.as_deref())
                .unwrap_or_else(|| packet.l_3_protocol.as_str())
                .to_string();

            // Add edge if it doesn't exist
            if !self.edge_exists(source_ip, target_ip, &label) {
                self.edges.push(Edge {
                    source: source_ip.to_string(),
                    target: target_ip.to_string(),
                    label,
                    source_port: packet.layer_3_infos.layer_4_infos.port_source.clone(),
                    destination_port: packet.layer_3_infos.layer_4_infos.port_destination.clone(),
                });
            }
        }
    }

    fn is_valid_ip(&self, ip_type: &Option<IpType>) -> bool {
        matches!(ip_type, Some(IpType::Private | IpType::Public))
    }

    fn determine_color(&self, ip_type: &Option<IpType>) -> &'static str {
        match ip_type {
            Some(IpType::Private) => "#D4D3DC",
            Some(IpType::Public) => "#317AC1",
            Some(
                IpType::Multicast
                | IpType::Loopback
                | IpType::Unknown
                | IpType::Apipa
                | IpType::LinkLocal
                | IpType::Ula,
            ) => "#FF5733",
            _ => "#FF5733",
        }
    }

    fn edge_exists(&self, source: &str, target: &str, label: &str) -> bool {
        self.edges.iter().any(|e| {
            (e.source == source && e.target == target && e.label == label) ||
            (e.source == target && e.target == source && e.label == label)
        })
    }

    pub fn build_graph_data(self) -> GraphData {
        let edges = self.edges.into_iter()
            .enumerate()
            .map(|(i, edge)| (format!("edge{}", i + 1), edge))
            .collect();

        GraphData {
            nodes: self.nodes,
            edges,
        }
    }
}