//! Miroir TypeScript de [`super::CaptureEvent`] (#142).
//!
//! `CaptureEvent` reste l'unique forme réellement envoyée sur le Channel
//! Tauri (avec ses emprunts zéro-copie, pour rester zéro-copie à la
//! sérialisation). Ce module ne sert qu'au contrat IPC : chaque type
//! ci-dessous est une forme *possédée*, `ts_rs::TS`-dérivée, avec les mêmes
//! attributs `serde` que l'original — y compris pour les types qui
//! traversent `sonar-flows-core` et la crate vendorée `packet_parser`
//! (ni l'une ni l'autre ne portent `ts-rs`, et `packet_parser` ne se modifie
//! pas, cf. `never-edit-vendor-packet-parser`).
//!
//! ## Exhaustivité Rust → miroir
//!
//! [`to_contract`] convertit un `CaptureEvent` réel en `CaptureEventContract`
//! par un `match` sans bras `_` : ajouter une variante à `CaptureEvent` sans
//! mettre à jour ce module casse la compilation, pas seulement les tests.
//! C'est `to_contract` que les tests de fidélité JSON appellent — le miroir
//! n'est plus jamais construit indépendamment à la main dans un test.
//!
//! Limite assumée : certains types traversés (`LinkLayerOwnedKind`,
//! `NetworkProtocol`) sont `#[non_exhaustive]` côté `packet_parser`. Le
//! compilateur *impose* un bras `_` pour ces `match`-là (règle du langage
//! pour les enums non-exhaustifs hors de leur crate) ; ce bras panique au
//! lieu de dériver en silence si `packet_parser` ajoute un DLT ou un
//! protocole que ce module ne connaît pas encore.
//!
//! Exporté par `cargo test export_ipc_bindings` (voir `crate::errors::bindings`)
//! dans `src/types/generated/`.

use std::collections::HashMap;

use serde::Serialize;
use ts_rs::TS;

use crate::state::capture::capture_handle::messages::capture::CapturedPacketOwned;
use crate::state::capture::capture_handle::messages::stats::StatsPayload;
use crate::state::graph::{Edge, GraphData, GraphUpdate, Node};

// ---------------------------------------------------------------------
// Graphe (mirror de `sonar_flows_core::graph::{Node, Edge, GraphUpdate, GraphData}`)
// ---------------------------------------------------------------------

#[derive(Serialize, Clone, TS)]
#[ts(rename = "Node")]
pub struct NodeContract {
    pub id: String,
    pub name: String,
    pub color: String,
    pub mac: String,
    pub macs: Vec<String>,
    pub ip: String,
    /// VLAN 802.1Q du nœud (`null` hors trame taguée) : fait partie de la
    /// clé d'identité — la même IP sur deux VLAN est deux nœuds (#154).
    pub vlan_id: Option<u16>,
    /// Vrai quand la même IP est portée par plusieurs nœuds (VLAN
    /// différents) : anomalie à signaler visuellement (#154).
    pub duplicate_ip: bool,
    pub label: Option<String>,
}

