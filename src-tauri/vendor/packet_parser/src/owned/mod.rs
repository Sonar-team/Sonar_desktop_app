// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::Serialize;
use std::{
    fmt::{Display, Formatter, Result},
    net::IpAddr,
};

use crate::parse::CorruptedLayer;
use crate::parse::data_link::vlan_tag::VlanTag;
use crate::parse::data_link::{ethertype, ethertype::Ethertype, mac_addres::MacAddress};
use crate::parse::link_layer::{
    Ieee80211Link, LinkLayer, LinkLayerKind, LinuxArphrdType, LinuxCookedPacketType, LinuxSll2Link,
    LinuxSllLink, NetworkProtocol, RawIpLink,
};
use crate::{DataLink, IpType, LinkType, PacketFlow};

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct PacketFlowOwned {
    pub data_link: LinkLayerOwned,
    #[serde(flatten)]
    pub internet: Option<InternetOwned>,
    #[serde(flatten)]
    pub transport: Option<TransportOwned>,
    #[serde(flatten)]
    pub application: Option<ApplicationOwned>,
    /// Encapsulated flow when this flow is a tunnel (e.g. CAPWAP), mirroring
    /// [`PacketFlow::inner`](crate::PacketFlow).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner: Option<Box<PacketFlowOwned>>,
    /// Corruption report, mirroring [`PacketFlow::corrupted`](crate::PacketFlow).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrupted: Option<CorruptedLayer>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct DataLinkOwned {
    pub destination_mac: MacAddress,
    /// The source MAC address.
    pub source_mac: MacAddress,
    /// The Ethertype of the packet, indicating the protocol in the payload.
    #[serde(serialize_with = "ethertype::serialize_name")]
    pub ethertype: Ethertype,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<VlanTag>,
}

/// Owned format-specific link-layer information.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
#[serde(tag = "link_kind", content = "link_details", rename_all = "snake_case")]
pub enum LinkLayerOwnedKind {
    Ethernet(DataLinkOwned),
    RawIp(RawIpLinkOwned),
    LinuxSll(LinuxSllLinkOwned),
    LinuxSll2(LinuxSll2LinkOwned),
    Ieee80211(Ieee80211LinkOwned),
}

/// Owned RAW IP metadata. RAW has no link-layer addresses or EtherType.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Hash, Eq)]
pub struct RawIpLinkOwned {
    pub ip_version: u8,
}

/// Owned Linux cooked capture v1 metadata.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct LinuxSllLinkOwned {
    pub packet_type: LinuxCookedPacketType,
    pub hardware_type: LinuxArphrdType,
    pub address_length: u16,
    pub source_address: Option<Vec<u8>>,
    pub protocol: u16,
}

impl LinuxSllLinkOwned {
    /// Builds owned SLL metadata without going through the parser — the path
    /// a consumer takes to rebuild a link layer from serialized fields (e.g.
    /// reimporting an exported flow matrix). The struct stays
    /// `#[non_exhaustive]`, so this constructor is the supported way to
    /// create one from outside the crate.
    pub fn new(
        packet_type: LinuxCookedPacketType,
        hardware_type: LinuxArphrdType,
        address_length: u16,
        source_address: Option<Vec<u8>>,
        protocol: u16,
    ) -> Self {
        Self {
            packet_type,
            hardware_type,
            address_length,
            source_address,
            protocol,
        }
    }

    pub const fn address_is_truncated(&self) -> bool {
        self.address_length > 8
    }
}

/// Owned Linux cooked capture v2 metadata.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct LinuxSll2LinkOwned {
    pub protocol: u16,
    pub reserved_mbz: u16,
    pub interface_index: u32,
    pub hardware_type: LinuxArphrdType,
    pub packet_type: LinuxCookedPacketType,
    pub address_length: u8,
    pub source_address: Option<Vec<u8>>,
}

