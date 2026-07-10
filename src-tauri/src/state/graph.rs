//! Graphe réseau : nœuds (équipements identifiés par IP ou MAC) et arêtes
//! (conversations par protocole), construit incrémentalement pendant la
//! capture ou l'import et synchronisé avec le frontend par `GraphUpdate`.

use packet_parser::{IpType, owned::PacketFlowOwned};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Graphe complet : l'état de référence côté backend, dont le frontend
/// reçoit soit des updates incrémentales, soit un snapshot entier.
#[derive(Serialize, Default, Debug)]
pub struct GraphData {
    // clé = IP (stringifiée) ou "mac:XX:XX:..."
    pub nodes: HashMap<String, Node>,
    // clé = "a_id:b_id:protocol" (canonique: a_id <= b_id)
    pub edges: HashMap<String, Edge>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum GraphUpdate {
    #[serde(rename = "NodeAdded")]
    NewNode(Node),
    #[serde(rename = "NodeUpdated")]
    NodeUpdated(Node),
    #[serde(rename = "EdgeAdded")]
    NewEdge(Edge),
    #[serde(rename = "EdgeUpdated")]
    EdgeUpdated(Edge),
}

static NODE_COUNTER: AtomicU64 = AtomicU64::new(1);
static EDGE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Accumulateur d'updates graphe avec coalescence par entité : au sein d'une
/// fenêtre de batch, une seule entrée par nœud/arête (la dernière valeur gagne,
/// le variant `New*` d'origine est conservé). Les payloads étant des snapshots
/// complets, aucune information n'est perdue.
#[derive(Default)]
pub struct GraphUpdateBatch {
    updates: Vec<GraphUpdate>,
    node_index: HashMap<String, usize>,
    edge_index: HashMap<String, usize>,
}

impl GraphUpdateBatch {
    pub fn push(&mut self, update: GraphUpdate) {
        match update {
            GraphUpdate::NewNode(node) => self.upsert_node(node, true),
            GraphUpdate::NodeUpdated(node) => self.upsert_node(node, false),
            GraphUpdate::NewEdge(edge) => self.upsert_edge(edge, true),
            GraphUpdate::EdgeUpdated(edge) => self.upsert_edge(edge, false),
        }
    }

    fn upsert_node(&mut self, node: Node, is_new: bool) {
        match self.node_index.get(&node.id) {
            Some(&index) => {
                if let GraphUpdate::NewNode(existing) | GraphUpdate::NodeUpdated(existing) =
                    &mut self.updates[index]
                {
                    *existing = node;
                }
            }
            None => {
                self.node_index.insert(node.id.clone(), self.updates.len());
                self.updates.push(if is_new {
                    GraphUpdate::NewNode(node)
                } else {
                    GraphUpdate::NodeUpdated(node)
                });
            }
        }
    }

    fn upsert_edge(&mut self, edge: Edge, is_new: bool) {
        match self.edge_index.get(&edge.id) {
            Some(&index) => {
                if let GraphUpdate::NewEdge(existing) | GraphUpdate::EdgeUpdated(existing) =
                    &mut self.updates[index]
                {
                    *existing = edge;
                }
            }
            None => {
                self.edge_index.insert(edge.id.clone(), self.updates.len());
                self.updates.push(if is_new {
                    GraphUpdate::NewEdge(edge)
                } else {
                    GraphUpdate::EdgeUpdated(edge)
                });
            }
        }
    }

    /// Vide le batch et retourne les updates coalescées, dans l'ordre d'arrivée.
    pub fn take(&mut self) -> Vec<GraphUpdate> {
        self.node_index.clear();
        self.edge_index.clear();
        std::mem::take(&mut self.updates)
    }