impl From<&Node> for NodeContract {
    fn from(node: &Node) -> Self {
        Self {
            id: node.id.clone(),
            name: node.name.clone(),
            color: node.color.clone(),
            mac: node.mac.clone(),
            macs: node.macs.clone(),
            ip: node.ip.clone(),
            vlan_id: node.vlan_id,
            duplicate_ip: node.duplicate_ip,
            label: node.label.clone(),
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "Edge", rename_all = "camelCase")]
pub struct EdgeContract {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub ports: Vec<u16>,
    pub has_dynamic_ports: bool,
    pub bidir: bool,
    pub count: u64,
    pub total_bytes: u64,
    pub encap_ids: Vec<String>,
}

impl From<&Edge> for EdgeContract {
    fn from(edge: &Edge) -> Self {
        Self {
            id: edge.id.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            label: edge.label.clone(),
            source_port: edge.source_port,
            destination_port: edge.destination_port,
            ports: edge.ports.iter().copied().collect(),
            has_dynamic_ports: edge.has_dynamic_ports,
            bidir: edge.bidir,
            count: edge.count,
            total_bytes: edge.total_bytes,
            encap_ids: edge.encap_ids.clone(),
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[serde(tag = "type", content = "payload")]
#[ts(rename = "GraphUpdate")]
pub enum GraphUpdateContract {
    #[serde(rename = "nodeAdded")]
    NewNode(NodeContract),
    #[serde(rename = "nodeUpdated")]
    NodeUpdated(NodeContract),
    #[serde(rename = "edgeAdded")]
    NewEdge(EdgeContract),
    #[serde(rename = "edgeUpdated")]
    EdgeUpdated(EdgeContract),
}

impl From<&GraphUpdate> for GraphUpdateContract {
    fn from(update: &GraphUpdate) -> Self {
        match update {
            GraphUpdate::NewNode(node) => Self::NewNode(node.into()),
            GraphUpdate::NodeUpdated(node) => Self::NodeUpdated(node.into()),
            GraphUpdate::NewEdge(edge) => Self::NewEdge(edge.into()),
            GraphUpdate::EdgeUpdated(edge) => Self::EdgeUpdated(edge.into()),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(rename = "GraphData")]
pub struct GraphDataContract {
    pub nodes: HashMap<String, NodeContract>,
    pub edges: HashMap<String, EdgeContract>,
}

impl From<&GraphData> for GraphDataContract {
    fn from(graph: &GraphData) -> Self {
        Self {
            nodes: graph
                .nodes
                .iter()
                .map(|(k, v)| (k.clone(), v.into()))
                .collect(),
            edges: graph
                .edges
                .iter()
                .map(|(k, v)| (k.clone(), v.into()))
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------
// Paquet (mirror de `CapturedPacketOwned` et de la couche liaison
// multi-LINKTYPE de `packet_parser::owned`, cf. `link.rs`)
// ---------------------------------------------------------------------

#[derive(Serialize, Clone, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(rename = "NetworkProtocol")]
pub enum NetworkProtocolContract {
    Ipv4,
    Ipv6,
    Arp,
    Profinet,
    Other(u16),
}

impl From<packet_parser::NetworkProtocol> for NetworkProtocolContract {
    fn from(protocol: packet_parser::NetworkProtocol) -> Self {
        use packet_parser::NetworkProtocol;
        match protocol {
            NetworkProtocol::Ipv4 => Self::Ipv4,
            NetworkProtocol::Ipv6 => Self::Ipv6,
            NetworkProtocol::Arp => Self::Arp,
            NetworkProtocol::Profinet => Self::Profinet,
            NetworkProtocol::Other(value) => Self::Other(value),
            // `NetworkProtocol` est `#[non_exhaustive]` côté packet_parser : le
            // compilateur impose ce bras. Un panic loud vaut mieux qu'un match
            // silencieusement incomplet si un protocole est ajouté en amont.
            _ => panic!(
                "NetworkProtocol non couvert par crate::events::contract : \
                 mettre à jour NetworkProtocolContract"
            ),
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "VlanTag")]
pub struct VlanTagContract {
    pub id: u16,
    pub pcp: u8,
    pub dei: bool,
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "DataLinkDetails")]
pub struct DataLinkDetailsContract {
    pub destination_mac: String,
    pub source_mac: String,
    pub ethertype: String,
    // Omis (pas `null`) quand aucun VLAN : #[ts(optional)] déclare la clé
    // TypeScript optionnelle plutôt que faussement obligatoire.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub vlan: Option<VlanTagContract>,
}

impl From<&packet_parser::owned::DataLinkOwned> for DataLinkDetailsContract {
    fn from(frame: &packet_parser::owned::DataLinkOwned) -> Self {
        Self {
            destination_mac: frame.destination_mac.to_string(),
            source_mac: frame.source_mac.to_string(),
            ethertype: frame.ethertype.name(),
            vlan: frame.vlan.as_ref().map(|v| VlanTagContract {
                id: v.id,
                pcp: v.pcp,
                dei: v.dei,
            }),
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "RawIpLinkDetails")]
pub struct RawIpLinkDetailsContract {
    pub ip_version: u8,
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "LinuxSllLinkDetails")]
pub struct LinuxSllLinkDetailsContract {
    pub packet_type: u16,
    pub hardware_type: u16,
    pub address_length: u16,
    pub source_address: Option<Vec<u8>>,
    pub protocol: u16,
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "LinuxSll2LinkDetails")]
pub struct LinuxSll2LinkDetailsContract {
    pub protocol: u16,
    pub reserved_mbz: u16,
    pub interface_index: u32,
    pub hardware_type: u16,
    pub packet_type: u16,
    pub address_length: u8,
    pub source_address: Option<Vec<u8>>,
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "Ieee80211LinkDetails")]
pub struct Ieee80211LinkDetailsContract {
    pub destination_mac: String,
    pub source_mac: String,
    pub snap_protocol: String,
}

#[derive(Serialize, Clone, TS)]
#[serde(tag = "link_kind", content = "link_details", rename_all = "snake_case")]
#[ts(rename = "LinkLayerKind")]
pub enum LinkLayerKindContract {
    Ethernet(DataLinkDetailsContract),
    RawIp(RawIpLinkDetailsContract),
    LinuxSll(LinuxSllLinkDetailsContract),
    LinuxSll2(LinuxSll2LinkDetailsContract),
    Ieee80211(Ieee80211LinkDetailsContract),
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "DataLink")]
pub struct DataLinkContract {
    pub link_type: u32,
    pub network_protocol: NetworkProtocolContract,
    #[serde(flatten)]
    pub kind: LinkLayerKindContract,
}

impl From<&packet_parser::owned::LinkLayerOwned> for DataLinkContract {
    fn from(link: &packet_parser::owned::LinkLayerOwned) -> Self {
        use packet_parser::owned::LinkLayerOwnedKind;

        let kind = match link.kind() {
            LinkLayerOwnedKind::Ethernet(frame) => LinkLayerKindContract::Ethernet(frame.into()),
            LinkLayerOwnedKind::RawIp(details) => {
                LinkLayerKindContract::RawIp(RawIpLinkDetailsContract {
                    ip_version: details.ip_version,
                })
            }
            LinkLayerOwnedKind::LinuxSll(details) => {
                LinkLayerKindContract::LinuxSll(LinuxSllLinkDetailsContract {
                    packet_type: details.packet_type.0,
                    hardware_type: details.hardware_type.0,
                    address_length: details.address_length,
                    source_address: details.source_address.clone(),
                    protocol: details.protocol,
                })
            }
            LinkLayerOwnedKind::LinuxSll2(details) => {
                LinkLayerKindContract::LinuxSll2(LinuxSll2LinkDetailsContract {
                    protocol: details.protocol,
                    reserved_mbz: details.reserved_mbz,
                    interface_index: details.interface_index,
                    hardware_type: details.hardware_type.0,
                    packet_type: details.packet_type.0,
                    address_length: details.address_length,
                    source_address: details.source_address.clone(),
                })
            }
            LinkLayerOwnedKind::Ieee80211(frame) => {
                LinkLayerKindContract::Ieee80211(Ieee80211LinkDetailsContract {
                    destination_mac: frame.destination_mac.to_string(),
                    source_mac: frame.source_mac.to_string(),
                    snap_protocol: frame.snap_protocol.name(),
                })
            }
            // `LinkLayerOwnedKind` est `#[non_exhaustive]` : bras imposé par le
            // compilateur, cf. commentaire de tête de module.
            _ => panic!(
                "LinkLayerOwnedKind non couvert par crate::events::contract : \
                 mettre à jour DataLinkContract"
            ),
        };

        Self {
            link_type: link.link_type().0,
            network_protocol: link.network_protocol().into(),
            kind,
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "IpType")]
pub enum IpTypeContract {
    Private,
    Multicast,
    Loopback,
    Apipa,
    LinkLocal,
    Ula,
    Public,
    Documentation,
    Unknown,
}

impl From<&packet_parser::IpType> for IpTypeContract {
    fn from(ip_type: &packet_parser::IpType) -> Self {
        use packet_parser::IpType;
        match ip_type {
            IpType::Private => Self::Private,
            IpType::Multicast => Self::Multicast,
            IpType::Loopback => Self::Loopback,
            IpType::Apipa => Self::Apipa,
            IpType::LinkLocal => Self::LinkLocal,
            IpType::Ula => Self::Ula,
            IpType::Public => Self::Public,
            IpType::Documentation => Self::Documentation,
            IpType::Unknown => Self::Unknown,
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "CorruptedLayerKind")]
pub enum CorruptedLayerKindContract {
    Internet,
    Transport,
}

impl From<&packet_parser::CorruptedLayerKind> for CorruptedLayerKindContract {
    fn from(kind: &packet_parser::CorruptedLayerKind) -> Self {
        use packet_parser::CorruptedLayerKind;
        match kind {
            CorruptedLayerKind::Internet => Self::Internet,
            CorruptedLayerKind::Transport => Self::Transport,
        }
    }
}

#[derive(Serialize, Clone, TS)]
#[ts(rename = "CorruptedLayer")]
pub struct CorruptedLayerContract {
    pub layer: CorruptedLayerKindContract,
    pub error: String,
}

impl From<&packet_parser::CorruptedLayer> for CorruptedLayerContract {
    fn from(corrupted: &packet_parser::CorruptedLayer) -> Self {
        Self {
            layer: (&corrupted.layer).into(),
            error: corrupted.error.clone(),
        }
    }
}

/// `internet`/`transport`/`application` sont *aplatis* sur le fil réel
/// (`#[serde(flatten)] Option<T>` côté `PacketFlowOwned`) : chaque groupe
/// n'apparaît que si présent, sans clé intermédiaire `internet`/`transport`/
/// `application`. `ts-rs` ne sait pas dériver un flatten d'`Option` (panique
/// « cannot be flattened »), donc ce type porte les champs à plat pour le
/// contrat TS et fournit sa propre [`Serialize`] ci-dessous qui reproduit le
/// flatten exact — la présence de chaque groupe se lit sur son champ non
/// optionnel côté source (`protocol_internet`/`protocol_transport`/
/// `application_protocol`, chacun garanti présent si et seulement si son
/// groupe l'est, cf. `packet_parser::owned::{InternetOwned,TransportOwned,
/// ApplicationOwned}::protocol`).
///
/// Chaque champ peut être *absent* (groupe non présent) : `#[ts(optional =
/// nullable)]` déclare `champ?: T | null` (omis OU `null` OU `T`), qui reste
/// une sur-approximation sûre de l'invariant réel « tout le groupe ou rien »
/// — un consommateur TypeScript ne peut pas supposer `protocol_internet`
/// présent sans vérifier, ce qui est correct dans tous les cas. `inner` et
/// `corrupted` sont soit absents, soit un objet complet (jamais `null`) :
/// `#[ts(optional)]` (sans nullable) leur suffit.
#[derive(Clone, TS)]
#[ts(rename = "PacketFlow")]
pub struct PacketFlowContract {
    pub data_link: DataLinkContract,
    #[ts(optional = nullable)]
    pub source_ip: Option<String>,
    #[ts(optional = nullable)]
    pub ip_source_type: Option<IpTypeContract>,
    #[ts(optional = nullable)]
    pub destination_ip: Option<String>,
    #[ts(optional = nullable)]
    pub ip_destination_type: Option<IpTypeContract>,
    #[ts(optional)]
    pub protocol_internet: Option<String>,
    #[ts(optional = nullable)]
    pub source_port: Option<u16>,
    #[ts(optional = nullable)]
    pub destination_port: Option<u16>,
    #[ts(optional)]
    pub protocol_transport: Option<String>,
    #[ts(optional)]
    pub application_protocol: Option<String>,
    #[ts(optional)]
    pub inner: Option<Box<PacketFlowContract>>,
    #[ts(optional)]
    pub corrupted: Option<CorruptedLayerContract>,
}

impl Serialize for PacketFlowContract {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("data_link", &self.data_link)?;
        if let Some(protocol_internet) = &self.protocol_internet {
            map.serialize_entry("source_ip", &self.source_ip)?;
            map.serialize_entry("ip_source_type", &self.ip_source_type)?;
            map.serialize_entry("destination_ip", &self.destination_ip)?;
            map.serialize_entry("ip_destination_type", &self.ip_destination_type)?;
            map.serialize_entry("protocol_internet", protocol_internet)?;
        }
        if let Some(protocol_transport) = &self.protocol_transport {
            map.serialize_entry("source_port", &self.source_port)?;
            map.serialize_entry("destination_port", &self.destination_port)?;
            map.serialize_entry("protocol_transport", protocol_transport)?;
        }
        if let Some(application_protocol) = &self.application_protocol {
            map.serialize_entry("application_protocol", application_protocol)?;
        }
        if let Some(inner) = &self.inner {
            map.serialize_entry("inner", inner)?;
        }
        if let Some(corrupted) = &self.corrupted {
            map.serialize_entry("corrupted", corrupted)?;
        }
        map.end()
    }
}

impl From<&packet_parser::owned::PacketFlowOwned> for PacketFlowContract {
    fn from(flow: &packet_parser::owned::PacketFlowOwned) -> Self {
        Self {
            data_link: (&flow.data_link).into(),
            source_ip: flow
                .internet
                .as_ref()
                .and_then(|i| i.source_ip)
                .map(|ip| ip.to_string()),
            ip_source_type: flow
                .internet
                .as_ref()
                .and_then(|i| i.ip_source_type.as_ref().map(Into::into)),
            destination_ip: flow
                .internet
                .as_ref()
                .and_then(|i| i.destination_ip)
                .map(|ip| ip.to_string()),
            ip_destination_type: flow
                .internet
                .as_ref()
                .and_then(|i| i.ip_destination_type.as_ref().map(Into::into)),
            protocol_internet: flow.internet.as_ref().map(|i| i.protocol.clone()),
            source_port: flow.transport.as_ref().and_then(|t| t.source_port),
            destination_port: flow.transport.as_ref().and_then(|t| t.destination_port),
            protocol_transport: flow.transport.as_ref().map(|t| t.protocol.clone()),
            application_protocol: flow.application.as_ref().map(|a| a.protocol.clone()),
            inner: flow.inner.as_deref().map(|inner| Box::new(inner.into())),
            corrupted: flow.corrupted.as_ref().map(Into::into),
        }
    }
}

/// Paquet tel qu'émis dans `packetBatch` (#142). Miroir de
/// `CapturedPacketOwned` ; `ts_sec`/`ts_usec` (`tsSec`/`tsUsec` sur le fil,
/// `rename_all = "camelCase"`) sont `i64` ici même si la forme réelle varie
/// par OS (`i32` sous Windows/macOS pour `ts_usec`) — sans incidence sur la
/// forme JSON, un nombre reste un nombre. `encap_id` (`encapId` sur le fil)
/// est en revanche une vraie divergence de type à préserver : c'est un hash
/// FNV-1a 64 bits (`sonar_flows_core::packet::serialize_encap_id_hex`),
/// sérialisé en hex 16 caractères comme `Edge::encap_ids` (`encapIds` sur le
/// fil), PAS en nombre — un `u64` pleine plage dépasse
/// `Number.MAX_SAFE_INTEGER` et perdrait en précision une fois passé par un
/// `number` JSON.
#[derive(Serialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "PacketBatchPacket", rename_all = "camelCase")]
pub struct PacketBatchPacketContract {
    pub ts_sec: i64,
    pub ts_usec: i64,
    pub caplen: u32,
    pub len: u32,
    pub flow: PacketFlowContract,
    pub encap_id: Option<String>,
}

impl From<&CapturedPacketOwned> for PacketBatchPacketContract {
    // `ts_sec`/`ts_usec` sont `i64` sous Linux (donc un cast identité, que
    // clippy signale ici) mais `i32` sous Windows/macOS : le cast reste
    // nécessaire pour rester portable, cf. la largeur par OS de
    // `CapturedPacketOwned` dans `sonar_flows_core::packet`.
    #[allow(clippy::unnecessary_cast)]
    fn from(packet: &CapturedPacketOwned) -> Self {
        Self {
            ts_sec: packet.ts_sec as i64,
            ts_usec: packet.ts_usec as i64,
            caplen: packet.caplen,
            len: packet.len,
            flow: (&packet.flow).into(),
            encap_id: packet.encap_id.map(|id| format!("{id:016x}")),
        }
    }
}

// ---------------------------------------------------------------------
// Union complète des événements (mirror de `super::CaptureEvent`)
// ---------------------------------------------------------------------

/// Contrat exhaustif généré pour `src/types/generated/CaptureEvent.ts`,
/// consommé tel quel par `src/types/capture.ts`. `camelCase` uniforme (tags
/// de variante ET champs de struct, `rename_all`/`rename_all_fields`, #142) —
/// seule exception assumée : les champs qui traversent la couche paquet/flux
/// de la crate vendorée `packet_parser` (`PacketFlowContract` et ce qu'elle
/// contient) restent dans la casse qu'elle impose, cf. le commentaire de tête
/// de module.
///
/// Pas de variante `Packet` : `CaptureEvent::Packet` n'est construite nulle
/// part (`#[allow(dead_code)]` côté Rust) — plutôt qu'un DTO jamais exercé,
/// le variant est retiré des deux côtés (voir aussi le nettoyage du store
/// frontend, #142).
#[derive(Serialize, TS)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
#[ts(rename = "CaptureEvent", rename_all_fields = "camelCase")]
pub enum CaptureEventContract {
    Started {
        session_id: u64,
        device: String,
        buffer_size: i32,
        chan_capacity: i32,
        timeout: i32,
        snaplen: i32,
        link_type: String,
        protocol_version: u32,
    },
    Stats(StatsPayload),
    ChannelCapacityPayload {
        session_id: u64,
        channel_size: u64,
        current_size: u64,
        backpressure: bool,
    },
    PacketBatch {
        session_id: u64,
        packets: Vec<PacketBatchPacketContract>,
    },
    Graph {
        update: GraphUpdateContract,
    },
    GraphBatch {
        session_id: u64,
        updates: Vec<GraphUpdateContract>,
    },
    Stopped {
        session_id: u64,
        reason: String,
    },
    Finished {
        file_name: String,
        packet_total_count: u64,
        integrated_count: u64,
        rejected_truncated_count: u64,
        rejected_unsupported_link_type_count: u64,
        rejected_malformed_count: u64,
        matrix_total_count: u64,
    },
    ImportProgress {
        file_name: String,
        file_index: u64,
        files_total: u64,
        current: u64,
        total: u64,
    },
    GraphSnapshot {
        graph_data: GraphDataContract,
    },
}

/// Conversion exhaustive `CaptureEvent → CaptureEventContract` (voir le
/// commentaire de tête de module). Utilisée par tous les tests de fidélité
/// JSON ci-dessous : le miroir n'est plus jamais construit indépendamment
/// dans un test, il est *dérivé* de la valeur réelle.
pub fn to_contract(event: &super::CaptureEvent<'_>) -> CaptureEventContract {
    match event {
        super::CaptureEvent::Started {
            session_id,
            device,
            buffer_size,
            chan_capacity,
            timeout,
            snaplen,
            link_type,
            protocol_version,
        } => CaptureEventContract::Started {
            session_id: *session_id,
            device: device.to_string(),
            buffer_size: *buffer_size,
            chan_capacity: *chan_capacity,
            timeout: *timeout,
            snaplen: *snaplen,
            link_type: link_type.to_string(),
            protocol_version: *protocol_version,
        },
        super::CaptureEvent::Stats {
            session_id,
            received,
            dropped,
            if_dropped,
            app_dropped,
            parse_errors,
            processed,
        } => CaptureEventContract::Stats(StatsPayload {
            session_id: *session_id,
            received: *received,
            dropped: *dropped,
            if_dropped: *if_dropped,
            app_dropped: *app_dropped,
            parse_errors: *parse_errors,
            processed: *processed,
        }),
        super::CaptureEvent::ChannelCapacityPayload {
            session_id,
            channel_size,
            current_size,
            backpressure,
        } => CaptureEventContract::ChannelCapacityPayload {
            session_id: *session_id,
            channel_size: *channel_size as u64,
            current_size: *current_size as u64,
            backpressure: *backpressure,
        },
        super::CaptureEvent::PacketBatch {
            session_id,
            packets,
        } => CaptureEventContract::PacketBatch {
            session_id: *session_id,
            packets: packets.iter().map(Into::into).collect(),
        },
        super::CaptureEvent::Graph { update } => CaptureEventContract::Graph {
            update: (*update).into(),
        },
        super::CaptureEvent::GraphBatch {
            session_id,
            updates,
        } => CaptureEventContract::GraphBatch {
            session_id: *session_id,
            updates: updates.iter().map(Into::into).collect(),
        },
        super::CaptureEvent::Stopped { session_id, reason } => CaptureEventContract::Stopped {
            session_id: *session_id,
            reason: reason.clone(),
        },
        super::CaptureEvent::Finished {
            file_name,
            packet_total_count,
            integrated_count,
            rejected_truncated_count,
            rejected_unsupported_link_type_count,
            rejected_malformed_count,
            matrix_total_count,
        } => CaptureEventContract::Finished {
            file_name: file_name.to_string(),
            packet_total_count: *packet_total_count as u64,
            integrated_count: *integrated_count as u64,
            rejected_truncated_count: *rejected_truncated_count as u64,
            rejected_unsupported_link_type_count: *rejected_unsupported_link_type_count as u64,
            rejected_malformed_count: *rejected_malformed_count as u64,
            matrix_total_count: *matrix_total_count as u64,
        },
        super::CaptureEvent::ImportProgress {
            file_name,
            file_index,
            files_total,
            current,
            total,
        } => CaptureEventContract::ImportProgress {
            file_name: file_name.to_string(),
            file_index: *file_index as u64,
            files_total: *files_total as u64,
            current: *current as u64,
            total: *total as u64,
        },
        super::CaptureEvent::GraphSnapshot { graph_data } => CaptureEventContract::GraphSnapshot {
            graph_data: (*graph_data).into(),
        },
    }
}

#[cfg(test)]
mod tests {
    //! Vérifie, variante par variante, que `to_contract` sérialise en JSON
    //! identique au vrai `CaptureEvent` — le test que réclame #142 ("Rust →
    //! JSON → TypeScript valide chaque variante et champ"). Le miroir n'est
    //! jamais construit à la main ici : il est *dérivé* de la valeur réelle
    //! par `to_contract`, la même fonction que celle dont l'exhaustivité du
    //! `match` est imposée par le compilateur.
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};

    use packet_parser::owned::{
        Ieee80211LinkOwned, LinkLayerOwned, LinuxSll2LinkOwned, LinuxSllLinkOwned,
    };
    use packet_parser::parse::data_link::ethertype::Ethertype;
    use packet_parser::parse::data_link::mac_addres::MacAddress;
    use packet_parser::parse::data_link::vlan_tag::VlanTag;
    use packet_parser::parse::link_layer::{LinuxArphrdType, LinuxCookedPacketType};
    use packet_parser::{CorruptedLayer, CorruptedLayerKind, IpType, NetworkProtocol};

    use super::*;
    use crate::state::graph::{Edge, GraphUpdate, Node};

    fn assert_same_json<A: Serialize>(real: &A, mirror: &CaptureEventContract, label: &str) {
        let real_json = serde_json::to_value(real).expect("sérialisation du type réel");
        let mirror_json = serde_json::to_value(mirror).expect("sérialisation du miroir");
        assert_eq!(real_json, mirror_json, "{label} : miroir divergent du réel");
    }

    #[test]
    fn started_matches() {
        let real = crate::events::CaptureEvent::Started {
            session_id: 7,
            device: "eth0",
            buffer_size: 65536,
            chan_capacity: 1024,
            timeout: 100,
            snaplen: 262144,
            link_type: "Ethernet",
            protocol_version: crate::events::CAPTURE_EVENT_PROTOCOL_VERSION,
        };
        assert_same_json(&real, &to_contract(&real), "started");
    }

    #[test]
    fn stats_matches() {
        let real = crate::events::CaptureEvent::Stats {
            session_id: 1,
            received: 10,
            dropped: 0,
            if_dropped: 0,
            app_dropped: 0,
            parse_errors: 1,
            processed: 3,
        };
        assert_same_json(&real, &to_contract(&real), "stats");
    }

    #[test]
    fn channel_capacity_matches() {
        let real = crate::events::CaptureEvent::ChannelCapacityPayload {
            session_id: 2,
            channel_size: 1024,
            current_size: 512,
            backpressure: true,
        };
        assert_same_json(&real, &to_contract(&real), "channelCapacityPayload");
    }

    #[test]
    fn stopped_matches() {
        let real = crate::events::CaptureEvent::Stopped {
            session_id: 3,
            reason: "erreur pcap".to_string(),
        };
        assert_same_json(&real, &to_contract(&real), "stopped");
    }

    #[test]
    fn finished_matches() {
        // Les quatre catégories non nulles à la fois : le test de fidélité
        // couvre chaque champ du bilan, pas seulement le chemin heureux.
        let real = crate::events::CaptureEvent::Finished {
            file_name: "capture.pcap",
            packet_total_count: 100,
            integrated_count: 95,
            rejected_truncated_count: 3,
            rejected_unsupported_link_type_count: 1,
            rejected_malformed_count: 1,
            matrix_total_count: 40,
        };
        assert_same_json(&real, &to_contract(&real), "finished");
    }

    #[test]
    fn import_progress_matches() {
        let real = crate::events::CaptureEvent::ImportProgress {
            file_name: "capture.pcap",
            file_index: 2,
            files_total: 5,
            current: 1_000,
            total: 48_350,
        };
        assert_same_json(&real, &to_contract(&real), "importProgress");
    }

    // Depuis sonar-flows-core 0.5, `Node::new`/`Edge::new` dérivent leur
    // `id` de la clé d'identité passée en premier argument (hash stable,
    // #154). On écrase quand même `id` par une valeur courte et lisible :
    // les fixtures JSON de `export_ipc_fixtures` restent auto-descriptives.
    fn sample_node() -> Node {
        let mut node = Node::new(
            "192.168.1.10",
            "192.168.1.10".to_string(),
            "aa:bb:cc:dd:ee:ff".to_string(),
            "#ff0000",
            "192.168.1.10".to_string(),
            None,
            Some("Serveur".to_string()),
        );
        node.id = "node-1".to_string();
        node
    }

    fn sample_edge() -> Edge {
        let mut edge = Edge::new(
            "node-1:node-2:TCP",
            "node-1".to_string(),
            "node-2".to_string(),
        );
        edge.id = "edge-1".to_string();
        edge.label = "TCP".to_string();
        edge.source_port = Some(443);
        edge.destination_port = Some(51820);
        edge.ports = std::collections::BTreeSet::from([80, 443]);
        edge.has_dynamic_ports = false;
        edge.bidir = true;
        edge.count = 12;
        edge.total_bytes = 4096;
        edge.encap_ids = vec!["deadbeef".to_string()];
        edge
    }

    /// Nœud sans label utilisateur : `sample_node()` en fixe toujours un,
    /// laissant `label: string | null` sans fixture TS exerçant le cas
    /// `null` (#142) — retirer le `| null` du type généré resterait alors
    /// invisible au typecheck.
    /// Ce nœud exerce aussi les cas `vlan_id` non nul et `duplicate_ip`
    /// vrai (#154) : `sample_node()` les laisse à `null`/`false`, une
    /// régression du type généré resterait sinon invisible au typecheck.
    fn sample_node_without_label() -> Node {
        let mut node = Node::new(
            "vlan42:192.168.1.20",
            "192.168.1.20".to_string(),
            "aa:bb:cc:dd:ee:00".to_string(),
            "#00ff00",
            "192.168.1.20".to_string(),
            Some(42),
            None,
        );
        node.id = "node-2".to_string();
        node.duplicate_ip = true;
        node
    }

    /// Arête sans port service observé : `sample_edge()` en fixe toujours un
    /// couple, laissant `sourcePort`/`destinationPort: number | null` sans
    /// fixture TS exerçant le cas `null` (#142).
    fn sample_edge_without_ports() -> Edge {
        let mut edge = Edge::new(
            "node-1:node-2:IPv6",
            "node-1".to_string(),
            "node-2".to_string(),
        );
        edge.id = "edge-2".to_string();
        edge.label = "IPv6".to_string();
        edge.bidir = false;
        edge.count = 3;
        edge.total_bytes = 512;
        edge
    }

    #[test]
    fn graph_update_variants_match() {
        let node = sample_node();
        let edge = sample_edge();
        let node_without_label = sample_node_without_label();
        let edge_without_ports = sample_edge_without_ports();
        for update in [
            GraphUpdate::NewNode(node.clone()),
            GraphUpdate::NodeUpdated(node.clone()),
            GraphUpdate::NewEdge(edge.clone()),
            GraphUpdate::EdgeUpdated(edge.clone()),
            GraphUpdate::NewNode(node_without_label.clone()),
            GraphUpdate::NewEdge(edge_without_ports.clone()),
        ] {
            let real_json = serde_json::to_value(&update).unwrap();
            let mirror: GraphUpdateContract = (&update).into();
            let mirror_json = serde_json::to_value(&mirror).unwrap();
            assert_eq!(
                real_json, mirror_json,
                "GraphUpdate {update:?} : miroir divergent"
            );
        }
    }

    #[test]
    fn graph_event_matches() {
        let update = GraphUpdate::NewNode(sample_node());
        let real = crate::events::CaptureEvent::Graph { update: &update };
        assert_same_json(&real, &to_contract(&real), "graph");
    }

    #[test]
    fn graph_batch_matches() {
        let updates = vec![
            GraphUpdate::NewNode(sample_node()),
            GraphUpdate::NewEdge(sample_edge()),
        ];
        let real = crate::events::CaptureEvent::GraphBatch {
            session_id: 5,
            updates,
        };
        assert_same_json(&real, &to_contract(&real), "graphBatch");
    }

    #[test]
    fn graph_snapshot_matches() {
        let node = sample_node();
        let edge = sample_edge();
        let mut nodes = HashMap::new();
        nodes.insert(node.id.clone(), node);
        let mut edges = HashMap::new();
        edges.insert(edge.id.clone(), edge);

        // `GraphData` n'est plus constructible par littéral (index interne
        // privé depuis sonar-flows-core 0.5) : on remplit ses champs publics.
        let mut real_graph = crate::state::graph::GraphData::new();
        real_graph.nodes = nodes;
        real_graph.edges = edges;
        let real = crate::events::CaptureEvent::GraphSnapshot {
            graph_data: &real_graph,
        };
        assert_same_json(&real, &to_contract(&real), "graphSnapshot");
    }

    fn ethernet_link(vlan: Option<VlanTag>) -> LinkLayerOwned {
        LinkLayerOwned::ethernet(packet_parser::owned::DataLinkOwned {
            destination_mac: MacAddress([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]),
            source_mac: MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            ethertype: Ethertype(0x0800),
            vlan,
        })
    }

    /// Comme [`ethernet_link`] mais avec un ethertype choisi : sert à exercer
    /// les variantes de `NetworkProtocol` autres qu'`Ipv4` (dérivées de
    /// l'ethertype par `LinkLayerOwned::ethernet`, cf. `packet_parser`), non
    /// couvertes par le reste des fixtures `packetBatch` (#142).
    fn ethernet_link_with_ethertype(ethertype: u16) -> LinkLayerOwned {
        LinkLayerOwned::ethernet(packet_parser::owned::DataLinkOwned {
            destination_mac: MacAddress([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]),
            source_mac: MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            ethertype: Ethertype(ethertype),
            vlan: None,
        })
    }

    /// Couche liaison + niveaux internet/transport/application au choix — `None`
    /// exerce le cas « clé absente », `Some` le cas « présent, éventuellement
    /// individuellement `null` ».
    fn flow_with(
        data_link: LinkLayerOwned,
        internet: Option<packet_parser::owned::InternetOwned>,
        transport: Option<packet_parser::owned::TransportOwned>,
        application: Option<packet_parser::owned::ApplicationOwned>,
        inner: Option<Box<packet_parser::owned::PacketFlowOwned>>,
        corrupted: Option<CorruptedLayer>,
    ) -> packet_parser::owned::PacketFlowOwned {
        packet_parser::owned::PacketFlowOwned {
            data_link,
            internet,
            transport,
            application,
            inner,
            corrupted,
        }
    }

    fn full_internet(
        source_ip: Ipv4Addr,
        destination_ip: Ipv4Addr,
    ) -> packet_parser::owned::InternetOwned {
        packet_parser::owned::InternetOwned {
            source_ip: Some(IpAddr::V4(source_ip)),
            ip_source_type: Some(IpType::Private),
            destination_ip: Some(IpAddr::V4(destination_ip)),
            ip_destination_type: Some(IpType::Public),
            protocol: "TCP".to_string(),
        }
    }

    /// Comme [`full_internet`] mais avec les `IpType` source/destination
    /// choisis : sert à exercer les variantes autres que `Private`/`Public`
    /// (#142).
    fn internet_with(
        source_ip: Ipv4Addr,
        ip_source_type: IpType,
        destination_ip: Ipv4Addr,
        ip_destination_type: IpType,
    ) -> packet_parser::owned::InternetOwned {
        packet_parser::owned::InternetOwned {
            source_ip: Some(IpAddr::V4(source_ip)),
            ip_source_type: Some(ip_source_type),
            destination_ip: Some(IpAddr::V4(destination_ip)),
            ip_destination_type: Some(ip_destination_type),
            protocol: "TCP".to_string(),
        }
    }

    fn full_transport() -> packet_parser::owned::TransportOwned {
        packet_parser::owned::TransportOwned {
            source_port: Some(51820),
            destination_port: Some(443),
            protocol: "TCP".to_string(),
        }
    }

    /// Groupe internet *présent* (protocole connu) mais adresses/`IpType`
    /// individuellement absents : `full_internet`/`internet_with` mettent
    /// toujours `Some` sur ces champs, laissant `source_ip`/`ip_source_type`/
    /// `destination_ip`/`ip_destination_type` sans fixture TS exerçant le
    /// cas `null` alors que le groupe lui-même est présent (#142) — un
    /// paquet valide sans IP résolue sur ce niveau (ex. ARP) doit rester
    /// distinct du cas « groupe absent » (clé complètement omise).
    fn internet_without_addresses() -> packet_parser::owned::InternetOwned {
        packet_parser::owned::InternetOwned {
            source_ip: None,
            ip_source_type: None,
            destination_ip: None,
            ip_destination_type: None,
            protocol: "ARP".to_string(),
        }
    }

    /// Retourne la couleur réellement attribuée par `sonar-flows-core` à un
    /// nœud du graphe. Le paquet passe volontairement par `GraphData` : la
    /// palette exportée vers TypeScript ci-dessous ne recopie donc aucune
    /// constante Rust et dérive du même chemin que le rendu en production.
    fn rendered_graph_node_color(ip_type: Option<IpType>) -> String {
        let internet = ip_type.map(|kind| {
            internet_with(
                Ipv4Addr::new(192, 0, 2, 1),
                kind.clone(),
                Ipv4Addr::new(192, 0, 2, 2),
                kind,
            )
        });
        let packet = flow_with(ethernet_link(None), internet, None, None, None, None);
        let mut graph = crate::state::graph::GraphData::default();
        graph.add_packet_flow(&packet, None, None, 1, 60, &[]);

        let colors: std::collections::BTreeSet<_> = graph
            .nodes
            .values()
            .map(|node| node.color.clone())
            .collect();
        assert_eq!(
            colors.len(),
            1,
            "une catégorie doit produire une couleur unique"
        );
        colors.into_iter().next().expect("au moins un nœud produit")
    }

    /// Groupe transport *présent* mais ports individuellement absents (même
    /// principe qu'[`internet_without_addresses`]) : `full_transport` met
    /// toujours `Some` sur les deux ports.
    fn transport_without_ports() -> packet_parser::owned::TransportOwned {
        packet_parser::owned::TransportOwned {
            source_port: None,
            destination_port: None,
            protocol: "ICMP".to_string(),
        }
    }

    fn full_application() -> packet_parser::owned::ApplicationOwned {
        packet_parser::owned::ApplicationOwned {
            protocol: "HTTPS".to_string(),
        }
    }

    fn packet_batch_of(
        flow: packet_parser::owned::PacketFlowOwned,
        encap_id: Option<u64>,
    ) -> crate::events::CaptureEvent<'static> {
        let packet = CapturedPacketOwned {
            ts_sec: 0,
            ts_usec: 0,
            caplen: 60,
            len: 60,
            flow,
            encap_id,
        };
        crate::events::CaptureEvent::PacketBatch {
            session_id: 1,
            packets: vec![packet],
        }
    }

    #[test]
    fn packet_batch_ethernet_with_vlan_matches() {
        let flow = flow_with(
            ethernet_link(Some(VlanTag {
                id: 42,
                pcp: 0,
                dei: false,
                inner_ethertype: Ethertype(0x0800),
            })),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            Some(full_transport()),
            Some(full_application()),
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(&real, &to_contract(&real), "packetBatch (Ethernet + VLAN)");
    }

    /// Sans VLAN et sans couche applicative : exerce le cas « clé absente »
    /// que `#[ts(optional)]`/`#[ts(optional = nullable)]` déclarent (#142) —
    /// un paquet valide sans ces couches doit rester identique au miroir.
    #[test]
    fn packet_batch_ethernet_without_vlan_or_application_matches() {
        let flow = flow_with(
            ethernet_link(None),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            Some(full_transport()),
            None,
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(
            &real,
            &to_contract(&real),
            "packetBatch (sans VLAN ni application)",
        );
    }

    /// Aucune couche internet/transport/application (ex. trame non-IP) :
    /// toutes les clés aplaties doivent être absentes, pas `null`.
    #[test]
    fn packet_batch_data_link_only_matches() {
        let flow = flow_with(ethernet_link(None), None, None, None, None, None);
        let real = packet_batch_of(flow, None);
        assert_same_json(&real, &to_contract(&real), "packetBatch (data_link seul)");
    }

    /// Groupes internet/transport *présents* (protocole connu) mais chaque
    /// champ individuel absent (`source_ip`/`ip_source_type`/…, ports) :
    /// distinct du cas « groupe absent » ci-dessus, ce que `full_internet`/
    /// `full_transport` (toujours `Some`) n'exercent jamais (#142).
    #[test]
    fn packet_batch_internet_and_transport_present_without_addresses_matches() {
        let flow = flow_with(
            ethernet_link(None),
            Some(internet_without_addresses()),
            Some(transport_without_ports()),
            None,
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(
            &real,
            &to_contract(&real),
            "packetBatch (groupes présents, champs individuels absents)",
        );
    }

    #[test]
    fn packet_batch_tunneled_matches() {
        let inner_flow = flow_with(
            ethernet_link(None),
            Some(full_internet(
                Ipv4Addr::new(10, 0, 0, 1),
                Ipv4Addr::new(10, 0, 0, 2),
            )),
            None,
            None,
            None,
            None,
        );
        let outer_flow = flow_with(
            ethernet_link(None),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            None,
            None,
            Some(Box::new(inner_flow)),
            None,
        );
        let real = packet_batch_of(outer_flow, Some(0xdead_beef));
        assert_same_json(&real, &to_contract(&real), "packetBatch (tunnelé)");

        // `encapId` doit être une chaîne hex, jamais un nombre JSON brut :
        // c'est un hash 64 bits qui peut dépasser
        // `Number.MAX_SAFE_INTEGER` (#142).
        let real_json = serde_json::to_value(&real).unwrap();
        let encap_id = &real_json["data"]["packets"][0]["encapId"];
        assert_eq!(
            encap_id.as_str(),
            Some("00000000deadbeef"),
            "encapId doit être sérialisé en hex, pas en nombre : {encap_id:?}"
        );
    }

    #[test]
    fn packet_batch_corrupted_matches() {
        let flow = flow_with(
            ethernet_link(None),
            None,
            None,
            None,
            None,
            Some(CorruptedLayer {
                layer: CorruptedLayerKind::Internet,
                error: "en-tête IPv4 tronqué".to_string(),
            }),
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(&real, &to_contract(&real), "packetBatch (couche corrompue)");
    }

    #[test]
    fn packet_batch_sll_matches() {
        let details = LinuxSllLinkOwned::new(
            LinuxCookedPacketType::OUTGOING,
            LinuxArphrdType::ETHERNET,
            6,
            Some(vec![0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4]),
            0x0800,
        );
        let flow = flow_with(
            LinkLayerOwned::linux_sll(details),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            Some(full_transport()),
            Some(full_application()),
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(&real, &to_contract(&real), "packetBatch (Linux SLL)");
    }

    /// SLL sans adresse source (`source_address: None`, ex. paquet sortant
    /// dont l'adresse n'a pas de sens) : `packet_batch_sll_matches` fixe
    /// toujours une adresse, laissant `source_address: number[] | null` sans
    /// fixture TS exerçant le cas `null` (#142).
    #[test]
    fn packet_batch_sll_without_address_matches() {
        let details = LinuxSllLinkOwned::new(
            LinuxCookedPacketType::OUTGOING,
            LinuxArphrdType::ETHERNET,
            0,
            None,
            0x0800,
        );
        let flow = flow_with(
            LinkLayerOwned::linux_sll(details),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            Some(full_transport()),
            Some(full_application()),
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(
            &real,
            &to_contract(&real),
            "packetBatch (Linux SLL sans adresse)",
        );
    }

    /// SLL2 sans adresse source : même trou de couverture que
    /// [`packet_batch_sll_without_address_matches`], mais pour SLL2 — aucun
    /// fixture `packetBatch` SLL2 n'exerçait `source_address: None` avant
    /// (#142).
    #[test]
    fn packet_batch_sll2_without_address_matches() {
        let details = LinuxSll2LinkOwned::new(
            0x0800,
            0,
            3,
            LinuxArphrdType::ETHERNET,
            LinuxCookedPacketType::HOST,
            0,
            None,
        );
        let flow = flow_with(
            LinkLayerOwned::linux_sll2(details),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            Some(full_transport()),
            Some(full_application()),
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(
            &real,
            &to_contract(&real),
            "packetBatch (Linux SLL2 sans adresse)",
        );
    }

    #[test]
    fn packet_batch_raw_ip_matches() {
        let flow = flow_with(
            LinkLayerOwned::raw_ipv4(),
            Some(full_internet(
                Ipv4Addr::new(192, 168, 1, 10),
                Ipv4Addr::new(8, 8, 8, 8),
            )),
            Some(full_transport()),
            Some(full_application()),
            None,
            None,
        );
        let real = packet_batch_of(flow, None);
        assert_same_json(&real, &to_contract(&real), "packetBatch (RAW IP)");
    }

    #[test]
    fn ieee80211_and_sll2_link_shapes_match() {
        // Couvre les deux variantes non exercées par les tests packetBatch
        // ci-dessus, pour que toute dérive de leur shape JSON (#[non_exhaustive]
        // côté packet_parser) casse ici plutôt qu'en silence.
        let ieee = LinkLayerOwned::ieee80211(Ieee80211LinkOwned::new(
            MacAddress([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]),
            MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            Ethertype(0x0800),
        ));
        assert_eq!(
            serde_json::to_value(&ieee).unwrap(),
            serde_json::to_value(DataLinkContract::from(&ieee)).unwrap(),
            "IEEE 802.11 : miroir divergent du réel"
        );

        let sll2 = LinkLayerOwned::linux_sll2(LinuxSll2LinkOwned::new(
            0x0800,
            0,
            3,
            LinuxArphrdType::ETHERNET,
            LinuxCookedPacketType::HOST,
            6,
            Some(vec![0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4]),
        ));
        assert_eq!(
            serde_json::to_value(&sll2).unwrap(),
            serde_json::to_value(DataLinkContract::from(&sll2)).unwrap(),
            "Linux SLL2 : miroir divergent du réel"
        );
    }

    /// Couvre les variantes de `NetworkProtocol`, `IpType` et
    /// `CorruptedLayerKind` qu'aucun fixture de paquet ci-dessus n'exerce.
    #[test]
    fn remaining_enum_shapes_match() {
        for protocol in [
            NetworkProtocol::Ipv6,
            NetworkProtocol::Arp,
            NetworkProtocol::Profinet,
            NetworkProtocol::Other(0xabcd),
        ] {
            assert_eq!(
                serde_json::to_value(protocol).unwrap(),
                serde_json::to_value(NetworkProtocolContract::from(protocol)).unwrap(),
                "NetworkProtocol {protocol:?} : miroir divergent du réel"
            );
        }

        for ip_type in [
            IpType::Private,
            IpType::Multicast,
            IpType::Loopback,
            IpType::Apipa,
            IpType::LinkLocal,
            IpType::Ula,
            IpType::Public,
            IpType::Documentation,
            IpType::Unknown,
        ] {
            assert_eq!(
                serde_json::to_value(&ip_type).unwrap(),
                serde_json::to_value(IpTypeContract::from(&ip_type)).unwrap(),
                "IpType {ip_type:?} : miroir divergent du réel"
            );
        }

        for kind in [CorruptedLayerKind::Internet, CorruptedLayerKind::Transport] {
            let real = CorruptedLayer {
                layer: kind,
                error: "détail".to_string(),
            };
            assert_eq!(
                serde_json::to_value(&real).unwrap(),
                serde_json::to_value(CorruptedLayerContract::from(&real)).unwrap(),
                "CorruptedLayer {kind:?} : miroir divergent du réel"
            );
        }
    }

    /// Génère la palette des catégories de nœuds à partir du graphe Rust
    /// réel. La légende Vue importe directement ce fichier ; la gate CI
    /// `export_ipc` le régénère puis vérifie le worktree, ce qui empêche une
    /// modification des couleurs backend de dériver silencieusement du
    /// frontend.
    #[test]
    fn export_ipc_graph_node_categories() {
        #[derive(Serialize)]
        struct Category {
            kind: &'static str,
            color: String,
        }

        let categories = [
            ("Private", Some(IpType::Private)),
            ("Public", Some(IpType::Public)),
            ("Multicast", Some(IpType::Multicast)),
            ("Loopback", Some(IpType::Loopback)),
            ("Apipa", Some(IpType::Apipa)),
            ("LinkLocal", Some(IpType::LinkLocal)),
            ("Ula", Some(IpType::Ula)),
            ("Documentation", Some(IpType::Documentation)),
            ("Layer2", None),
        ]
        .into_iter()
        .map(|(kind, ip_type)| Category {
            kind,
            color: rendered_graph_node_color(ip_type),
        })
        .collect::<Vec<_>>();

        // `Unknown` suit le repli L2 dans GraphData : il ne constitue pas
        // une dixième catégorie visuelle.
        assert_eq!(
            rendered_graph_node_color(Some(IpType::Unknown)),
            rendered_graph_node_color(None)
        );

        let json = serde_json::to_string_pretty(&categories).expect("sérialisation palette");
        let out = format!(
            "// Généré par `cargo test export_ipc_graph_node_categories` — NE PAS ÉDITER À LA MAIN.\n\
             // Couleurs observées en faisant passer chaque catégorie par GraphData.\n\
             import type {{ IpType }} from \"./IpType\";\n\n\
             export type GraphNodeCategory = Exclude<IpType, \"Unknown\"> | \"Layer2\";\n\n\
             export const GRAPH_NODE_CATEGORIES = ({json}) as const satisfies ReadonlyArray<{{\n\
               kind: GraphNodeCategory;\n\
               color: string;\n\
             }}>;\n"
        );

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/types/generated");
        std::fs::create_dir_all(&dir).expect("création du dossier généré");
        std::fs::write(dir.join("graphNodeCategories.ts"), out)
            .expect("écriture de la palette du graphe");
    }

    /// Écrit le JSON **réellement produit par `CaptureEvent`** (jamais par
    /// son miroir) dans `src/types/generated/captureEventFixtures.ts`, avec
    /// `satisfies CaptureEvent` (#142) : contrairement à une annotation
    /// directe `: CaptureEvent` (qui élargirait le type exporté à l'union
    /// entière) ou à un import JSON brut (qui élargirait les tags en
    /// `string`), `satisfies` conserve le type littéral précis de l'objet
    /// pour les consommateurs (`"started"`, pas `string`) TOUT EN vérifiant
    /// à l'assignation qu'il n'a ni champ manquant ni champ excédentaire par
    /// rapport à `CaptureEvent` — contrairement à l'ancien `fx<T extends
    /// CaptureEvent>(v: T): T` (inférence générique) qui ne détectait pas
    /// les champs excédentaires : un `#[ts(skip)]` accidentel sur un champ
    /// réellement présent dans le JSON restait invisible, `T` s'inférant
    /// alors sur la forme du literal lui-même plutôt que d'être vérifié
    /// contre elle. `as const` a un défaut similaire à l'envers : il rend
    /// aussi les tableaux (`macs`, `packets`, `updates`…) `readonly`,
    /// incompatibles avec les tableaux mutables de `CaptureEvent`.
    /// `src/types/captureContractFixtures.ts` importe ces constantes et les
    /// assigne à `CaptureEvent` : si `deno task typecheck` (CI) passe, du
    /// JSON réellement sorti de Rust — pas une valeur retapée à la main —
    /// satisfait le contrat généré au champ près. C'est le test Rust → JSON
    /// → TypeScript que #142 réclame, au sens strict.
    #[test]
    fn export_ipc_fixtures() {
        use std::fmt::Write as _;

        let mut out = String::from(
            "// Généré par `cargo test export_ipc_fixtures` (#142) — NE PAS ÉDITER À LA MAIN.\n\
             // JSON réellement produit par `CaptureEvent` (src-tauri/src/events/contract.rs,\n\
             // test export_ipc_fixtures). Relancer ce test après toute modification du\n\
             // contrat ; la CI échoue si le résultat commité a dérivé.\n\
             import type { CaptureEvent } from \"../capture\";\n\n",
        );

        // Alimente la garde « champ fantôme » ci-dessous (#142) : le JSON de
        // chaque fixture, une fois relu, sert à vérifier qu'aucun champ
        // optionnel déclaré par un type de contrat ne reste systématiquement
        // absent — ce que `satisfies` ne peut pas détecter (un champ jamais
        // alimenté par une donnée réelle est, par construction, toujours
        // omis des deux côtés, real et miroir, donc jamais en désaccord).
        let mut all_fixture_values: Vec<serde_json::Value> = Vec::new();

        macro_rules! fixture {
            ($name:ident, $value:expr) => {
                let json = serde_json::to_string_pretty(&$value).expect("sérialisation fixture");
                all_fixture_values
                    .push(serde_json::from_str(&json).expect("relecture JSON de la fixture"));
                // `satisfies` (pas `as const`, qui figerait aussi les tableaux
                // en `readonly` ; pas d'annotation directe `: CaptureEvent`,
                // qui élargirait le type exporté à l'union entière au lieu du
                // membre précis) : contrôle la forme À l'ASSIGNATION (donc
                // avec vérification des propriétés excédentaires — un champ
                // qu'un `#[ts(skip)]` aurait fait disparaître du type généré
                // resterait sinon invisible) tout en conservant le type
                // littéral précis de l'objet pour les consommateurs.
                writeln!(
                    out,
                    "export const {} = ({json}) satisfies CaptureEvent;\n",
                    stringify!($name)
                )
                .expect("écriture fixture");
            };
        }

        fixture!(
            startedFixture,
            crate::events::CaptureEvent::Started {
                session_id: 7,
                device: "eth0",
                buffer_size: 65536,
                chan_capacity: 1024,
                timeout: 100,
                snaplen: 262144,
                link_type: "Ethernet",
                protocol_version: crate::events::CAPTURE_EVENT_PROTOCOL_VERSION,
            }
        );
        fixture!(
            statsFixture,
            crate::events::CaptureEvent::Stats {
                session_id: 1,
                received: 10,
                dropped: 0,
                if_dropped: 0,
                app_dropped: 0,
                parse_errors: 1,
                processed: 3,
            }
        );
        fixture!(
            channelCapacityPayloadFixture,
            crate::events::CaptureEvent::ChannelCapacityPayload {
                session_id: 2,
                channel_size: 1024,
                current_size: 512,
                backpressure: true,
            }
        );
        fixture!(
            stoppedFixture,
            crate::events::CaptureEvent::Stopped {
                session_id: 3,
                reason: "erreur pcap".to_string(),
            }
        );
        fixture!(
            finishedFixture,
            crate::events::CaptureEvent::Finished {
                file_name: "capture.pcap",
                packet_total_count: 100,
                integrated_count: 95,
                rejected_truncated_count: 3,
                rejected_unsupported_link_type_count: 1,
                rejected_malformed_count: 1,
                matrix_total_count: 40,
            }
        );
        fixture!(
            importProgressFixture,
            crate::events::CaptureEvent::ImportProgress {
                file_name: "capture.pcap",
                file_index: 2,
                files_total: 5,
                current: 1_000,
                total: 48_350,
            }
        );

        let node_added = GraphUpdate::NewNode(sample_node());
        fixture!(
            graphNodeAddedFixture,
            crate::events::CaptureEvent::Graph {
                update: &node_added
            }
        );
        let node_updated = GraphUpdate::NodeUpdated(sample_node());
        fixture!(
            graphNodeUpdatedFixture,
            crate::events::CaptureEvent::Graph {
                update: &node_updated
            }
        );
        fixture!(
            graphBatchFixture,
            crate::events::CaptureEvent::GraphBatch {
                session_id: 5,
                updates: vec![
                    GraphUpdate::NewEdge(sample_edge()),
                    GraphUpdate::EdgeUpdated(sample_edge()),
                ],
            }
        );
        // Nœud sans label, arête sans port : sans ce batch, `label` sur
        // `Node` et `sourcePort`/`destinationPort` sur `Edge` ne seraient
        // jamais exercés à `null` dans un fixture consommé par TypeScript
        // (#142) — retirer leur `| null` du type généré resterait invisible
        // au typecheck.
        fixture!(
            graphBatchWithoutLabelOrPortsFixture,
            crate::events::CaptureEvent::GraphBatch {
                session_id: 5,
                updates: vec![
                    GraphUpdate::NewNode(sample_node_without_label()),
                    GraphUpdate::NewEdge(sample_edge_without_ports()),
                ],
            }
        );
        let node = sample_node();
        let edge = sample_edge();
        let mut nodes = HashMap::new();
        nodes.insert(node.id.clone(), node);
        let mut edges = HashMap::new();
        edges.insert(edge.id.clone(), edge);
        let mut graph_data = crate::state::graph::GraphData::new();
        graph_data.nodes = nodes;
        graph_data.edges = edges;
        fixture!(
            graphSnapshotFixture,
            crate::events::CaptureEvent::GraphSnapshot {
                graph_data: &graph_data
            }
        );

        fixture!(
            packetBatchEthernetWithVlanFixture,
            packet_batch_of(
                flow_with(
                    ethernet_link(Some(VlanTag {
                        id: 42,
                        pcp: 0,
                        dei: false,
                        inner_ethertype: Ethertype(0x0800),
                    })),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    Some(full_transport()),
                    Some(full_application()),
                    None,
                    None,
                ),
                // encap_id non nul : distingue explicitement du `null` des
                // autres fixtures (une régression string -> number ne peut
                // pas se cacher derrière deux `null`, cf. revue #142).
                Some(0x0000_0000_dead_beef),
            )
        );
        fixture!(
            packetBatchMinimalFixture,
            packet_batch_of(
                flow_with(ethernet_link(None), None, None, None, None, None),
                None
            )
        );
        let inner_flow = flow_with(
            ethernet_link(None),
            Some(full_internet(
                Ipv4Addr::new(10, 0, 0, 1),
                Ipv4Addr::new(10, 0, 0, 2),
            )),
            None,
            None,
            None,
            None,
        );
        fixture!(
            packetBatchTunneledFixture,
            packet_batch_of(
                flow_with(
                    ethernet_link(None),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    None,
                    None,
                    Some(Box::new(inner_flow)),
                    None,
                ),
                Some(0x1122_3344_5566_7788),
            )
        );
        // Les deux variantes de `CorruptedLayerKind` (#142) : un seul paquet
        // n'en exercerait qu'une, laissant l'autre absente de tout fixture
        // consommé côté TypeScript.
        fixture!(
            packetBatchCorruptedFixture,
            crate::events::CaptureEvent::PacketBatch {
                session_id: 1,
                packets: vec![
                    CapturedPacketOwned {
                        ts_sec: 0,
                        ts_usec: 0,
                        caplen: 60,
                        len: 60,
                        flow: flow_with(
                            ethernet_link(None),
                            None,
                            None,
                            None,
                            None,
                            Some(CorruptedLayer {
                                layer: CorruptedLayerKind::Internet,
                                error: "en-tête IPv4 tronqué".to_string(),
                            }),
                        ),
                        encap_id: None,
                    },
                    CapturedPacketOwned {
                        ts_sec: 0,
                        ts_usec: 0,
                        caplen: 60,
                        len: 60,
                        flow: flow_with(
                            ethernet_link(None),
                            None,
                            None,
                            None,
                            None,
                            Some(CorruptedLayer {
                                layer: CorruptedLayerKind::Transport,
                                error: "en-tête TCP tronqué".to_string(),
                            }),
                        ),
                        encap_id: None,
                    },
                ],
            }
        );
        fixture!(
            packetBatchSllFixture,
            packet_batch_of(
                flow_with(
                    LinkLayerOwned::linux_sll(LinuxSllLinkOwned::new(
                        LinuxCookedPacketType::OUTGOING,
                        LinuxArphrdType::ETHERNET,
                        6,
                        Some(vec![0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4]),
                        0x0800,
                    )),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    Some(full_transport()),
                    None,
                    None,
                    None,
                ),
                None,
            )
        );
        // Groupes internet/transport présents mais chaque champ individuel
        // absent : distinct du cas « groupe absent » (packetBatchMinimalFixture)
        // que `full_internet`/`full_transport` (toujours `Some`) n'exercent
        // jamais côté TypeScript (#142).
        fixture!(
            packetBatchInternetAndTransportPresentWithoutAddressesFixture,
            packet_batch_of(
                flow_with(
                    ethernet_link(None),
                    Some(internet_without_addresses()),
                    Some(transport_without_ports()),
                    None,
                    None,
                    None,
                ),
                None,
            )
        );
        // SLL sans adresse source : `packetBatchSllFixture` en fixe toujours
        // une, laissant `source_address: number[] | null` sans fixture TS
        // exerçant le cas `null` (#142).
        fixture!(
            packetBatchSllWithoutAddressFixture,
            packet_batch_of(
                flow_with(
                    LinkLayerOwned::linux_sll(LinuxSllLinkOwned::new(
                        LinuxCookedPacketType::OUTGOING,
                        LinuxArphrdType::ETHERNET,
                        0,
                        None,
                        0x0800,
                    )),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    Some(full_transport()),
                    None,
                    None,
                    None,
                ),
                None,
            )
        );
        fixture!(
            packetBatchSll2Fixture,
            packet_batch_of(
                flow_with(
                    LinkLayerOwned::linux_sll2(LinuxSll2LinkOwned::new(
                        0x0800,
                        0,
                        3,
                        LinuxArphrdType::ETHERNET,
                        LinuxCookedPacketType::HOST,
                        6,
                        Some(vec![0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4]),
                    )),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    None,
                    None,
                    None,
                    None,
                ),
                None,
            )
        );
        // SLL2 sans adresse source : `packetBatchSll2Fixture` en fixe
        // toujours une, laissant `source_address: number[] | null` sans
        // fixture TS exerçant le cas `null` (#142).
        fixture!(
            packetBatchSll2WithoutAddressFixture,
            packet_batch_of(
                flow_with(
                    LinkLayerOwned::linux_sll2(LinuxSll2LinkOwned::new(
                        0x0800,
                        0,
                        3,
                        LinuxArphrdType::ETHERNET,
                        LinuxCookedPacketType::HOST,
                        0,
                        None,
                    )),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    Some(full_transport()),
                    None,
                    None,
                    None,
                ),
                None,
            )
        );
        fixture!(
            packetBatchIeee80211Fixture,
            packet_batch_of(
                flow_with(
                    LinkLayerOwned::ieee80211(Ieee80211LinkOwned::new(
                        MacAddress([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]),
                        MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
                        Ethertype(0x0800),
                    )),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    None,
                    None,
                    None,
                    None,
                ),
                None,
            )
        );
        fixture!(
            packetBatchRawIpFixture,
            packet_batch_of(
                flow_with(
                    LinkLayerOwned::raw_ipv4(),
                    Some(full_internet(
                        Ipv4Addr::new(192, 168, 1, 10),
                        Ipv4Addr::new(8, 8, 8, 8)
                    )),
                    Some(full_transport()),
                    Some(full_application()),
                    None,
                    None,
                ),
                None,
            )
        );

        // Une variante d'`IpType` par paquet (#142) : les autres fixtures
        // `packetBatch` ne construisent que `Private`/`Public` (via
        // `full_internet`), ce qui laisserait les autres variantes absentes
        // de tout fixture consommé côté TypeScript (elles ne restent
        // couvertes que par `remaining_enum_shapes_match`, côté Rust seul).
        fixture!(
            packetBatchAllIpTypesFixture,
            crate::events::CaptureEvent::PacketBatch {
                session_id: 1,
                packets: [
                    IpType::Private,
                    IpType::Multicast,
                    IpType::Loopback,
                    IpType::Apipa,
                    IpType::LinkLocal,
                    IpType::Ula,
                    IpType::Public,
                    IpType::Documentation,
                    IpType::Unknown,
                ]
                .into_iter()
                .map(|ip_type| CapturedPacketOwned {
                    ts_sec: 0,
                    ts_usec: 0,
                    caplen: 60,
                    len: 60,
                    flow: flow_with(
                        ethernet_link(None),
                        Some(internet_with(
                            Ipv4Addr::new(192, 168, 1, 10),
                            ip_type.clone(),
                            Ipv4Addr::new(8, 8, 8, 8),
                            ip_type,
                        )),
                        None,
                        None,
                        None,
                        None,
                    ),
                    encap_id: None,
                })
                .collect(),
            }
        );

        // Une variante de `NetworkProtocol` par paquet, au-delà de l'`Ipv4`
        // implicite des autres fixtures Ethernet (dérivé de l'ethertype par
        // `LinkLayerOwned::ethernet`, cf. `packet_parser`) : `Ipv6`, `Arp`,
        // `Profinet` et un `Other` numérique (#142).
        fixture!(
            packetBatchAllNetworkProtocolsFixture,
            crate::events::CaptureEvent::PacketBatch {
                session_id: 1,
                packets: [0x86ddu16, 0x0806, 0x8892, 0x88b5]
                    .into_iter()
                    .map(|ethertype| CapturedPacketOwned {
                        ts_sec: 0,
                        ts_usec: 0,
                        caplen: 60,
                        len: 60,
                        flow: flow_with(
                            ethernet_link_with_ethertype(ethertype),
                            None,
                            None,
                            None,
                            None,
                            None,
                        ),
                        encap_id: None,
                    })
                    .collect(),
            }
        );

        assert_no_phantom_contract_fields(&all_fixture_values);

        // Chaque fixture ajoute une ligne vide de séparation après elle
        // (writeln!) : après la dernière, ça laisse une ligne vide en fin de
        // fichier que `git diff --check` rejette ("new blank line at EOF").
        out.truncate(out.trim_end().len());
        out.push('\n');

        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/types/generated");
        // Créé explicitement : ce test peut s'exécuter avant
        // `export_ipc_bindings` (ordre non garanti entre tests parallèles),
        // qui est jusqu'ici le seul à créer ce dossier via `ts-rs`.
        std::fs::create_dir_all(&dir).expect("création du dossier des fixtures");
        std::fs::write(dir.join("captureEventFixtures.ts"), out).expect("écriture des fixtures");
    }

    /// Garde « champ fantôme » (#142) : vérifie qu'aucun champ optionnel
    /// déclaré par un type de contrat n'est *systématiquement* absent de
    /// tous les fixtures. Les types de contrat sont recopiés à la main
    /// (`NodeContract`, `EdgeContract`…) : rien n'empêche syntaxiquement d'y
    /// ajouter un champ qui ne provient d'aucune donnée réelle. Un tel champ
    /// fantôme, toujours `None`, est donc toujours omis — des deux côtés,
    /// réel et miroir — et `assert_same_json` ne peut jamais le remarquer,
    /// puisqu'il compare des JSON qui restent d'accord en silence. Ici, on
    /// vérifie plutôt qu'il existe AU MOINS UN fixture où le champ apparaît
    /// avec une valeur présente : un champ qui n'apparaît jamais est un
    /// candidat direct à un champ jamais produit par le vrai backend.
    ///
    /// Heuristique de couverture, pas une garantie structurelle : un champ
    /// fantôme accompagné d'un faux fixture plausible resterait invisible.
    /// Mais elle attrape le cas concret trouvé en revue (champ optionnel
    /// ajouté au miroir, jamais alimenté par `to_contract`).
    fn assert_no_phantom_contract_fields(fixtures: &[serde_json::Value]) {
        let cfg = ts_rs::Config::default();
        let mut declared_optional_fields = std::collections::HashSet::new();
        for decl in [
            CaptureEventContract::decl(&cfg),
            NodeContract::decl(&cfg),
            EdgeContract::decl(&cfg),
            GraphUpdateContract::decl(&cfg),
            GraphDataContract::decl(&cfg),
            LinkLayerKindContract::decl(&cfg),
            DataLinkContract::decl(&cfg),
            DataLinkDetailsContract::decl(&cfg),
            RawIpLinkDetailsContract::decl(&cfg),
            LinuxSllLinkDetailsContract::decl(&cfg),
            LinuxSll2LinkDetailsContract::decl(&cfg),
            Ieee80211LinkDetailsContract::decl(&cfg),
            NetworkProtocolContract::decl(&cfg),
            VlanTagContract::decl(&cfg),
            IpTypeContract::decl(&cfg),
            CorruptedLayerKindContract::decl(&cfg),
            CorruptedLayerContract::decl(&cfg),
            PacketFlowContract::decl(&cfg),
            PacketBatchPacketContract::decl(&cfg),
        ] {
            collect_optional_field_names(&decl, &mut declared_optional_fields);
        }

        let mut present_fields = std::collections::HashSet::new();
        for value in fixtures {
            collect_present_field_names(value, &mut present_fields);
        }

        let mut phantom_fields: Vec<&String> = declared_optional_fields
            .iter()
            .filter(|f| !present_fields.contains(*f))
            .collect();
        phantom_fields.sort();
        assert!(
            phantom_fields.is_empty(),
            "champ(s) optionnel(s) déclaré(s) dans le contrat mais jamais \
             présent(s) dans aucun fixture (candidat à un champ jamais \
             produit par le vrai backend, #142) : {phantom_fields:?}"
        );
    }

    /// Extrait les noms de champs optionnels (`champ?:` ou `"champ"?:`) d'une
    /// déclaration TypeScript générée par ts-rs. Scan caractère par
    /// caractère plutôt qu'un vrai parseur TS : le format émis par ts-rs est
    /// assez régulier pour que ça suffise.
    fn collect_optional_field_names(ts_decl: &str, into: &mut std::collections::HashSet<String>) {
        let bytes = ts_decl.as_bytes();
        for i in 0..bytes.len().saturating_sub(1) {
            if bytes[i] != b'?' || bytes[i + 1] != b':' {
                continue;
            }
            let mut end = i;
            if end > 0 && bytes[end - 1] == b'"' {
                end -= 1;
            }
            let mut start = end;
            while start > 0
                && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_')
            {
                start -= 1;
            }
            if start < end {
                into.insert(ts_decl[start..end].to_string());
            }
        }
    }

    /// Parcourt récursivement un JSON de fixture et retient les clés dont la
    /// valeur n'est ni `null` ni absente.
    fn collect_present_field_names(
        value: &serde_json::Value,
        into: &mut std::collections::HashSet<String>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, v) in map {
                    if !v.is_null() {
                        into.insert(key.clone());
                    }
                    collect_present_field_names(v, into);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    collect_present_field_names(item, into);
                }
            }
            _ => {}
        }
    }
}
