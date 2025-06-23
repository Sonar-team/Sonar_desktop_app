use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::tauri_state::{
    capture::capture_handle::layer_2_infos::layer_3_infos::ip_type::IpType, matrice::PacketKey,
};

#[derive(Serialize)]
pub struct GraphData<'a> {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge<'a>>,
}

#[derive(Serialize, Clone)]
struct Node {
    name: String,
    color: String,
    mac: String,
}

#[derive(Serialize, Clone)]
struct Edge<'a> {
    source: String,
    target: String,
    label: Cow<'a, str>,
}

pub struct GraphBuilder<'a> {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge<'a>>,
    edge_counter: u32,
}

impl<'a> GraphBuilder<'a> {
    pub fn new() -> Self {
        GraphBuilder {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            edge_counter: 1,
        }
    }

    fn determine_color(ip_type: &Option<IpType>) -> String {
        match ip_type {
            Some(IpType::Private) => "#D4D3DC".to_string(),
            Some(IpType::Public) => "#317AC1".to_string(),
            Some(
                IpType::Multicast
                | IpType::Loopback
                | IpType::Unknown
                | IpType::Apipa
                | IpType::LinkLocal
                | IpType::Ula,
            ) => "#FF5733".to_string(),
            _ => "#FF5733".to_string(),
        }
    }

    pub fn edge_exists(&self, source_ip: &String, target_ip: &String, label: &str) -> bool {
        self.edges.values().any(|e| {
            (e.source == *source_ip && e.target == *target_ip && e.label == label)
                || (e.source == *target_ip && e.target == *source_ip && e.label == label)
        })
    }

    pub fn add_edge(&mut self, packet: &'a PacketKey) {
        if let (Some(source_ip), Some(target_ip)) = (
            &packet.layer_3_infos.ip_source,
            &packet.layer_3_infos.ip_destination,
        ) {
            let is_source_ip_valid = matches!(
                packet.layer_3_infos.ip_source_type,
                Some(IpType::Private | IpType::Public)
            );
            let is_target_ip_valid = matches!(
                packet.layer_3_infos.ip_destination_type,
                Some(IpType::Private | IpType::Public)
            );

            if is_source_ip_valid && is_target_ip_valid {
                let source_color = Self::determine_color(&packet.layer_3_infos.ip_source_type);
                let target_color = Self::determine_color(&packet.layer_3_infos.ip_destination_type);

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

                let label = if let Some(protocol) =
                    packet.layer_3_infos.layer_4_infos.l_7_protocol.as_deref()
                {
                    Cow::Borrowed(protocol)
                } else if let Some(protocol) =
                    packet.layer_3_infos.l_4_protocol.as_deref()
                {
                    Cow::Borrowed(protocol)
                } else {
                    Cow::Borrowed(packet.l_3_protocol.as_str())
                };

                if !self.edge_exists(source_ip, target_ip, &label) {
                    let edge_name = format!("edge{}", self.edge_counter);
                    self.edges.insert(
                        edge_name,
                        Edge {
                            source: source_ip.clone(),
                            target: target_ip.clone(),
                            label,
                        },
                    );
                    self.edge_counter += 1;
                }
            }
        }
    }

    pub fn build_graph_data(&self) -> GraphData<'a> {
        GraphData {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}
