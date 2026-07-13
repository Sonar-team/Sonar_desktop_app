// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::Serialize;
use std::{
    fmt,
    hash::{Hash, Hasher},
};

use super::data_link::{DataLink, ethertype, ethertype::Ethertype, mac_addres::MacAddress};
use crate::LinkType;

/// Protocol carried immediately after the link-layer header.
///
/// This is deliberately independent from Ethernet: RAW IP can announce IPv4
/// or IPv6 without fabricating an EtherType, while Ethernet and Linux cooked
/// captures can map their protocol field to the same semantic value.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum NetworkProtocol {
    Ipv4,
    Ipv6,
    Arp,
    Profinet,
    Other(u16),
}

impl NetworkProtocol {
    pub(crate) const fn from_link_protocol(protocol: u16) -> Self {
        match protocol {
            0x0800 => Self::Ipv4,
            0x86dd => Self::Ipv6,
            0x0806 => Self::Arp,
            0x8892 => Self::Profinet,
            other => Self::Other(other),
        }
    }
}

impl fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4 => f.write_str("IPv4"),
            Self::Ipv6 => f.write_str("IPv6"),
            Self::Arp => f.write_str("ARP"),
            Self::Profinet => f.write_str("Profinet"),
            Self::Other(value) => write!(f, "0x{value:04X}"),
        }
    }
}

/// Open numeric packet-direction value carried by Linux cooked captures.
///
/// SLL v1 stores this value in 16 bits; SLL2 stores one byte, widened into
/// this shared representation. Values from either format remain numeric.
///
/// Values beyond the five historical libpcap constants are deliberately
/// preserved so newer kernels do not become fatal parsing errors.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct LinuxCookedPacketType(pub u16);

impl LinuxCookedPacketType {
    pub const HOST: Self = Self(0);
    pub const BROADCAST: Self = Self(1);
    pub const MULTICAST: Self = Self(2);
    pub const OTHER_HOST: Self = Self(3);
    pub const OUTGOING: Self = Self(4);

    pub const fn name(self) -> Option<&'static str> {
        match self {
            Self::HOST => Some("Host"),
            Self::BROADCAST => Some("Broadcast"),
            Self::MULTICAST => Some("Multicast"),
            Self::OTHER_HOST => Some("Other host"),
            Self::OUTGOING => Some("Outgoing"),
            _ => None,
        }
    }
}

impl fmt::Display for LinuxCookedPacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "{name} ({})", self.0),
            None => write!(f, "Unknown ({})", self.0),
        }
    }
}

/// Open numeric ARPHRD hardware type carried by Linux cooked captures.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct LinuxArphrdType(pub u16);

impl LinuxArphrdType {
    pub const ETHERNET: Self = Self(1);
    pub const LOOPBACK: Self = Self(772);
}

impl fmt::Display for LinuxArphrdType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ETHERNET => write!(f, "Ethernet ({})", self.0),
            Self::LOOPBACK => write!(f, "Loopback ({})", self.0),
            _ => write!(f, "ARPHRD {}", self.0),
        }
    }
}

/// LINKTYPE_LINUX_SLL (v1) metadata decoded from its 16-byte cooked header.
///
/// `source_address` contains the bytes available in the fixed eight-byte wire
/// slot, limited by `address_length`. If the declared length is greater than
/// eight, the declaration is preserved and [`Self::address_is_truncated`]
/// reports that only the first eight bytes can exist in this format.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Eq)]
pub struct LinuxSllLink<'a> {
    pub packet_type: LinuxCookedPacketType,
    pub hardware_type: LinuxArphrdType,
    pub address_length: u16,
    pub source_address: Option<&'a [u8]>,
    pub protocol: u16,
    #[serde(skip_serializing)]
    pub payload: &'a [u8],
}

