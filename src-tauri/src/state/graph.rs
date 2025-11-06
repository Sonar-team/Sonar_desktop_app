use packet_parser::{IpType, owned::PacketFlowOwned};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Default)]
pub struct GraphData {
    // clé = IP (stringifiée) ou "mac:XX:XX:..."
    nodes: HashMap<String, Node>,
    // clé = "a_id:b_id:protocol" (canonique: a_id <= b_id)
    edges: HashMap<String, Edge>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum GraphUpdate {
    #[serde(rename = "NodeAdded")]
    NewNode(Node),
    #[serde(rename = "EdgeAdded")]
    NewEdge(Edge),
    #[serde(rename = "EdgeUpdated")]
    EdgeUpdated(Edge),
}

static NODE_COUNTER: AtomicU64 = AtomicU64::new(1);
static EDGE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Clone, Debug)]
pub struct Node {
    pub id: String,
    pub name: String,  // l’IP sous forme de string (ou MAC)
    pub color: String, // stockée en String côté struct (UI-friendly)
    pub mac: String,
    pub ip: String,
}

impl Node {
    pub fn new(name: String, mac: String, color: &'static str, ip: String) -> Self {
        let id = NODE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: id.to_string(),
            name,
            color: color.to_string(),
            mac,
            ip,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String, // Node.id (a_id, canonique)
    pub target: String, // Node.id (b_id, canonique)
    pub label: String,  // protocole (ex: "DNS", "TCP", "IPv6"...)
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub bidir: bool, // true si trafic observé dans les deux sens
}

impl Edge {
    pub fn new(source: String, target: String) -> Self {
        let id = EDGE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: id.to_string(),
            source,
            target,
            label: String::new(),
            source_port: None,
            destination_port: None,
            bidir: false,
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

impl GraphData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_packet_flow(&mut self, packet: &PacketFlowOwned) -> Vec<GraphUpdate> {
        use std::collections::hash_map::Entry;
        let mut updates = Vec::new();

        // ===============================
        // 1) Chemin L3 (avec IP) si possible
        // ===============================
        if let Some(internet) = packet.internet.as_ref()
            && let (Some(src_ip), Some(dst_ip)) = (internet.source_ip, internet.destination_ip)
        {
            let src_type = internet.ip_source_type.as_ref();
            let dst_type = internet.ip_destination_type.as_ref();
            if is_valid_ip(src_type) && is_valid_ip(dst_type) {
                let src_color = color_of(src_type);
                let dst_color = color_of(dst_type);

                let src_ip_str = src_ip.to_string();
                let dst_ip_str = dst_ip.to_string();

                // Nœud source
                let src_node_id = match self.nodes.entry(src_ip_str.clone()) {
                    Entry::Occupied(e) => e.get().id.clone(),
                    Entry::Vacant(v) => {
                        let node = Node::new(
                            src_ip_str.clone(),
                            packet.data_link.source_mac.clone(),
                            src_color,
                            src_ip_str.clone(),
                        );
                        let node_id = node.id.clone();
                        v.insert(node.clone());
                        updates.push(GraphUpdate::NewNode(node));
                        node_id
                    }
                };

                // Nœud destination
                let dst_node_id = match self.nodes.entry(dst_ip_str.clone()) {
                    Entry::Occupied(e) => e.get().id.clone(),
                    Entry::Vacant(v) => {
                        let node = Node::new(
                            dst_ip_str.clone(),
                            packet.data_link.destination_mac.clone(),
                            dst_color,
                            dst_ip_str.clone(),
                        );
                        let node_id = node.id.clone();
                        v.insert(node.clone());
                        updates.push(GraphUpdate::NewNode(node));
                        node_id
                    }
                };

                let protocol = best_protocol_label(packet);

                // 🔥 Clé non orientée + direction courante vs canonique
                let (edge_key, a_id, b_id, current_is_a_to_b) =
                    undirected_key(&src_node_id, &dst_node_id, &protocol);

                match self.edges.get_mut(&edge_key) {
                    Some(edge) => {
                        // Arête existe déjà (A—B:proto). Si on observe le sens inverse pour la
                        // première fois, on passe bidir=true et on notifie le front.
                        if !edge.bidir {
                            // À la création, edge.source == a_id et edge.target == b_id.
                            // Si current_is_a_to_b == false -> on a vu b->a -> bidir.
                            if !current_is_a_to_b {
                                edge.bidir = true;
                                updates.push(GraphUpdate::EdgeUpdated(edge.clone()));
                            }
                        }
                    }
                    None => {
                        // Première observation de {A,B,proto} → création de l'arête canonique (A->B)
                        let edge = Edge::new(a_id.clone(), b_id.clone())
                            .with_label(protocol)
                            .with_ports(
                                packet.transport.as_ref().and_then(|t| t.source_port),
                                packet.transport.as_ref().and_then(|t| t.destination_port),
                            );
                        self.edges.insert(edge_key, edge.clone());
                        updates.push(GraphUpdate::NewEdge(edge));
                    }
                }

                return updates; // L3 traité
            }
        }

        // ===============================
        // 2) Fallback L2 (MAC-only)
        // ===============================
        const L2_COLOR: &str = "#00BCD4";

        let src_mac = packet.data_link.source_mac.clone();
        let dst_mac = packet.data_link.destination_mac.clone();

        let src_key = format!("mac:{src_mac}");
        let dst_key = format!("mac:{dst_mac}");

        // Nœud source (MAC)
        let src_node_id = match self.nodes.entry(src_key.clone()) {
            Entry::Occupied(e) => e.get().id.clone(),
            Entry::Vacant(v) => {
                let node = Node::new(src_mac.clone(), src_mac.clone(), L2_COLOR, "".to_string());
                let node_id = node.id.clone();
                v.insert(node.clone());
                updates.push(GraphUpdate::NewNode(node));
                node_id
            }
        };

        // Nœud destination (MAC)
        let dst_node_id = match self.nodes.entry(dst_key.clone()) {
            Entry::Occupied(e) => e.get().id.clone(),
            Entry::Vacant(v) => {
                let node = Node::new(dst_mac.clone(), dst_mac.clone(), L2_COLOR, "".to_string());
                let node_id = node.id.clone();
                v.insert(node.clone());
                updates.push(GraphUpdate::NewNode(node));
                node_id
            }
        };

        let l2_proto = packet.data_link.ethertype.clone();
        let (edge_key, a_id, b_id, current_is_a_to_b) =
            undirected_key(&src_node_id, &dst_node_id, &l2_proto);

        match self.edges.get_mut(&edge_key) {
            Some(edge) => {
                if !edge.bidir && !current_is_a_to_b {
                    edge.bidir = true;
                    updates.push(GraphUpdate::EdgeUpdated(edge.clone()));
                }
            }
            None => {
                let edge = Edge::new(a_id.clone(), b_id.clone())
                    .with_label(l2_proto)
                    .with_ports(None, None); // pas de ports en L2
                self.edges.insert(edge_key, edge.clone());
                updates.push(GraphUpdate::NewEdge(edge));
            }
        }

        updates
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}

// ————— helpers —————

fn is_valid_ip(ip_type: Option<&IpType>) -> bool {
    // invalide si None ou Unknown
    !matches!(ip_type, None | Some(IpType::Unknown))
}

fn color_of(ip_type: Option<&IpType>) -> &'static str {
    match ip_type {
        Some(IpType::Private) => "#8BC34A",       // vert
        Some(IpType::Public) => "#2196F3",        // bleu
        Some(IpType::Multicast) => "#FFC107",     // jaune
        Some(IpType::Loopback) => "#E53935",      // rouge
        Some(IpType::Apipa) => "#FF9800",         // orange
        Some(IpType::LinkLocal) => "#FF5722",     // orange foncé
        Some(IpType::Ula) => "#9C27B0",           // violet
        Some(IpType::Documentation) => "#9E9E9E", // gris
        _ => "#9E9E9E",                           // défaut
    }
}

fn is_unknown(s: &str) -> bool {
    let t = s.trim();
    t.is_empty() || t.eq_ignore_ascii_case("unknown")
}

fn best_protocol_label(flow: &PacketFlowOwned) -> String {
    // L7 d'abord (uniquement si réellement détecté)
    if let Some(app) = &flow.application {
        let p = app.application_protocol.as_str();
        if !is_unknown(p) {
            return p.to_string();
        }
    }

    // Puis L4
    if let Some(t) = &flow.transport {
        let p = t.protocol.as_str();
        if !is_unknown(p) {
            return p.to_string();
        }
    }

    // Puis L3
    if let Some(i) = &flow.internet {
        let p = i.protocol.as_str();
        if !is_unknown(p) {
            return p.to_string();
        }
    }

    // Enfin L2
    let p = flow.data_link.ethertype.as_str();
    if !is_unknown(p) {
        return p.to_string();
    }

    "Unknown".to_string()
}

/// Retourne (edge_key, a_id, b_id, current_is_a_to_b)
/// a_id <= b_id (ordre canonique stable)
fn undirected_key(a: &str, b: &str, proto: &str) -> (String, String, String, bool) {
    if a <= b {
        (
            format!("{a}:{b}:{proto}"),
            a.to_string(),
            b.to_string(),
            true,
        )
    } else {
        (
            format!("{b}:{a}:{proto}"),
            b.to_string(),
            a.to_string(),
            false,
        )
    }
}
