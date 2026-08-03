//! Graphe réseau : nœuds (équipements identifiés par IP ou MAC) et arêtes
//! (conversations par protocole), construit incrémentalement pendant la
//! capture ou l'import et synchronisé avec le frontend par `GraphUpdate`.

use packet_parser::{IpType, owned::PacketFlowOwned};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::link::LinkView;
use crate::matrix::FlowMatrix;
use crate::matrix::is_non_unicast_mac;

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
    #[serde(rename = "nodeAdded")]
    NewNode(Node),
    #[serde(rename = "nodeUpdated")]
    NodeUpdated(Node),
    #[serde(rename = "edgeAdded")]
    NewEdge(Edge),
    #[serde(rename = "edgeUpdated")]
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
    /// Première MAC observée pour ce nœud (clé de labellisation historique).
    pub mac: String,
    /// Toutes les MAC unicast observées pour cette IP, première comprise.
    /// Plusieurs entrées = anomalie à investiguer (IP partagée, VRRP,
    /// usurpation ARP…) : le front la signale visuellement.
    pub macs: Vec<String>,
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
        let macs = if mac.is_empty() || is_non_unicast_mac(&mac) {
            Vec::new()
        } else {
            vec![mac.clone()]
        };
        Self {
            id: id.to_string(),
            name,
            color: color.to_string(),
            mac,
            macs,
            ip,
            label,
        }
    }
}

/// Plafond de ports « service » retenus par arête : au-delà (scan de ports),
/// la liste n'apporte plus rien à la lecture du graphe et croîtrait sans
/// borne. La matrice de flux, elle, garde le détail exact.
const MAX_EDGE_PORTS: usize = 16;

/// Début de la plage de ports dynamiques/éphémères (IANA 49152–65535) : les
/// clients y prennent leurs ports source. Un « service » dans cette plage
/// est presque toujours un éphémère (P2P, RTP, réponse) — le lister rendrait
/// l'arête illisible sans rien dire du service.
const DYNAMIC_PORT_START: u16 = 49152;

/// Port « service » d'un paquet : le plus petit des deux ports présents (le
/// service écoute sur le port bas, le client répond depuis un éphémère
/// haut). En bidirectionnel, la réponse `443 → 50123` redonne bien 443 au
/// lieu d'accumuler l'éphémère du client sur l'arête.
fn service_port(source_port: Option<u16>, destination_port: Option<u16>) -> Option<u16> {
    match (source_port, destination_port) {
        (Some(s), Some(d)) => Some(s.min(d)),
        (Some(p), None) | (None, Some(p)) => Some(p),
        (None, None) => None,
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    pub source: String, // Node.id (sens du PREMIER paquet observé)
    pub target: String, // Node.id (sens du PREMIER paquet observé)
    pub label: String,  // protocole (ex: "DNS", "TCP", "IPv6"...)
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    // Ports « service » des paquets agrégés sur cette arête (triés, plafonnés
    // à MAX_EDGE_PORTS) : une arête {A,B,proto} porte souvent plusieurs
    // services distincts, que le seul premier couple
    // source_port/destination_port masquait (#154). Les ports de la plage
    // dynamique (>= 49152) n'y entrent pas : ils lèvent `has_dynamic_ports`.
    pub ports: BTreeSet<u16>,
    // Vrai si du trafic sur ports exclusivement dynamiques/éphémères a été
    // observé (P2P, RTP…) : rendu « … » côté front, détail dans la matrice.
    pub has_dynamic_ports: bool,
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
            ports: BTreeSet::new(),
            has_dynamic_ports: false,
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
        self.record_service_port(source_port, destination_port);
        self
    }

    /// Enregistre le port « service » d'un paquet agrégé sur cette arête
    /// (voir [`service_port`]). Retourne vrai si l'affichage doit être
    /// rafraîchi. Liste plafonnée à [`MAX_EDGE_PORTS`] ; un port de la plage
    /// dynamique n'est pas listé, il lève `has_dynamic_ports` une fois.
    fn record_service_port(
        &mut self,
        source_port: Option<u16>,
        destination_port: Option<u16>,
    ) -> bool {
        let Some(port) = service_port(source_port, destination_port) else {
            return false;
        };
        if port >= DYNAMIC_PORT_START {
            return !std::mem::replace(&mut self.has_dynamic_ports, true);
        }
        if self.ports.len() < MAX_EDGE_PORTS || self.ports.contains(&port) {
            self.ports.insert(port)
        } else {
            false
        }
    }
}