impl<'a> LinuxSllLink<'a> {
    pub(crate) const fn new(
        packet_type: LinuxCookedPacketType,
        hardware_type: LinuxArphrdType,
        address_length: u16,
        source_address: Option<&'a [u8]>,
        protocol: u16,
        payload: &'a [u8],
    ) -> Self {
        Self {
            packet_type,
            hardware_type,
            address_length,
            source_address,
            protocol,
            payload,
        }
    }

    pub const fn address_is_truncated(&self) -> bool {
        self.address_length > 8
    }
}

impl PartialEq for LinuxSllLink<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.packet_type == other.packet_type
            && self.hardware_type == other.hardware_type
            && self.address_length == other.address_length
            && self.source_address == other.source_address
            && self.protocol == other.protocol
    }
}

impl Hash for LinuxSllLink<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.packet_type.hash(state);
        self.hardware_type.hash(state);
        self.address_length.hash(state);
        self.source_address.hash(state);
        self.protocol.hash(state);
    }
}

/// LINKTYPE_LINUX_SLL2 metadata decoded from its 20-byte cooked header.
///
/// `reserved_mbz` is preserved even when non-zero so capture consumers can
/// account for non-conforming input without losing the packet. Interface
/// indices remain numeric because resolving a capture-machine interface name
/// is outside this crate's packet-level contract.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Eq)]
pub struct LinuxSll2Link<'a> {
    pub protocol: u16,
    pub reserved_mbz: u16,
    pub interface_index: u32,
    pub hardware_type: LinuxArphrdType,
    pub packet_type: LinuxCookedPacketType,
    pub address_length: u8,
    pub source_address: Option<&'a [u8]>,
    #[serde(skip_serializing)]
    pub payload: &'a [u8],
}

impl LinuxSll2Link<'_> {
    /// Whether the field marked "must be zero" by the SLL2 format is valid.
    pub const fn reserved_is_zero(&self) -> bool {
        self.reserved_mbz == 0
    }

    /// Whether the declared address is longer than the eight-byte wire slot.
    pub const fn address_is_truncated(&self) -> bool {
        self.address_length > 8
    }
}

impl PartialEq for LinuxSll2Link<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.protocol == other.protocol
            && self.reserved_mbz == other.reserved_mbz
            && self.interface_index == other.interface_index
            && self.hardware_type == other.hardware_type
            && self.packet_type == other.packet_type
            && self.address_length == other.address_length
            && self.source_address == other.source_address
    }
}

impl Hash for LinuxSll2Link<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.protocol.hash(state);
        self.reserved_mbz.hash(state);
        self.interface_index.hash(state);
        self.hardware_type.hash(state);
        self.packet_type.hash(state);
        self.address_length.hash(state);
        self.source_address.hash(state);
    }
}

/// LINKTYPE_RAW carries an IP packet directly and has no link-layer header.
///
/// The version nibble is the only format-specific metadata. No MAC address or
/// EtherType exists on the wire.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Eq)]
pub struct RawIpLink<'a> {
    pub ip_version: u8,
    #[serde(skip_serializing)]
    pub payload: &'a [u8],
}

impl<'a> RawIpLink<'a> {
    const fn new(ip_version: u8, payload: &'a [u8]) -> Self {
        Self {
            ip_version,
            payload,
        }
    }
}

impl PartialEq for RawIpLink<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.ip_version == other.ip_version
    }
}

impl Hash for RawIpLink<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip_version.hash(state);
    }
}

/// Effective addresses and LLC/SNAP payload of a decoded IEEE 802.11 frame.
///
/// The addresses have already been resolved according to the ToDS/FromDS bits.
/// `snap_protocol` is the real protocol value carried by the LLC/SNAP header;
/// no Ethernet frame is fabricated.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Eq)]
pub struct Ieee80211Link<'a> {
    pub destination_mac: MacAddress,
    pub source_mac: MacAddress,
    #[serde(serialize_with = "ethertype::serialize_name")]
    pub snap_protocol: Ethertype,
    #[serde(skip_serializing)]
    pub payload: &'a [u8],
}