impl LinuxSll2LinkOwned {
    /// Builds owned SLL2 metadata without going through the parser — see
    /// [`LinuxSllLinkOwned::new`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        protocol: u16,
        reserved_mbz: u16,
        interface_index: u32,
        hardware_type: LinuxArphrdType,
        packet_type: LinuxCookedPacketType,
        address_length: u8,
        source_address: Option<Vec<u8>>,
    ) -> Self {
        Self {
            protocol,
            reserved_mbz,
            interface_index,
            hardware_type,
            packet_type,
            address_length,
            source_address,
        }
    }

    pub const fn reserved_is_zero(&self) -> bool {
        self.reserved_mbz == 0
    }

    pub const fn address_is_truncated(&self) -> bool {
        self.address_length > 8
    }
}

/// Owned IEEE 802.11 fields produced by tunnel peeling.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct Ieee80211LinkOwned {
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
    #[serde(serialize_with = "ethertype::serialize_name")]
    pub snap_protocol: Ethertype,
}

impl Ieee80211LinkOwned {
    pub const fn new(
        destination_mac: MacAddress,
        source_mac: MacAddress,
        snap_protocol: Ethertype,
    ) -> Self {
        Self {
            destination_mac,
            source_mac,
            snap_protocol,
        }
    }
}

/// Owned counterpart of [`LinkLayer`].
#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct LinkLayerOwned {
    link_type: LinkType,
    network_protocol: NetworkProtocol,
    #[serde(flatten)]
    kind: LinkLayerOwnedKind,
}

impl LinkLayerOwned {
    pub fn ethernet(frame: DataLinkOwned) -> Self {
        Self {
            link_type: LinkType::ETHERNET,
            network_protocol: frame.ethertype.into(),
            kind: LinkLayerOwnedKind::Ethernet(frame),
        }
    }

    fn raw_ip(network_protocol: NetworkProtocol, details: RawIpLinkOwned) -> Self {
        Self {
            link_type: LinkType::RAW,
            network_protocol,
            kind: LinkLayerOwnedKind::RawIp(details),
        }
    }

    pub fn raw_ipv4() -> Self {
        Self::raw_ip(NetworkProtocol::Ipv4, RawIpLinkOwned { ip_version: 4 })
    }

    pub fn raw_ipv6() -> Self {
        Self::raw_ip(NetworkProtocol::Ipv6, RawIpLinkOwned { ip_version: 6 })
    }

    /// Wraps owned SLL metadata into a link layer, deriving the network
    /// protocol from the cooked `protocol` field — same derivation as the
    /// live decoder, so a rebuilt layer compares equal to a parsed one.
    pub fn linux_sll(details: LinuxSllLinkOwned) -> Self {
        Self {
            link_type: LinkType::LINUX_SLL,
            network_protocol: NetworkProtocol::from_link_protocol(details.protocol),
            kind: LinkLayerOwnedKind::LinuxSll(details),
        }
    }

    /// SLL2 counterpart of [`LinkLayerOwned::linux_sll`].
    pub fn linux_sll2(details: LinuxSll2LinkOwned) -> Self {
        Self {
            link_type: LinkType::LINUX_SLL2,
            network_protocol: NetworkProtocol::from_link_protocol(details.protocol),
            kind: LinkLayerOwnedKind::LinuxSll2(details),
        }
    }

    pub fn ieee80211(frame: Ieee80211LinkOwned) -> Self {
        Self {
            link_type: LinkType::IEEE802_11,
            network_protocol: frame.snap_protocol.into(),
            kind: LinkLayerOwnedKind::Ieee80211(frame),
        }
    }

    pub const fn link_type(&self) -> LinkType {
        self.link_type
    }

    pub const fn network_protocol(&self) -> NetworkProtocol {
        self.network_protocol
    }

    pub const fn kind(&self) -> &LinkLayerOwnedKind {
        &self.kind
    }

    pub const fn as_ethernet(&self) -> Option<&DataLinkOwned> {
        match &self.kind {
            LinkLayerOwnedKind::Ethernet(frame) => Some(frame),
            LinkLayerOwnedKind::RawIp(_) => None,
            LinkLayerOwnedKind::LinuxSll(_) => None,
            LinkLayerOwnedKind::LinuxSll2(_) => None,
            LinkLayerOwnedKind::Ieee80211(_) => None,
        }
    }

