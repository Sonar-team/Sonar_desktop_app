use packet_parser::{owned::PacketFlowOwned, IpType};
use serde::Serialize;
use std::{collections::HashMap};

#[derive(Serialize)]
pub struct GraphData {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

use serde::ser::SerializeStruct;

#[derive(Clone)]
pub enum GraphUpdate {
    NodeAdded(Node),
    EdgeAdded(Edge),
}

impl serde::Serialize for GraphUpdate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct Wrapper<'a, T> {
            #[serde(rename = "type")]
            typ: &'static str,
            data: &'a T,
        }

        match self {
            GraphUpdate::NodeAdded(node) => {
                let wrapper = Wrapper {
                    typ: "NodeAdded",
                    data: node,
                };
                wrapper.serialize(serializer)
            }
            GraphUpdate::EdgeAdded(edge) => {
                let wrapper = Wrapper {
                    typ: "EdgeAdded",
                    data: edge,
                };
                wrapper.serialize(serializer)
            }
        }
    }
}

impl GraphData {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_packet_flow(&mut self, packet: &PacketFlowOwned) -> Vec<GraphUpdate> {
        let mut updates = Vec::new();
        
        // Récupération des adresses IP source et destination
        let (source_ip, target_ip) = match (
            packet.internet.as_ref().and_then(|i| i.source_ip),
            packet.internet.as_ref().and_then(|i| i.destination_ip),
        ) {
            (Some(s), Some(t)) => (s, t),
            _ => return updates,
        };

        // Vérification des IPs valides
        if !self.is_valid_ip(&packet.internet.as_ref().and_then(|i| i.ip_source_type.clone())) ||
           !self.is_valid_ip(&packet.internet.as_ref().and_then(|i| i.ip_destination_type.clone())) {
            return updates;
        }

        // Détermination des couleurs
        let source_color = self.determine_color(
            &packet.internet.as_ref().and_then(|i| i.ip_source_type.clone()),
        );
        let target_color = self.determine_color(
            &packet.internet.as_ref().and_then(|i| i.ip_destination_type.clone()),
        );

        let source_str = source_ip.to_string();
        let target_str = target_ip.to_string();

        // Gestion du nœud source
        if !self.nodes.contains_key(&source_str) {
            let new_node = Node::new(
                source_str.clone(),
                packet.data_link.source_mac.clone(),
                source_color.to_string(),
            );
            self.nodes.insert(source_str.clone(), new_node.clone());
            updates.push(GraphUpdate::NodeAdded(new_node));
        }

        // Gestion du nœud cible
        if !self.nodes.contains_key(&target_str) {
            let new_node = Node::new(
                target_str.clone(),
                packet.data_link.destination_mac.clone(),
                target_color.to_string(),
            );
            self.nodes.insert(target_str.clone(), new_node.clone());
            updates.push(GraphUpdate::NodeAdded(new_node));
        }

        // Création de l'arête si les deux nœuds existent
        if let (Some(source_node), Some(target_node)) = (self.nodes.get(&source_str), self.nodes.get(&target_str)) {
            let protocol = packet
                .internet
                .as_ref()
                .map(|i| i.protocol.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let edge_key = format!("{}:{}:{}", source_node.id, target_node.id, protocol);

            if !self.edges.contains_key(&edge_key) {
                let edge = Edge {
                    id: EDGE_COUNTER.fetch_add(1, Ordering::SeqCst).to_string(),
                    source: source_node.id.clone(),
                    target: target_node.id.clone(),
                    label: protocol,
                    source_port: packet.transport.as_ref().and_then(|t| t.source_port),
                    destination_port: packet.transport.as_ref().and_then(|t| t.destination_port),
                };

                self.edges.insert(edge_key, edge.clone());
                updates.push(GraphUpdate::EdgeAdded(edge));
            }
        }

        updates
    }

    fn is_valid_ip(&self, ip_type: &Option<IpType>) -> bool {
        !matches!(ip_type, Some(IpType::Unknown) | None)
    }

    fn determine_color(&self, ip_type: &Option<IpType>) -> String {
        match ip_type {
            Some(IpType::Private) => "#8BC34A".to_string(),   // Vert doux → réseau local
            Some(IpType::Public) => "#2196F3".to_string(),    // Bleu → IP publique
            Some(IpType::Multicast) => "#FFC107".to_string(), // Jaune → multicast classique
            Some(IpType::Loopback) => "#E53935".to_string(),  // Rouge vif → local à la machine
            Some(IpType::Apipa) => "#FF9800".to_string(),     // Orange → APIPA
            Some(IpType::LinkLocal) => "#FF5722".to_string(), // Orange foncé → lien local
            Some(IpType::Ula) => "#9C27B0".to_string(),       // Violet → ULA IPv6
            Some(IpType::Unknown) => "#9E9E9E".to_string(),   // Gris → inconnu
            Some(IpType::Documentation) => "#9E9E9E".to_string(), // Gris → documentation
            None => "#9E9E9E".to_string(),                    // Par défaut : gris
        }
    }

}

use std::sync::atomic::{AtomicU64, Ordering};


static NODE_COUNTER: AtomicU64 = AtomicU64::new(1);
static EDGE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Clone, Debug)]
pub struct Node {
    pub id: String,  // Chaîne pour correspondre au type NodeId attendu
    pub name: String,
    pub color: String,  // Chaîne au lieu de &'static str
    pub mac: String,
}

impl Node {
    pub fn new(name: String, mac: String, color: String) -> Self {
        let id = NODE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self { 
            id: id.to_string(), 
            name, 
            color,
            mac 
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
}

impl Edge {
    pub fn new(source: u64, target: u64) -> Self {
        let id = EDGE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: id.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            label: String::new(),
            source_port: None,
            destination_port: None,
        }
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn with_ports(mut self, source_port: Option<u16>, destination_port: Option<u16>) -> Self {
        self.source_port = source_port;
        self.destination_port = destination_port;
        self
    }
}
