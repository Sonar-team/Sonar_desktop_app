use std::{collections::HashMap, fmt};

use log::error;
use serde::Serialize;
use tauri::State;

use crate::tauri_state::SonarState;

// [(PacketInfos  {
//     mac_address_source: "2c:fd:a1:60:a1:83",       mac_address_destination: "f4:05:95:5b:58:4c",
//     interface: "wlp6s0",
//     l_3_protocol: "Arp",
//     layer_3_infos: Layer3Infos {
//         ip_source: Some("192.168.1.254"),
//         ip_destination: Some("192.168.1.254"),
//         l_4_protocol: None,
//         layer_4_infos: Layer4Infos {
//             port_source: None,
//             port_destination: None } } },
// 1),

// (PacketInfos {
//     mac_address_source: "f4:05:95:5b:58:4c",
//     mac_address_destination: "2c:fd:a1:60:a1:83",
//     interface: "wlp6s0",
//     l_3_protocol: "Arp",
//     layer_3_infos: Layer3Infos {
//         ip_source: Some("192.168.1.20"),
//         ip_destination: Some("192.168.1.20"),
//         l_4_protocol: None,
//         layer_4_infos: Layer4Infos {
//             port_source: None,
//             port_destination: None } } },
// 1)
// ]
/// to get this
// graphData: {
//     nodes : {
//          node1: {
//              name: "2c:fd:a1:60:a1:83" },
//          node2: {
//              name: "f4:05:95:5b:58:4c" },
//     edges {
//       edges: {
//           source: “node1”,
//           target: “node2”
//           label: l_3_protocol}
//   }

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
    label: String, // Added to include L3 protocol as a label
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
        if !self.nodes.contains_key(&mac_address) {
            self.nodes.insert(
                mac_address.clone(),
                Node {
                    name: mac_address.clone(),
                },
            );
        }
    }

    fn add_edge(&mut self, source_mac: String, target_mac: String, label: String) {
        self.add_node(source_mac.clone());
        self.add_node(target_mac.clone());

        let edge_name = format!("edge{}", self.edge_counter);
        if !self.edges.contains_key(&edge_name) {
            self.edges.insert(
                edge_name.clone(),
                Edge {
                    source: source_mac.clone(),
                    target: target_mac.clone(),
                    label,
                },
            );
            self.edge_counter += 1;
        }
    }

    fn build_graph_data(&self) -> GraphData {
        GraphData {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

impl fmt::Debug for GraphData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "graphData {{ nodes {{")?;
        for (key, value) in &self.nodes {
            writeln!(f, "    {}: {{ name: \"{}\" }},", key, value.name)?;
        }
        writeln!(f, "  }}, edges  {{")?;
        for (key, value) in &self.edges {
            writeln!(
                f,
                "    {}: {{ source: \"{}\", target: \"{}\", label: \"{}\" }},",
                key, value.source, value.target, value.label
            )?;
        }
        writeln!(f, "  }} }}")
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Node {{ name: \"{}\" }}", self.name)
    }
}

impl fmt::Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Edge {{ source: \"{}\", target: \"{}\" }}",
            self.source, self.target
        )
    }
}

pub fn get_graph_data(shared_vec_infopackets: State<SonarState>) -> Result<String, String> {
    // Attempt to acquire the lock on the shared state
    match shared_vec_infopackets.0.lock() {
        Ok(matrice) => {
            let mut graph_builder = GraphBuilder::new();

            // Process your packet data here to populate nodes and edges
            for (packet, _) in matrice.iter() {
                let source_mac = packet.mac_address_source.clone();
                let target_mac = packet.mac_address_destination.clone();
                let l3_protocol_label = packet.l_3_protocol.clone(); // Assume this is a String

                graph_builder.add_edge(source_mac, target_mac, l3_protocol_label);
            }

            let graph_data = graph_builder.build_graph_data();

            // Serialize the GraphData to a JSON string
            let json_data = serde_json::to_string(&graph_data).map_err(|e| {
                let err_msg = format!("Serialization error: {}", e);
                error!("{}", err_msg);
                err_msg
            })?;
            //println!("{:?}", graph_data);

            Ok(json_data)
        }
        Err(_) => {
            let err_msg = "Failed to lock the mutex".to_string();
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}