    pub const fn as_raw_ip(&self) -> Option<&RawIpLinkOwned> {
        match &self.kind {
            LinkLayerOwnedKind::RawIp(details) => Some(details),
            LinkLayerOwnedKind::Ethernet(_)
            | LinkLayerOwnedKind::LinuxSll(_)
            | LinkLayerOwnedKind::LinuxSll2(_)
            | LinkLayerOwnedKind::Ieee80211(_) => None,
        }
    }

    pub const fn as_linux_sll(&self) -> Option<&LinuxSllLinkOwned> {
        match &self.kind {
            LinkLayerOwnedKind::LinuxSll(details) => Some(details),
            LinkLayerOwnedKind::Ethernet(_)
            | LinkLayerOwnedKind::RawIp(_)
            | LinkLayerOwnedKind::LinuxSll2(_)
            | LinkLayerOwnedKind::Ieee80211(_) => None,
        }
    }

    pub const fn as_linux_sll2(&self) -> Option<&LinuxSll2LinkOwned> {
        match &self.kind {
            LinkLayerOwnedKind::LinuxSll2(details) => Some(details),
            LinkLayerOwnedKind::Ethernet(_)
            | LinkLayerOwnedKind::RawIp(_)
            | LinkLayerOwnedKind::LinuxSll(_)
            | LinkLayerOwnedKind::Ieee80211(_) => None,
        }
    }

    pub const fn as_ieee80211(&self) -> Option<&Ieee80211LinkOwned> {
        match &self.kind {
            LinkLayerOwnedKind::Ethernet(_) => None,
            LinkLayerOwnedKind::RawIp(_) => None,
            LinkLayerOwnedKind::LinuxSll(_) => None,
            LinkLayerOwnedKind::LinuxSll2(_) => None,
            LinkLayerOwnedKind::Ieee80211(frame) => Some(frame),
        }
    }
}

impl Display for DataLinkOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "\n    Destination MAC: {},\n    Source MAC: {},\n    Ethertype: {},\n    VLAN: ",
            self.destination_mac,
            self.source_mac,
            self.ethertype.name(),
        )?;

        match &self.vlan {
            Some(vlan) => write!(f, "{vlan}")?,
            None => write!(f, "None")?,
        }

        writeln!(f)
    }
}

impl Display for LinkLayerOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.kind {
            LinkLayerOwnedKind::Ethernet(frame) => frame.fmt(f),
            LinkLayerOwnedKind::RawIp(details) => write!(
                f,
                "\n    RAW IP Version: {},\n    Protocol: {}\n",
                details.ip_version, self.network_protocol
            ),
            LinkLayerOwnedKind::LinuxSll(details) => write!(
                f,
                "\n    Linux SLL Packet Type: {},\n    Hardware Type: {},\n    Address Length: {},\n    Protocol: 0x{:04X}\n",
                details.packet_type,
                details.hardware_type,
                details.address_length,
                details.protocol
            ),
            LinkLayerOwnedKind::LinuxSll2(details) => write!(
                f,
                "\n    Linux SLL2 Interface Index: {},\n    Packet Type: {},\n    Hardware Type: {},\n    Address Length: {},\n    Protocol: 0x{:04X},\n    Reserved MBZ: 0x{:04X}\n",
                details.interface_index,
                details.packet_type,
                details.hardware_type,
                details.address_length,
                details.protocol,
                details.reserved_mbz
            ),
            LinkLayerOwnedKind::Ieee80211(frame) => write!(
                f,
                "\n    IEEE 802.11 Destination MAC: {},\n    Source MAC: {},\n    SNAP Protocol: {}\n",
                frame.destination_mac,
                frame.source_mac,
                frame.snap_protocol.name()
            ),
        }
    }
}

impl Display for InternetOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Pas d’allocation: on écrit directement.
        write!(f, "\n    Source IP: ")?;
        match &self.source_ip {
            Some(ip) => write!(f, "{ip}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Destination IP: ")?;
        match &self.destination_ip {
            Some(ip) => write!(f, "{ip}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Protocol: {}\n", self.protocol)
    }
}

