// Ce module est responsable de transformer les informations de paquets réseau en une structure de données graphique,
// qui est ensuite utilisée à des fins de visualisation. Il utilise le framework Tauri pour construire des applications multiplateformes.
use std::{collections::HashMap, fmt, sync::Mutex};

use log::error;
use serde::Serialize;
use tauri::{AppHandle, Manager};

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

/// La structure GraphData contient les nœuds et les arêtes d'un graphe.
/// Chaque nœud représente une adresse MAC unique et chaque arête représente une transmission de paquet réseau
/// entre deux nœuds. L'arête inclut une étiquette indiquant le protocole de couche 3 utilisé.
#[derive(Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

/// La structure Node représente un nœud dans le graphe réseau.
/// Elle contient le nom du nœud, qui est l'adresse MAC d'un dispositif réseau.
#[derive(Serialize, Clone)]
struct Node {
    name: String,
}

/// La structure Edge représente une arête dans le graphe réseau.
/// Elle contient le nœud source, le nœud cible et une étiquette indiquant le protocole de couche 3.
#[derive(Serialize, Clone)]
struct Edge {
    source: String,
    target: String,
    label: String, // Added to include L3 protocol as a label
}

/// GraphBuilder est une structure utilitaire utilisée pour construire GraphData.
/// Elle maintient une collection de nœuds et d'arêtes et fournit des méthodes pour ajouter des nœuds et des arêtes au graphe.

struct GraphBuilder {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
    edge_counter: u32,
}

impl GraphBuilder {
    /// Crée une nouvelle instance de GraphBuilder avec des nœuds et des arêtes vides.
    fn new() -> Self {
        GraphBuilder {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            edge_counter: 1,
        }
    }

    /// Ajoute un nœud au graphe s'il n'existe pas déjà.
    /// Le nœud est identifié par une adresse MAC.
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

    /// Ajoute une arête entre deux nœuds. Si les nœuds n'existent pas, ils sont créés.
    /// Chaque arête est identifiée par un nom unique et inclut les adresses MAC source et cible,
    /// ainsi que l'étiquette du protocole de couche 3.
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

    /// Construit et retourne GraphData à partir de l'état actuel du constructeur.
    fn build_graph_data(&self) -> GraphData {
        GraphData {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

// Implémentation des traits Debug pour améliorer les capacités de journalisation et de débogage.
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

/// get_graph_data est une fonction publique qui traite les données partagées de paquets réseau
/// et les transforme en une représentation JSON de GraphData.
/// Elle acquiert un verrou sur l'état partagé, itère à travers les paquets réseau,
/// et peuple le graphe avec des nœuds et des arêtes.
pub fn get_graph_data(app: AppHandle) -> Result<String, String> {
    let state = app.state::<Mutex<SonarState>>(); // Acquire a lock
    let state_guard = state.lock().unwrap();
    let matrice = state_guard.get_matrice();

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
    serde_json::to_string(&graph_data).map_err(|e| {
        let err_msg = format!("Serialization error: {}", e);
        error!("{}", err_msg);
        err_msg
    })
}