impl<'a> Ieee80211Link<'a> {
    /// Builds the semantic IEEE 802.11 view exposed after LLC/SNAP decoding.
    pub const fn new(
        destination_mac: MacAddress,
        source_mac: MacAddress,
        snap_protocol: Ethertype,
        payload: &'a [u8],
    ) -> Self {
        Self {
            destination_mac,
            source_mac,
            snap_protocol,
            payload,
        }
    }
}

impl PartialEq for Ieee80211Link<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.destination_mac == other.destination_mac
            && self.source_mac == other.source_mac
            && self.snap_protocol == other.snap_protocol
    }
}

impl Hash for Ieee80211Link<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.destination_mac.hash(state);
        self.source_mac.hash(state);
        self.snap_protocol.hash(state);
    }
}

impl From<Ethertype> for NetworkProtocol {
    fn from(ethertype: Ethertype) -> Self {
        Self::from_link_protocol(ethertype.0)
    }
}

/// Format-specific link-layer information.
///
/// New variants can be added without changing the common L3/L4/L7 pipeline.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "link_kind", content = "link_details", rename_all = "snake_case")]
pub enum LinkLayerKind<'a> {
    Ethernet(DataLink<'a>),
    RawIp(RawIpLink<'a>),
    LinuxSll(LinuxSllLink<'a>),
    LinuxSll2(LinuxSll2Link<'a>),
    Ieee80211(Ieee80211Link<'a>),
}

/// Parsed link layer together with its canonical LINKTYPE.
///
/// Construction goes through format-specific constructors so `link_type`,
/// `network_protocol` and `kind` cannot disagree.
#[derive(Debug, Clone, Serialize)]
pub struct LinkLayer<'a> {
    link_type: LinkType,
    network_protocol: NetworkProtocol,
    #[serde(skip_serializing)]
    network_payload: &'a [u8],
    #[serde(flatten)]
    kind: LinkLayerKind<'a>,
}

impl PartialEq for LinkLayer<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.link_type == other.link_type
            && self.network_protocol == other.network_protocol
            && self.kind == other.kind
    }
}

impl Eq for LinkLayer<'_> {}

impl Hash for LinkLayer<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.link_type.hash(state);
        self.network_protocol.hash(state);
        self.kind.hash(state);
    }
}

impl<'a> LinkLayer<'a> {
    /// Wraps an Ethernet II / 802.1Q frame in the generic link-layer model.
    pub fn ethernet(frame: DataLink<'a>) -> Self {
        Self {
            link_type: LinkType::ETHERNET,
            network_protocol: frame.ethertype.into(),
            network_payload: frame.payload,
            kind: LinkLayerKind::Ethernet(frame),
        }
    }

    fn raw_ip(network_protocol: NetworkProtocol, ip_version: u8, payload: &'a [u8]) -> Self {
        Self {
            link_type: LinkType::RAW,
            network_protocol,
            network_payload: payload,
            kind: LinkLayerKind::RawIp(RawIpLink::new(ip_version, payload)),
        }
    }

    /// Wraps a decoder-validated LINKTYPE_RAW IPv4 packet.
    pub(crate) fn raw_ipv4(payload: &'a [u8]) -> Self {
        Self::raw_ip(NetworkProtocol::Ipv4, 4, payload)
    }

    /// Wraps a decoder-validated LINKTYPE_RAW IPv6 packet.
    pub(crate) fn raw_ipv6(payload: &'a [u8]) -> Self {
        Self::raw_ip(NetworkProtocol::Ipv6, 6, payload)
    }

    /// Wraps a decoder-validated LINKTYPE_LINUX_SLL v1 header.
    pub(crate) fn linux_sll(frame: LinuxSllLink<'a>) -> Self {
        Self {
            link_type: LinkType::LINUX_SLL,
            network_protocol: NetworkProtocol::from_link_protocol(frame.protocol),
            network_payload: frame.payload,
            kind: LinkLayerKind::LinuxSll(frame),
        }
    }