    pub fn len(&self) -> usize {
        self.updates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Node {
    pub id: String,
    pub name: String,  // l’IP sous forme de string (ou MAC)
    pub color: String, // stockée en String côté struct (UI-friendly)
    pub mac: String,
    pub ip: String,
    pub label: Option<String>,
}

impl Node {
    pub fn new(
        name: String,
        mac: String,
        color: &'static str,
        ip: String,
        label: Option<String>,
    ) -> Self {
        let id = NODE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: id.to_string(),
            name,
            color: color.to_string(),
            mac,
            ip,
            label,
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
    pub bidir: bool,      // true si trafic observé dans les deux sens
    pub count: u64,       // paquets cumulés sur ce flux
    pub total_bytes: u64, // octets cumulés sur ce flux
    // Tunnels auxquels ce flux participe (hash de paire en hex, cf. TUNNELS.md) :
    // l'arête externe CAPWAP porte l'id de son tunnel, les arêtes internes
    // celui des tunnels qui les transportent. Sert à surligner la parenté
    // père/fils dans le graphe. Trié, sans doublon.
    pub encap_ids: Vec<String>,
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
            count: 0,
            total_bytes: 0,
            encap_ids: Vec::new(),
        }
    }

    pub fn with_traffic(mut self, count: u64, total_bytes: u64) -> Self {
        self.count = count;
        self.total_bytes = total_bytes;
        self
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

    pub fn add_packet_flow(
        &mut self,
        packet: &PacketFlowOwned,
        source_label: Option<String>,
        destination_label: Option<String>,
        packets: u64,
        bytes: u64,
        encap_ids: &[u64],
    ) -> Vec<GraphUpdate> {
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
                    Entry::Occupied(mut e) => {
                        maybe_update_node_label(e.get_mut(), source_label.clone(), &mut updates);
                        e.get().id.clone()
                    }
                    Entry::Vacant(v) => {
                        let node = Node::new(
                            src_ip_str.clone(),
                            packet.data_link.source_mac.clone(),
                            src_color,
                            src_ip_str.clone(),
                            source_label.clone(),
                        );
                        let node_id = node.id.clone();
                        v.insert(node.clone());
                        updates.push(GraphUpdate::NewNode(node));
                        node_id
                    }
                };

                // Nœud destination
                let dst_node_id = match self.nodes.entry(dst_ip_str.clone()) {
                    Entry::Occupied(mut e) => {
                        maybe_update_node_label(
                            e.get_mut(),
                            destination_label.clone(),
                            &mut updates,
                        );
                        e.get().id.clone()
                    }
                    Entry::Vacant(v) => {
                        let node = Node::new(
                            dst_ip_str.clone(),
                            packet.data_link.destination_mac.clone(),
                            dst_color,
                            dst_ip_str.clone(),
                            destination_label.clone(),
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
                        // Arête existe déjà (A—B:proto).
                        let mut notify = false;

                        // Si on observe le sens inverse pour la première fois,
                        // on passe bidir=true et on notifie le front.
                        // À la création, edge.source == a_id et edge.target == b_id.
                        // Si current_is_a_to_b == false -> on a vu b->a -> bidir.
                        if !edge.bidir && !current_is_a_to_b {
                            edge.bidir = true;
                            notify = true;
                        }

                        if accumulate_traffic(edge, packets, bytes) {
                            notify = true;
                        }

                        if merge_encap_ids(edge, encap_ids) {
                            notify = true;
                        }

                        if notify {
                            updates.push(GraphUpdate::EdgeUpdated(edge.clone()));
                        }
                    }
                    None => {
                        // Première observation de {A,B,proto} → création de l'arête canonique (A->B)
                        let mut edge = Edge::new(a_id.clone(), b_id.clone())
                            .with_label(protocol)
                            .with_ports(
                                packet.transport.as_ref().and_then(|t| t.source_port),
                                packet.transport.as_ref().and_then(|t| t.destination_port),
                            )
                            .with_traffic(packets, bytes);
                        merge_encap_ids(&mut edge, encap_ids);
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
                let node = Node::new(
                    src_mac.clone(),
                    src_mac.clone(),
                    L2_COLOR,
                    "".to_string(),
                    None,
                );
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
                let node = Node::new(
                    dst_mac.clone(),
                    dst_mac.clone(),
                    L2_COLOR,
                    "".to_string(),
                    None,
                );
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
                let mut notify = false;
                if !edge.bidir && !current_is_a_to_b {
                    edge.bidir = true;
                    notify = true;
                }
                if accumulate_traffic(edge, packets, bytes) {
                    notify = true;
                }
                if merge_encap_ids(edge, encap_ids) {
                    notify = true;
                }
                if notify {
                    updates.push(GraphUpdate::EdgeUpdated(edge.clone()));
                }
            }
            None => {
                let mut edge = Edge::new(a_id.clone(), b_id.clone())
                    .with_label(l2_proto)
                    .with_ports(None, None) // pas de ports en L2
                    .with_traffic(packets, bytes);
                merge_encap_ids(&mut edge, encap_ids);
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

    pub fn get_all_graph_data(&self) -> GraphData {
        let nodes = self.nodes.clone();
        let edges = self.edges.clone();
        GraphData { nodes, edges }
    }

    pub fn update_node_label(&mut self, mac: &str, ip: &str, label: String) -> Option<GraphUpdate> {
        let normalized = if label.trim().is_empty() {
            None
        } else {
            Some(label)
        };

        for node in self.nodes.values_mut() {
            if node.mac == mac && node.ip == ip {
                if node.label != normalized {
                    node.label = normalized;
                    return Some(GraphUpdate::NodeUpdated(node.clone()));
                }
                return None;
            }
        }

        None
    }

    /// Réapplique les labels de tous les nœuds via `resolve(mac, ip)`
    /// (typiquement `FlowMatrix::get_label` et ses replis IP seule / MAC seule).
    /// Retourne un `NodeUpdated` par nœud effectivement modifié.
    pub fn refresh_labels<F>(&mut self, resolve: F) -> Vec<GraphUpdate>
    where
        F: Fn(&str, &str) -> Option<String>,
    {
        let mut updates = Vec::new();

        for node in self.nodes.values_mut() {
            let Some(label) = resolve(&node.mac, &node.ip) else {
                continue;
            };

            let normalized = if label.trim().is_empty() {
                None
            } else {
                Some(label)
            };

            if node.label != normalized {
                node.label = normalized;
                updates.push(GraphUpdate::NodeUpdated(node.clone()));
            }
        }

        updates
    }
}

// ————— helpers —————

/// Cumule paquets/octets sur une arête. Retourne true quand le volume change
/// d'ordre de grandeur (log2) : l'épaisseur au rendu étant logarithmique,
/// inutile de notifier le front à chaque paquet.
fn accumulate_traffic(edge: &mut Edge, packets: u64, bytes: u64) -> bool {
    let bucket_before = 64 - edge.total_bytes.leading_zeros();
    edge.count = edge.count.saturating_add(packets);
    edge.total_bytes = edge.total_bytes.saturating_add(bytes);
    (64 - edge.total_bytes.leading_zeros()) != bucket_before
}

/// Ajoute à l'arête les ids de tunnel (format hex, celui de la colonne
/// `encap_id` du CSV) qu'elle ne portait pas encore. Retourne true si la
/// liste a changé (le front doit être notifié).
fn merge_encap_ids(edge: &mut Edge, encap_ids: &[u64]) -> bool {
    let mut changed = false;
    for id in encap_ids {
        let hex = format!("{id:016x}");
        if let Err(pos) = edge.encap_ids.binary_search(&hex) {
            edge.encap_ids.insert(pos, hex);
            changed = true;
        }
    }
    changed
}

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

fn maybe_update_node_label(node: &mut Node, label: Option<String>, updates: &mut Vec<GraphUpdate>) {
    if label.is_some() && node.label != label {
        node.label = label;
        updates.push(GraphUpdate::NodeUpdated(node.clone()));
    }
}

fn is_unknown(s: &str) -> bool {
    let t = s.trim();
    t.is_empty() || t.eq_ignore_ascii_case("unknown")
}

fn best_protocol_label(flow: &PacketFlowOwned) -> String {
    // L7 d'abord (uniquement si réellement détecté)
    if let Some(app) = &flow.application {
        let p = app.protocol.as_str();
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

#[cfg(test)]
mod graph_update_batch_tests {
    use super::*;

    fn node(id: &str, label: Option<&str>) -> Node {
        Node {
            id: id.to_string(),
            name: "10.0.0.1".to_string(),
            color: "#8BC34A".to_string(),
            mac: "aa:bb:cc:dd:ee:ff".to_string(),
            ip: "10.0.0.1".to_string(),
            label: label.map(str::to_string),
        }
    }

    fn edge(id: &str, count: u64) -> Edge {
        Edge {
            id: id.to_string(),
            source: "1".to_string(),
            target: "2".to_string(),
            label: "TCP".to_string(),
            source_port: None,
            destination_port: None,
            bidir: false,
            count,
            total_bytes: count * 100,
            encap_ids: Vec::new(),
        }
    }

    #[test]
    fn coalesces_edge_updates_keeping_last_payload() {
        let mut batch = GraphUpdateBatch::default();
        batch.push(GraphUpdate::EdgeUpdated(edge("7", 1)));
        batch.push(GraphUpdate::EdgeUpdated(edge("7", 5)));

        let updates = batch.take();
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            GraphUpdate::EdgeUpdated(e) => assert_eq!(e.count, 5),
            other => panic!("attendu EdgeUpdated, obtenu {other:?}"),
        }
    }

    #[test]
    fn new_edge_variant_survives_later_updates() {
        let mut batch = GraphUpdateBatch::default();
        batch.push(GraphUpdate::NewEdge(edge("7", 1)));
        batch.push(GraphUpdate::EdgeUpdated(edge("7", 9)));

        let updates = batch.take();
        assert_eq!(updates.len(), 1);
        match &updates[0] {
            GraphUpdate::NewEdge(e) => assert_eq!(e.count, 9),
            other => panic!("attendu NewEdge, obtenu {other:?}"),
        }
    }

    #[test]
    fn distinct_entities_are_kept_in_arrival_order() {
        let mut batch = GraphUpdateBatch::default();
        batch.push(GraphUpdate::NewNode(node("1", None)));
        batch.push(GraphUpdate::NewNode(node("2", None)));
        batch.push(GraphUpdate::NewEdge(edge("1", 1)));
        batch.push(GraphUpdate::NodeUpdated(node("1", Some("serveur"))));

        let updates = batch.take();
        assert_eq!(
            updates.len(),
            3,
            "nœud 1 coalescé, nœud 2 et arête 1 gardés"
        );
        match &updates[0] {
            GraphUpdate::NewNode(n) => {
                assert_eq!(n.id, "1");
                assert_eq!(n.label.as_deref(), Some("serveur"));
            }
            other => panic!("attendu NewNode, obtenu {other:?}"),
        }
    }

    #[test]
    fn take_resets_the_batch() {
        let mut batch = GraphUpdateBatch::default();
        batch.push(GraphUpdate::NewNode(node("1", None)));
        assert_eq!(batch.len(), 1);

        let _ = batch.take();
        assert!(batch.is_empty());

        // Réutilisable après take : nouvel index, pas de collision
        batch.push(GraphUpdate::NodeUpdated(node("1", Some("après"))));
        assert_eq!(batch.take().len(), 1);
    }
}