impl Display for TransportOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\n    Source Port: ")?;
        match self.source_port {
            Some(p) => write!(f, "{p}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Destination Port: ")?;
        match self.destination_port {
            Some(p) => write!(f, "{p}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Protocol: {}\n", self.protocol)
    }
}

impl Display for ApplicationOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\n    Protocol: {}\n", self.protocol)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct InternetOwned {
    pub source_ip: Option<IpAddr>,
    pub ip_source_type: Option<IpType>,
    pub destination_ip: Option<IpAddr>,
    pub ip_destination_type: Option<IpType>,
    #[serde(rename = "protocol_internet")]
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct TransportOwned {
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    #[serde(rename = "protocol_transport")]
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct ApplicationOwned {
    #[serde(rename = "application_protocol")]
    pub protocol: String,
}

impl<'a> From<&PacketFlow<'a>> for PacketFlowOwned {
    fn from(flow: &PacketFlow<'a>) -> Self {
        Self {
            data_link: LinkLayerOwned::from(&flow.data_link),
            internet: flow.internet.as_ref().map(|internet| InternetOwned {
                source_ip: internet.source,
                ip_source_type: internet.source_type.clone(),
                destination_ip: internet.destination,
                ip_destination_type: internet.destination_type.clone(),
                protocol: internet.protocol_name.to_string(),
            }),
            transport: flow.transport.as_ref().map(|transport| TransportOwned {
                source_port: transport.source_port,
                destination_port: transport.destination_port,
                protocol: transport.protocol.to_string(),
            }),
            application: flow
                .application
                .as_ref()
                .map(|application| ApplicationOwned {
                    protocol: application.application_protocol.to_string(),
                }),
            inner: flow
                .inner
                .as_deref()
                .map(|inner| Box::new(PacketFlowOwned::from(inner))),
            corrupted: flow.corrupted.clone(),
        }
    }
}

impl From<&DataLink<'_>> for DataLinkOwned {
    fn from(frame: &DataLink<'_>) -> Self {
        Self {
            destination_mac: frame.destination_mac,
            source_mac: frame.source_mac,
            ethertype: frame.ethertype,
            vlan: frame.vlan.clone(),
        }
    }
}

impl From<&Ieee80211Link<'_>> for Ieee80211LinkOwned {
    fn from(frame: &Ieee80211Link<'_>) -> Self {
        Self::new(frame.destination_mac, frame.source_mac, frame.snap_protocol)
    }
}

impl From<&RawIpLink<'_>> for RawIpLinkOwned {
    fn from(details: &RawIpLink<'_>) -> Self {
        Self {
            ip_version: details.ip_version,
        }
    }
}

impl From<&LinuxSllLink<'_>> for LinuxSllLinkOwned {
    fn from(details: &LinuxSllLink<'_>) -> Self {
        Self {
            packet_type: details.packet_type,
            hardware_type: details.hardware_type,
            address_length: details.address_length,
            source_address: details.source_address.map(<[u8]>::to_vec),
            protocol: details.protocol,
        }
    }
}

impl From<&LinuxSll2Link<'_>> for LinuxSll2LinkOwned {
    fn from(details: &LinuxSll2Link<'_>) -> Self {
        Self {
            protocol: details.protocol,
            reserved_mbz: details.reserved_mbz,
            interface_index: details.interface_index,
            hardware_type: details.hardware_type,
            packet_type: details.packet_type,
            address_length: details.address_length,
            source_address: details.source_address.map(<[u8]>::to_vec),
        }
    }
}