impl GraphData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruit le graphe relationnel d'une matrice batch. Les flux sont
    /// triés avant insertion afin que leur ordre reste stable malgré le
    /// `HashMap` de la matrice.
    pub fn from_flow_matrix(matrix: &FlowMatrix) -> Self {
        Self::from_flow_matrix_filtered(matrix, None)
    }

    /// Variante filtrée sur un protocole de n'importe quelle couche. Ainsi,
    /// `TCP` conserve aussi une relation affichée comme HTTP/TLS parce que le
    /// graphe privilégie normalement le protocole applicatif dans son label.
    pub fn from_flow_matrix_filtered(matrix: &FlowMatrix, protocol: Option<&str>) -> Self {
        let mut flows: Vec<_> = matrix.matrix.iter().collect();
        flows.sort_by_cached_key(|(flow, _)| format!("{flow:?}"));

        let mut graph = Self::new();
        for (flow, entries) in flows {
            if protocol.is_some_and(|protocol| !flow_uses_protocol(flow, protocol)) {
                continue;
            }
            let packets = entries.iter().map(|(_, stats)| stats.count).sum();
            let bytes = entries.iter().fold(0u64, |total, (_, stats)| {
                total.saturating_add(stats.total_bytes)
            });
            let mut encap_ids: Vec<u64> = entries.iter().filter_map(|(id, _)| *id).collect();
            encap_ids.sort_unstable();
            encap_ids.dedup();

            let link = LinkView::of(&flow.data_link);
            let source_ip = flow
                .internet
                .as_ref()
                .and_then(|internet| internet.source_ip)
                .map(|ip| ip.to_string())
                .unwrap_or_default();
            let destination_ip = flow
                .internet
                .as_ref()
                .and_then(|internet| internet.destination_ip)
                .map(|ip| ip.to_string())
                .unwrap_or_default();

            graph.add_packet_flow(
                flow,
                matrix.get_label(&link.source_mac, &source_ip),
                matrix.get_label(&link.destination_mac, &destination_ip),
                packets,
                bytes,
                &encap_ids,
            );
        }
        graph
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
        let link = LinkView::of(&packet.data_link);

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
                        record_observed_mac(e.get_mut(), &link.source_mac, &mut updates);
                        e.get().id.clone()
                    }
                    Entry::Vacant(v) => {
                        let node = Node::new(
                            src_ip_str.clone(),
                            link.source_mac.clone(),
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
                        record_observed_mac(e.get_mut(), &link.destination_mac, &mut updates);
                        e.get().id.clone()
                    }
                    Entry::Vacant(v) => {
                        let node = Node::new(
                            dst_ip_str.clone(),
                            link.destination_mac.clone(),
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

                // Clé non orientée : {A,B,proto} indépendamment du sens.
                let edge_key = undirected_edge_key(&src_node_id, &dst_node_id, &protocol);

                match self.edges.get_mut(&edge_key) {
                    Some(edge) => {
                        // Arête existe déjà (A—B:proto).
                        let mut notify = false;

                        // L'arête est stockée dans le sens du premier paquet
                        // observé : bidir passe à vrai seulement quand le sens
                        // courant en diffère réellement.
                        if !edge.bidir && edge.source != src_node_id {
                            edge.bidir = true;
                            notify = true;
                        }

                        if accumulate_traffic(edge, packets, bytes) {
                            notify = true;
                        }

                        if merge_encap_ids(edge, encap_ids) {
                            notify = true;
                        }

                        // Chaque service observé s'ajoute à l'arête : le
                        // premier couple de ports ne masque plus les autres
                        // services (#154), et les ports éphémères ne polluent
                        // pas la liste (port « service » + plage dynamique).
                        if edge.record_service_port(
                            packet.transport.as_ref().and_then(|t| t.source_port),
                            packet.transport.as_ref().and_then(|t| t.destination_port),
                        ) {
                            notify = true;
                        }

                        if notify {
                            updates.push(GraphUpdate::EdgeUpdated(edge.clone()));
                        }
                    }
                    None => {
                        // Première observation de {A,B,proto} : l'arête est
                        // créée dans le sens réellement observé (la flèche du
                        // rendu reflète le premier paquet).
                        let mut edge = Edge::new(src_node_id.clone(), dst_node_id.clone())
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

        let src_mac = link.source_mac.clone();
        let dst_mac = link.destination_mac.clone();

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

        let l2_proto = link.protocol.clone();
        let edge_key = undirected_edge_key(&src_node_id, &dst_node_id, &l2_proto);

        match self.edges.get_mut(&edge_key) {
            Some(edge) => {
                let mut notify = false;
                if !edge.bidir && edge.source != src_node_id {
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
                let mut edge = Edge::new(src_node_id.clone(), dst_node_id.clone())
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

/// Plafond de MAC retenues par nœud : l'anomalie multi-MAC est déjà signalée
/// dès la deuxième, en garder plus n'apporte rien à l'arbitrage et une
/// usurpation massive (MAC aléatoires) gonflerait la mémoire sans borne (#147).
const MAX_MACS_PER_NODE: usize = 16;

/// Enregistre une MAC unicast supplémentaire observée pour ce nœud (IP).
/// Plusieurs MAC pour une même IP = anomalie à investiguer (IP partagée,
/// VRRP, usurpation ARP…) : le front la signale visuellement. Les MAC
/// broadcast/multicast, partageables par nature, sont ignorées.
fn record_observed_mac(node: &mut Node, mac: &str, updates: &mut Vec<GraphUpdate>) {
    if mac.is_empty() || is_non_unicast_mac(mac) {
        return;
    }
    if node.macs.iter().any(|known| known == mac) {
        return;
    }
    if node.macs.len() >= MAX_MACS_PER_NODE {
        return;
    }
    node.macs.push(mac.to_string());
    updates.push(GraphUpdate::NodeUpdated(node.clone()));
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
    let p = LinkView::of(&flow.data_link).protocol;
    if !is_unknown(&p) {
        return p;
    }

    "Unknown".to_string()
}

fn flow_uses_protocol(flow: &PacketFlowOwned, expected: &str) -> bool {
    flow.application
        .as_ref()
        .is_some_and(|layer| layer.protocol.eq_ignore_ascii_case(expected))
        || flow
            .transport
            .as_ref()
            .is_some_and(|layer| layer.protocol.eq_ignore_ascii_case(expected))
        || flow
            .internet
            .as_ref()
            .is_some_and(|layer| layer.protocol.eq_ignore_ascii_case(expected))
        || LinkView::of(&flow.data_link)
            .protocol
            .eq_ignore_ascii_case(expected)
}

/// Clé d'arête non orientée `{A,B,proto}` : ordre canonique stable des ids,
/// indépendant du sens observé (les deux sens partagent la même arête).
fn undirected_edge_key(a: &str, b: &str, proto: &str) -> String {
    if a <= b {
        format!("{a}:{b}:{proto}")
    } else {
        format!("{b}:{a}:{proto}")
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
            macs: vec!["aa:bb:cc:dd:ee:ff".to_string()],
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
            ports: BTreeSet::new(),
            has_dynamic_ports: false,
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

#[cfg(test)]
mod add_packet_flow_tests {
    use super::*;
    use crate::link::ethernet_link_from_text;
    use packet_parser::owned::InternetOwned;
    use std::net::IpAddr;

    /// Flux L3 minimal : IPv4 privé des deux côtés (chemin L3 du graphe).
    fn flow(src_mac: &str, src_ip: &str, dst_mac: &str, dst_ip: &str) -> PacketFlowOwned {
        PacketFlowOwned {
            data_link: ethernet_link_from_text(src_mac, dst_mac, "IPv4", None),
            internet: Some(InternetOwned {
                source_ip: Some(src_ip.parse::<IpAddr>().unwrap()),
                ip_source_type: Some(IpType::from_ip(src_ip)),
                destination_ip: Some(dst_ip.parse::<IpAddr>().unwrap()),
                ip_destination_type: Some(IpType::from_ip(dst_ip)),
                protocol: "IPv4".to_string(),
            }),
            transport: None,
            application: None,
            corrupted: None,
            inner: None,
        }
    }

    fn add(graph: &mut GraphData, f: &PacketFlowOwned) -> Vec<GraphUpdate> {
        graph.add_packet_flow(f, None, None, 1, 100, &[])
    }

    fn flow_with_ports(
        src_mac: &str,
        src_ip: &str,
        dst_mac: &str,
        dst_ip: &str,
        source_port: u16,
        destination_port: u16,
    ) -> PacketFlowOwned {
        let mut f = flow(src_mac, src_ip, dst_mac, dst_ip);
        f.transport = Some(packet_parser::owned::TransportOwned {
            source_port: Some(source_port),
            destination_port: Some(destination_port),
            protocol: "TCP".to_string(),
        });
        f
    }

    /// Plusieurs services entre les mêmes équipements : l'arête accumule
    /// tous les ports « service » au lieu de figer le premier couple (#154).
    #[test]
    fn edge_accumulates_all_service_ports() {
        let mut graph = GraphData::new();
        let src = ("10:00:00:00:00:0a", "192.168.1.10");
        let dst = ("10:00:00:00:00:0b", "192.168.1.20");

        add(
            &mut graph,
            &flow_with_ports(src.0, src.1, dst.0, dst.1, 50_000, 80),
        );
        let updates = add(
            &mut graph,
            &flow_with_ports(src.0, src.1, dst.0, dst.1, 50_001, 443),
        );

        let edge = single_edge(&graph);
        assert_eq!(
            edge.ports.iter().copied().collect::<Vec<_>>(),
            vec![80, 443],
            "les deux services sont visibles"
        );
        assert_eq!(edge.destination_port, Some(80), "premier couple préservé");
        assert!(
            updates
                .iter()
                .any(|u| matches!(u, GraphUpdate::EdgeUpdated(e) if e.ports.len() == 2)),
            "le front est notifié du nouveau service"
        );

        // Même service revu : pas de notification pour rien.
        let updates = add(
            &mut graph,
            &flow_with_ports(src.0, src.1, dst.0, dst.1, 50_002, 443),
        );
        assert!(
            !updates
                .iter()
                .any(|u| matches!(u, GraphUpdate::EdgeUpdated(e) if e.ports.len() != 2)),
            "ports inchangés"
        );
    }

    /// En bidirectionnel, la réponse `443 → éphémère` ne doit pas accumuler
    /// le port éphémère du client : le port « service » (le plus petit des
    /// deux) est retenu dans les deux sens.
    #[test]
    fn reply_direction_records_service_port_not_ephemeral() {
        let mut graph = GraphData::new();
        let a = ("10:00:00:00:00:0a", "192.168.1.10");
        let b = ("10:00:00:00:00:0b", "192.168.1.20");

        // Requête client -> serveur puis réponse serveur -> client.
        add(
            &mut graph,
            &flow_with_ports(a.0, a.1, b.0, b.1, 50_123, 443),
        );
        add(
            &mut graph,
            &flow_with_ports(b.0, b.1, a.0, a.1, 443, 50_123),
        );

        let edge = single_edge(&graph);
        assert_eq!(
            edge.ports.iter().copied().collect::<Vec<_>>(),
            vec![443],
            "seul le port service est listé, pas l'éphémère du client"
        );
        assert!(!edge.has_dynamic_ports, "443 n'est pas un port dynamique");
        assert!(edge.bidir);
    }

    /// Deux ports éphémères (P2P, RTP…) : rien n'entre dans la liste, le
    /// marqueur has_dynamic_ports est levé une seule fois — l'arête reste
    /// lisible au lieu d'accumuler des ports dynamiques sans signification.
    #[test]
    fn purely_dynamic_ports_raise_flag_without_polluting_list() {
        let mut graph = GraphData::new();
        let a = ("10:00:00:00:00:0a", "192.168.1.10");
        let b = ("10:00:00:00:00:0b", "192.168.1.20");

        add(
            &mut graph,
            &flow_with_ports(a.0, a.1, b.0, b.1, 50_000, 60_000),
        );
        add(
            &mut graph,
            &flow_with_ports(a.0, a.1, b.0, b.1, 50_001, 60_001),
        );

        let edge = single_edge(&graph);
        assert!(edge.ports.is_empty(), "aucun port dynamique listé");
        assert!(edge.has_dynamic_ports, "le trafic éphémère est signalé");

        // Un vrai service apparaît ensuite sur la même arête : il est listé.
        add(
            &mut graph,
            &flow_with_ports(a.0, a.1, b.0, b.1, 50_002, 502),
        );
        let edge = single_edge(&graph);
        assert_eq!(edge.ports.iter().copied().collect::<Vec<_>>(), vec![502]);
    }

    fn single_edge(graph: &GraphData) -> &Edge {
        assert_eq!(graph.edges.len(), 1, "une seule arête attendue");
        graph.edges.values().next().unwrap()
    }

    /// Régression : deux paquets dans le MÊME sens ne doivent jamais marquer
    /// l'arête bidirectionnelle (l'ancienne clé canonique le faisait dès que
    /// l'ordre lexicographique des ids ne suivait pas le sens observé).
    #[test]
    fn same_direction_packets_never_mark_bidir() {
        let mut graph = GraphData::new();
        let a_to_b = flow(
            "10:00:00:00:00:0a",
            "192.168.1.10",
            "10:00:00:00:00:0b",
            "192.168.1.20",
        );

        add(&mut graph, &a_to_b);
        add(&mut graph, &a_to_b);
        add(&mut graph, &a_to_b);

        assert!(!single_edge(&graph).bidir, "un seul sens observé");
    }

    /// Le premier paquet fixe le sens affiché ; le sens inverse passe bidir.
    #[test]
    fn reverse_direction_marks_bidir_and_first_direction_is_kept() {
        let mut graph = GraphData::new();
        let a_to_b = flow(
            "10:00:00:00:00:0a",
            "192.168.1.10",
            "10:00:00:00:00:0b",
            "192.168.1.20",
        );
        let b_to_a = flow(
            "10:00:00:00:00:0b",
            "192.168.1.20",
            "10:00:00:00:00:0a",
            "192.168.1.10",
        );

        add(&mut graph, &a_to_b);
        assert!(!single_edge(&graph).bidir);
        let source_id = graph.nodes.get("192.168.1.10").unwrap().id.clone();
        assert_eq!(
            single_edge(&graph).source,
            source_id,
            "l'arête part du premier émetteur observé"
        );

        add(&mut graph, &b_to_a);
        assert!(single_edge(&graph).bidir, "sens inverse observé -> bidir");
    }

    /// Une IP vue avec plusieurs MAC unicast accumule les MAC observées et
    /// notifie le front (anomalie type usurpation ARP à signaler).
    #[test]
    fn multiple_macs_for_same_ip_are_recorded() {
        let mut graph = GraphData::new();
        add(
            &mut graph,
            &flow(
                "10:00:00:00:00:0a",
                "192.168.1.10",
                "10:00:00:00:00:0b",
                "192.168.1.20",
            ),
        );
        let updates = add(
            &mut graph,
            &flow(
                "de:ad:be:ef:00:01",
                "192.168.1.10",
                "10:00:00:00:00:0b",
                "192.168.1.20",
            ),
        );

        let node = graph.nodes.get("192.168.1.10").unwrap();
        assert_eq!(node.macs.len(), 2, "les deux MAC observées sont retenues");
        assert_eq!(
            node.mac, "10:00:00:00:00:0a",
            "la première MAC reste la clé"
        );
        assert!(
            updates
                .iter()
                .any(|u| matches!(u, GraphUpdate::NodeUpdated(n) if n.macs.len() == 2)),
            "le front est notifié de l'anomalie"
        );

        // Usurpation massive (MAC aléatoires) : la liste est bornée, la
        // mémoire ne croît plus au-delà du plafond (#147).
        for i in 0..100u32 {
            add(
                &mut graph,
                &flow(
                    &format!("02:00:00:00:{:02x}:{:02x}", i / 256, i % 256),
                    "192.168.1.10",
                    "10:00:00:00:00:0b",
                    "192.168.1.20",
                ),
            );
        }
        let node = graph.nodes.get("192.168.1.10").unwrap();
        assert_eq!(
            node.macs.len(),
            MAX_MACS_PER_NODE,
            "la liste des MAC est plafonnée"
        );
    }

    /// Les MAC broadcast/multicast, partageables par nature, ne comptent pas
    /// comme conflit.
    #[test]
    fn non_unicast_macs_are_not_recorded_as_conflict() {
        let mut graph = GraphData::new();
        add(
            &mut graph,
            &flow(
                "10:00:00:00:00:0a",
                "192.168.1.10",
                "10:00:00:00:00:0b",
                "192.168.1.20",
            ),
        );
        add(
            &mut graph,
            &flow(
                "ff:ff:ff:ff:ff:ff",
                "192.168.1.10",
                "10:00:00:00:00:0b",
                "192.168.1.20",
            ),
        );

        let node = graph.nodes.get("192.168.1.10").unwrap();
        assert_eq!(node.macs.len(), 1, "la MAC broadcast est ignorée");
    }
}