    /// Wraps a decoder-validated LINKTYPE_LINUX_SLL2 header.
    pub(crate) fn linux_sll2(frame: LinuxSll2Link<'a>) -> Self {
        Self {
            link_type: LinkType::LINUX_SLL2,
            network_protocol: NetworkProtocol::from_link_protocol(frame.protocol),
            network_payload: frame.payload,
            kind: LinkLayerKind::LinuxSll2(frame),
        }
    }

    /// Wraps a decoded native IEEE 802.11 frame without inventing Ethernet.
    pub fn ieee80211(frame: Ieee80211Link<'a>) -> Self {
        Self {
            link_type: LinkType::IEEE802_11,
            network_protocol: frame.snap_protocol.into(),
            network_payload: frame.payload,
            kind: LinkLayerKind::Ieee80211(frame),
        }
    }

    /// Canonical LINKTYPE used to decode this packet.
    pub const fn link_type(&self) -> LinkType {
        self.link_type
    }

    /// Protocol carried by the link-layer payload.
    pub const fn network_protocol(&self) -> NetworkProtocol {
        self.network_protocol
    }

    /// Format-specific link-layer fields.
    pub const fn kind(&self) -> &LinkLayerKind<'a> {
        &self.kind
    }

    /// Ethernet view when this packet was decoded as LINKTYPE_ETHERNET.
    pub const fn as_ethernet(&self) -> Option<&DataLink<'a>> {
        match &self.kind {
            LinkLayerKind::Ethernet(frame) => Some(frame),
            LinkLayerKind::RawIp(_) => None,
            LinkLayerKind::LinuxSll(_) => None,
            LinkLayerKind::LinuxSll2(_) => None,
            LinkLayerKind::Ieee80211(_) => None,
        }
    }

    /// RAW IP details when this packet was decoded as LINKTYPE_RAW.
    pub const fn as_raw_ip(&self) -> Option<&RawIpLink<'a>> {
        match &self.kind {
            LinkLayerKind::RawIp(details) => Some(details),
            LinkLayerKind::Ethernet(_)
            | LinkLayerKind::LinuxSll(_)
            | LinkLayerKind::LinuxSll2(_)
            | LinkLayerKind::Ieee80211(_) => None,
        }
    }

    /// Linux cooked capture v1 details when decoded as LINKTYPE_LINUX_SLL.
    pub const fn as_linux_sll(&self) -> Option<&LinuxSllLink<'a>> {
        match &self.kind {
            LinkLayerKind::LinuxSll(details) => Some(details),
            LinkLayerKind::Ethernet(_)
            | LinkLayerKind::RawIp(_)
            | LinkLayerKind::LinuxSll2(_)
            | LinkLayerKind::Ieee80211(_) => None,
        }
    }

    /// Linux cooked capture v2 details when decoded as LINKTYPE_LINUX_SLL2.
    pub const fn as_linux_sll2(&self) -> Option<&LinuxSll2Link<'a>> {
        match &self.kind {
            LinkLayerKind::LinuxSll2(details) => Some(details),
            LinkLayerKind::Ethernet(_)
            | LinkLayerKind::RawIp(_)
            | LinkLayerKind::LinuxSll(_)
            | LinkLayerKind::Ieee80211(_) => None,
        }
    }

    /// IEEE 802.11 view when the link layer was decoded from a wireless frame.
    pub const fn as_ieee80211(&self) -> Option<&Ieee80211Link<'a>> {
        match &self.kind {
            LinkLayerKind::Ethernet(_) => None,
            LinkLayerKind::RawIp(_) => None,
            LinkLayerKind::LinuxSll(_) => None,
            LinkLayerKind::LinuxSll2(_) => None,
            LinkLayerKind::Ieee80211(frame) => Some(frame),
        }
    }

    /// Zero-copy payload passed to the network-layer parser.
    pub const fn network_payload(&self) -> &'a [u8] {
        self.network_payload
    }
}

impl<'a> From<DataLink<'a>> for LinkLayer<'a> {
    fn from(frame: DataLink<'a>) -> Self {
        Self::ethernet(frame)
    }
}