impl From<&LinkLayer<'_>> for LinkLayerOwned {
    fn from(layer: &LinkLayer<'_>) -> Self {
        match layer.kind() {
            LinkLayerKind::Ethernet(frame) => Self::ethernet(DataLinkOwned::from(frame)),
            LinkLayerKind::RawIp(details) => {
                Self::raw_ip(layer.network_protocol(), RawIpLinkOwned::from(details))
            }
            LinkLayerKind::LinuxSll(details) => Self::linux_sll(LinuxSllLinkOwned::from(details)),
            LinkLayerKind::LinuxSll2(details) => {
                Self::linux_sll2(LinuxSll2LinkOwned::from(details))
            }
            LinkLayerKind::Ieee80211(frame) => Self::ieee80211(Ieee80211LinkOwned::from(frame)),
        }
    }
}

impl<'a> From<PacketFlow<'a>> for PacketFlowOwned {
    fn from(flow: PacketFlow<'a>) -> Self {
        Self::from(&flow)
    }
}

impl Display for PacketFlowOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Packet Flow:")?;
        writeln!(f, "  Data Link: {}", self.data_link)?;

        if let Some(internet) = &self.internet {
            writeln!(f, "  Internet: {internet}")?;
        }
        if let Some(transport) = &self.transport {
            writeln!(f, "  Transport: {transport}")?;
        }
        if let Some(application) = &self.application {
            writeln!(f, "  Application: {application}")?;
        }
        if let Some(inner) = &self.inner {
            writeln!(f, "  Inner {inner}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::net::{IpAddr, Ipv4Addr};

    fn sample_data_link_without_vlan() -> DataLinkOwned {
        DataLinkOwned {
            destination_mac: MacAddress([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]),
            source_mac: MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            ethertype: Ethertype(0x0800),
            vlan: None,
        }
    }

    fn sample_link_layer_owned() -> LinkLayerOwned {
        LinkLayerOwned::ethernet(sample_data_link_without_vlan())
    }

    fn sample_internet() -> InternetOwned {
        InternetOwned {
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            ip_source_type: None,
            destination_ip: Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            ip_destination_type: None,
            protocol: "TCP".to_string(),
        }
    }

    fn sample_transport() -> TransportOwned {
        TransportOwned {
            source_port: Some(12345),
            destination_port: Some(80),
            protocol: "TCP".to_string(),
        }
    }

    fn sample_application() -> ApplicationOwned {
        ApplicationOwned {
            protocol: "HTTP".to_string(),
        }
    }

    fn sample_packet_flow_owned() -> PacketFlowOwned {
        PacketFlowOwned {
            data_link: sample_link_layer_owned(),
            internet: Some(sample_internet()),
            transport: Some(sample_transport()),
            application: Some(sample_application()),
            inner: None,
            corrupted: None,
        }
    }

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_data_link_owned_display_without_vlan() {
        let data_link = sample_data_link_without_vlan();

        let expected = "\n    Destination MAC: aa:bb:cc:dd:ee:ff,\n    Source MAC: 11:22:33:44:55:66,\n    Ethertype: IPv4,\n    VLAN: None\n";
        assert_eq!(data_link.to_string(), expected);
    }

    #[test]
    fn test_internet_owned_display_with_ips() {
        let internet = sample_internet();

        let expected =
            "\n    Source IP: 192.168.1.10,\n    Destination IP: 8.8.8.8,\n    Protocol: TCP\n";
        assert_eq!(internet.to_string(), expected);
    }

    #[test]
    fn test_internet_owned_display_with_none_ips() {
        let internet = InternetOwned {
            source_ip: None,
            ip_source_type: None,
            destination_ip: None,
            ip_destination_type: None,
            protocol: "UDP".to_string(),
        };

        let expected = "\n    Source IP: None,\n    Destination IP: None,\n    Protocol: UDP\n";
        assert_eq!(internet.to_string(), expected);
    }

    #[test]
    fn test_transport_owned_display_with_ports() {
        let transport = sample_transport();

        let expected = "\n    Source Port: 12345,\n    Destination Port: 80,\n    Protocol: TCP\n";
        assert_eq!(transport.to_string(), expected);
    }

    #[test]
    fn test_transport_owned_display_with_none_ports() {
        let transport = TransportOwned {
            source_port: None,
            destination_port: None,
            protocol: "ICMP".to_string(),
        };

        let expected =
            "\n    Source Port: None,\n    Destination Port: None,\n    Protocol: ICMP\n";
        assert_eq!(transport.to_string(), expected);
    }

    #[test]
    fn test_application_owned_display() {
        let application = sample_application();

        let expected = "\n    Protocol: HTTP\n";
        assert_eq!(application.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_owned_display_full() {
        let flow = sample_packet_flow_owned();

        let expected = concat!(
            "Packet Flow:\n",
            "  Data Link: \n",
            "    Destination MAC: aa:bb:cc:dd:ee:ff,\n",
            "    Source MAC: 11:22:33:44:55:66,\n",
            "    Ethertype: IPv4,\n",
            "    VLAN: None\n",
            "\n",
            "  Internet: \n",
            "    Source IP: 192.168.1.10,\n",
            "    Destination IP: 8.8.8.8,\n",
            "    Protocol: TCP\n",
            "\n",
            "  Transport: \n",
            "    Source Port: 12345,\n",
            "    Destination Port: 80,\n",
            "    Protocol: TCP\n",
            "\n",
            "  Application: \n",
            "    Protocol: HTTP\n",
            "\n"
        );

        assert_eq!(flow.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_owned_display_only_data_link() {
        let flow = PacketFlowOwned {
            data_link: sample_link_layer_owned(),
            internet: None,
            transport: None,
            application: None,
            inner: None,
            corrupted: None,
        };

        let expected = concat!(
            "Packet Flow:\n",
            "  Data Link: \n",
            "    Destination MAC: aa:bb:cc:dd:ee:ff,\n",
            "    Source MAC: 11:22:33:44:55:66,\n",
            "    Ethertype: IPv4,\n",
            "    VLAN: None\n",
            "\n"
        );

        assert_eq!(flow.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_owned_clone_and_eq() {
        let flow = sample_packet_flow_owned();
        let cloned = flow.clone();

        assert_eq!(flow, cloned);
    }

    #[test]
    fn test_packet_flow_owned_hash_stable_for_equal_values() {
        let flow1 = sample_packet_flow_owned();
        let flow2 = sample_packet_flow_owned();

        assert_eq!(flow1, flow2);
        assert_eq!(hash_of(&flow1), hash_of(&flow2));
    }

    #[test]
    fn test_data_link_owned_hash_stable_for_equal_values() {
        let dl1 = sample_data_link_without_vlan();
        let dl2 = sample_data_link_without_vlan();

        assert_eq!(dl1, dl2);
        assert_eq!(hash_of(&dl1), hash_of(&dl2));
    }

    #[test]
    fn test_packet_flow_owned_serialize() {
        let flow = sample_packet_flow_owned();
        let json = serde_json::to_string(&flow).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["data_link"]["link_type"], 1);
        assert_eq!(value["data_link"]["network_protocol"]["kind"], "ipv4");
        assert_eq!(value["data_link"]["link_kind"], "ethernet");
        assert_eq!(
            value["data_link"]["link_details"]["destination_mac"],
            "aa:bb:cc:dd:ee:ff"
        );
        assert_eq!(
            value["data_link"]["link_details"]["source_mac"],
            "11:22:33:44:55:66"
        );
        assert_eq!(value["data_link"]["link_details"]["ethertype"], "IPv4");
        assert!(value["data_link"]["link_details"].get("vlan").is_none());
        assert!(json.contains("\"source_ip\":\"192.168.1.10\""));
        assert!(json.contains("\"destination_ip\":\"8.8.8.8\""));
        assert!(json.contains("\"source_port\":12345"));
        assert!(json.contains("\"destination_port\":80"));
        assert_eq!(value["protocol_internet"], "TCP");
        assert_eq!(value["protocol_transport"], "TCP");
        assert_eq!(value["application_protocol"], "HTTP");
        assert!(value.get("protocol").is_none());
    }

    #[test]
    fn test_internet_owned_serialize_none_fields() {
        let internet = InternetOwned {
            source_ip: None,
            ip_source_type: None,
            destination_ip: None,
            ip_destination_type: None,
            protocol: "UDP".to_string(),
        };

        let json = serde_json::to_string(&internet).unwrap();

        assert!(json.contains("\"source_ip\":null"));
        assert!(json.contains("\"destination_ip\":null"));
        assert!(json.contains("\"protocol_internet\":\"UDP\""));
        assert!(!json.contains("\"protocol\":\"UDP\""));
    }

    /// A consumer rebuilding an SLL link layer from serialized fields (e.g.
    /// a flow-matrix reimport) must obtain the exact same owned value as the
    /// parser produces from the wire — equality and hash included.
    #[test]
    fn constructed_owned_sll_equals_the_parsed_one() {
        // LINKTYPE_LINUX_SLL frame: outgoing, ARPHRD_ETHER, 6-byte address,
        // IPv4 protocol, followed by a minimal IPv4 header.
        let mut frame = Vec::new();
        frame.extend_from_slice(&4_u16.to_be_bytes()); // packet type OUTGOING
        frame.extend_from_slice(&1_u16.to_be_bytes()); // ARPHRD_ETHER
        frame.extend_from_slice(&6_u16.to_be_bytes()); // address length
        frame.extend_from_slice(&[0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4, 0, 0]); // address slot
        frame.extend_from_slice(&0x0800_u16.to_be_bytes()); // protocol IPv4
        frame.extend_from_slice(&[
            0x45, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x40, 0x11, 0x00, 0x00, 192, 168, 1,
            181, 192, 168, 1, 254,
        ]);

        let flow = crate::parse::parse(LinkType::LINUX_SLL, &frame).unwrap();
        let parsed = LinkLayerOwned::from(&flow.data_link);

        let rebuilt = LinkLayerOwned::linux_sll(LinuxSllLinkOwned::new(
            LinuxCookedPacketType::OUTGOING,
            LinuxArphrdType::ETHERNET,
            6,
            Some(vec![0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4]),
            0x0800,
        ));

        assert_eq!(rebuilt, parsed);
        assert_eq!(hash_of(&rebuilt), hash_of(&parsed));
        assert_eq!(rebuilt.link_type(), LinkType::LINUX_SLL);
        assert_eq!(rebuilt.network_protocol(), NetworkProtocol::Ipv4);
    }

    /// SLL2 counterpart of `constructed_owned_sll_equals_the_parsed_one`.
    #[test]
    fn constructed_owned_sll2_equals_the_parsed_one() {
        // LINKTYPE_LINUX_SLL2 frame: IPv4 protocol, interface 3, incoming
        // unicast, ARPHRD_ETHER, 6-byte address, minimal IPv4 header.
        let mut frame = Vec::new();
        frame.extend_from_slice(&0x0800_u16.to_be_bytes()); // protocol IPv4
        frame.extend_from_slice(&0_u16.to_be_bytes()); // reserved (MBZ)
        frame.extend_from_slice(&3_u32.to_be_bytes()); // interface index
        frame.extend_from_slice(&1_u16.to_be_bytes()); // ARPHRD_ETHER
        frame.push(0); // packet type HOST
        frame.push(6); // address length
        frame.extend_from_slice(&[0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4, 0, 0]); // address slot
        frame.extend_from_slice(&[
            0x45, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x40, 0x11, 0x00, 0x00, 192, 168, 1,
            181, 192, 168, 1, 254,
        ]);

        let flow = crate::parse::parse(LinkType::LINUX_SLL2, &frame).unwrap();
        let parsed = LinkLayerOwned::from(&flow.data_link);

        let rebuilt = LinkLayerOwned::linux_sll2(LinuxSll2LinkOwned::new(
            0x0800,
            0,
            3,
            LinuxArphrdType::ETHERNET,
            LinuxCookedPacketType::HOST,
            6,
            Some(vec![0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4]),
        ));

        assert_eq!(rebuilt, parsed);
        assert_eq!(hash_of(&rebuilt), hash_of(&parsed));
        assert_eq!(rebuilt.link_type(), LinkType::LINUX_SLL2);
        assert_eq!(rebuilt.network_protocol(), NetworkProtocol::Ipv4);
    }
}