impl fmt::Display for LinkLayer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            LinkLayerKind::Ethernet(frame) => frame.fmt(f),
            LinkLayerKind::RawIp(details) => write!(
                f,
                "\n    RAW IP Version: {},\n    Protocol: {}\n",
                details.ip_version, self.network_protocol
            ),
            LinkLayerKind::LinuxSll(details) => write!(
                f,
                "\n    Linux SLL Packet Type: {},\n    Hardware Type: {},\n    Address Length: {},\n    Protocol: 0x{:04X}\n",
                details.packet_type,
                details.hardware_type,
                details.address_length,
                details.protocol
            ),
            LinkLayerKind::LinuxSll2(details) => write!(
                f,
                "\n    Linux SLL2 Interface Index: {},\n    Packet Type: {},\n    Hardware Type: {},\n    Address Length: {},\n    Protocol: 0x{:04X},\n    Reserved MBZ: 0x{:04X}\n",
                details.interface_index,
                details.packet_type,
                details.hardware_type,
                details.address_length,
                details.protocol,
                details.reserved_mbz
            ),
            LinkLayerKind::Ieee80211(frame) => write!(
                f,
                "\n    IEEE 802.11 Destination MAC: {},\n    Source MAC: {},\n    SNAP Protocol: {},\n    Payload Length: {}\n",
                frame.destination_mac,
                frame.source_mac,
                frame.snap_protocol.name(),
                frame.payload.len()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::data_link::mac_addres::MacAddress;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn ethernet<'a>(payload: &'a [u8]) -> LinkLayer<'a> {
        LinkLayer::ethernet(DataLink {
            destination_mac: MacAddress([0, 1, 2, 3, 4, 5]),
            source_mac: MacAddress([6, 7, 8, 9, 10, 11]),
            vlan: None,
            ethertype: Ethertype(0x0800),
            payload,
        })
    }

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn ethernet_exposes_common_and_specific_views() {
        let payload = [0x45, 0, 0, 20];
        let layer = ethernet(&payload);

        assert_eq!(layer.link_type(), LinkType::ETHERNET);
        assert_eq!(layer.network_protocol(), NetworkProtocol::Ipv4);
        assert_eq!(layer.network_payload().as_ptr(), payload.as_ptr());
        assert_eq!(layer.as_ethernet().unwrap().ethertype, Ethertype(0x0800));
    }

    #[test]
    fn serialization_is_explicitly_tagged() {
        let payload = [0x45, 0, 0, 20];
        let value = serde_json::to_value(ethernet(&payload)).unwrap();

        assert_eq!(value["link_type"], 1);
        assert_eq!(value["network_protocol"]["kind"], "ipv4");
        assert_eq!(value["link_kind"], "ethernet");
        assert_eq!(value["link_details"]["ethertype"], "IPv4");
        assert!(value["link_details"].get("payload").is_none());
    }

    #[test]
    fn equality_and_hash_ignore_network_payload_bytes() {
        let first = ethernet(&[1, 2, 3]);
        let second = ethernet(&[9, 8, 7, 6]);

        assert_eq!(first, second);
        assert_eq!(hash_of(&first), hash_of(&second));
    }

    #[test]
    fn raw_ip_details_are_zero_copy_and_ignore_payload_in_flow_identity() {
        let first_bytes = [0x45, 1, 2];
        let second_bytes = [0x45, 9, 8, 7];
        let first = LinkLayer::raw_ipv4(&first_bytes);
        let second = LinkLayer::raw_ipv4(&second_bytes);
        let raw = first.as_raw_ip().unwrap();

        assert_eq!(first.link_type(), LinkType::RAW);
        assert_eq!(first.network_protocol(), NetworkProtocol::Ipv4);
        assert_eq!(raw.ip_version, 4);
        assert_eq!(raw.payload.as_ptr(), first_bytes.as_ptr());
        assert_eq!(first.network_payload().as_ptr(), first_bytes.as_ptr());
        assert_eq!(first, second);
        assert_eq!(hash_of(&first), hash_of(&second));
        assert_ne!(first, LinkLayer::raw_ipv6(&first_bytes));
    }

    #[test]
    fn raw_ip_serialization_has_no_fabricated_ethernet_fields() {
        let value = serde_json::to_value(LinkLayer::raw_ipv4(&[0x45])).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "link_type": 101,
                "network_protocol": { "kind": "ipv4" },
                "link_kind": "raw_ip",
                "link_details": { "ip_version": 4 }
            })
        );
    }

    #[test]
    fn linux_sll_flow_identity_uses_metadata_but_ignores_payload() {
        let first_address = [1, 2, 3, 4];
        let second_address = [1, 2, 3, 4];
        let first_payload = [0x45, 1, 2];
        let second_payload = [0x45, 9, 8, 7];
        let first = LinkLayer::linux_sll(LinuxSllLink::new(
            LinuxCookedPacketType::HOST,
            LinuxArphrdType::ETHERNET,
            4,
            Some(&first_address),
            0x0800,
            &first_payload,
        ));
        let second = LinkLayer::linux_sll(LinuxSllLink::new(
            LinuxCookedPacketType::HOST,
            LinuxArphrdType::ETHERNET,
            4,
            Some(&second_address),
            0x0800,
            &second_payload,
        ));

        assert_eq!(first, second);
        assert_eq!(hash_of(&first), hash_of(&second));

        let different_direction = LinkLayer::linux_sll(LinuxSllLink::new(
            LinuxCookedPacketType::OUTGOING,
            LinuxArphrdType::ETHERNET,
            4,
            Some(&second_address),
            0x0800,
            &second_payload,
        ));
        assert_ne!(first, different_direction);
    }

    #[test]
    fn linux_sll2_flow_identity_keeps_interface_but_ignores_payload() {
        let first_address = [1, 2, 3, 4];
        let second_address = [1, 2, 3, 4];
        let first_payload = [0x45, 1, 2];
        let second_payload = [0x45, 9, 8, 7];
        let first = LinkLayer::linux_sll2(LinuxSll2Link {
            protocol: 0x0800,
            reserved_mbz: 0,
            interface_index: 7,
            hardware_type: LinuxArphrdType::ETHERNET,
            packet_type: LinuxCookedPacketType::HOST,
            address_length: 4,
            source_address: Some(&first_address),
            payload: &first_payload,
        });
        let second = LinkLayer::linux_sll2(LinuxSll2Link {
            protocol: 0x0800,
            reserved_mbz: 0,
            interface_index: 7,
            hardware_type: LinuxArphrdType::ETHERNET,
            packet_type: LinuxCookedPacketType::HOST,
            address_length: 4,
            source_address: Some(&second_address),
            payload: &second_payload,
        });

        assert_eq!(first, second);
        assert_eq!(hash_of(&first), hash_of(&second));

        let different_interface = LinkLayer::linux_sll2(LinuxSll2Link {
            protocol: 0x0800,
            reserved_mbz: 0,
            interface_index: 8,
            hardware_type: LinuxArphrdType::ETHERNET,
            packet_type: LinuxCookedPacketType::HOST,
            address_length: 4,
            source_address: Some(&second_address),
            payload: &second_payload,
        });
        assert_ne!(first, different_interface);
    }

    #[test]
    fn unknown_protocol_keeps_its_wire_value_in_json() {
        let layer = LinkLayer::ethernet(DataLink {
            destination_mac: MacAddress([0, 1, 2, 3, 4, 5]),
            source_mac: MacAddress([6, 7, 8, 9, 10, 11]),
            vlan: None,
            ethertype: Ethertype(0xabcd),
            payload: &[],
        });
        let value = serde_json::to_value(layer).unwrap();

        assert_eq!(value["network_protocol"]["kind"], "other");
        assert_eq!(value["network_protocol"]["value"], 0xabcd);
        assert_eq!(value["link_details"]["ethertype"], "Unknown (0xABCD)");
    }
}
