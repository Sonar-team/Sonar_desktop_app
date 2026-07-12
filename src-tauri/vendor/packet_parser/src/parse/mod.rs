// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! PacketFlow – Unified network packet parsing abstraction
//!
//! This module provides the [`PacketFlow`] structure, which represents a
//! fully parsed network packet across multiple layers:
//!
//! - Data Link (L2)
//! - Internet (L3)
//! - Transport (L4)
//! - Application (L7, best-effort)
//!
//! The parsing model is **layered and progressive**: each layer is parsed
//! from the payload of the previous one. Unsupported protocols do **not**
//! cause a hard failure and are represented as `None`, allowing partial
//! decoding of real-world traffic.
//!
//! ## Design goals
//!
//! - Deterministic, zero-copy parsing of the L2/L3/L4 layers using `&[u8]`
//!   references. Some application-layer parsers (e.g. DNS, HTTP, SNMP) and
//!   tunnel recursion do allocate (`Vec`, `Box`).
//! - Clear separation between protocol layers
//! - Robust handling of unknown or unsupported protocols
//! - Suitable for network auditing, traffic analysis and post-capture inspection
//!
//! This module does **not** perform stream reassembly or session tracking.
//! It expects a complete packet buffer (e.g. from PCAP capture).

use application::Application;
use application::protocols::ams::AmsPacket;
use application::protocols::copt::CotpHeader;
use application::protocols::dhcpv6::Dhcpv6Packet;
use application::protocols::postgresql::is_likely_postgresql_payload;
use application::protocols::snmp::SnmpPacket;
use internet::Internet;
use serde::Serialize;
use transport::Transport;
use transport::protocols::TransportProtocol;

use crate::{
    DataLink,
    errors::{ParsedPacketError, internet::InternetError, transport::TransportError},
    owned::PacketFlowOwned,
};

pub mod application;
pub mod data_link;
pub mod internet;
pub mod transport;
pub(crate) mod tunnel;

/// Layer at which recognized-but-invalid bytes stopped the parsing.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub enum CorruptedLayerKind {
    /// The EtherType announced a known L3 protocol but its bytes are invalid.
    Internet,
    /// The internet layer announced a known L4 protocol but its bytes are
    /// invalid.
    Transport,
}

/// A layer that was **recognized** (by the EtherType or the IP protocol
/// field) but whose bytes are invalid.
///
/// Parsing degrades gracefully instead of failing: every layer *above* the
/// corrupted one stays filled, the corrupted layer and everything below it
/// are `None`, and the corruption is reported here.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct CorruptedLayer {
    /// Which layer had invalid bytes.
    pub layer: CorruptedLayerKind,
    /// Human-readable description of the parse error.
    pub error: String,
}

/// A fully or partially parsed network packet flow.
///
/// `PacketFlow` represents a packet parsed across protocol layers.
/// Each layer is optional except for the data link layer, which is mandatory.
///
/// Unsupported or unrecognized protocols do **not** fail parsing and instead
/// result in `None` for the corresponding layer. A **recognized but corrupt**
/// layer does not fail parsing either: the layers above it are kept and the
/// problem is reported in [`PacketFlow::corrupted`]. Parsing only returns an
/// error when the data-link layer itself cannot be read.
///
/// The structure borrows from the original packet buffer (`&[u8]`) and is
/// therefore zero-copy.
///
/// ## Equality and hashing
///
/// `PartialEq`, `Eq` and `Hash` compare the **flow identity** (addresses,
/// protocols, ports…), not the raw bytes: layer payloads are deliberately
/// ignored. Two packets of the same conversation carrying different data
/// therefore compare equal and hash identically.
#[derive(Debug, Clone, Serialize, Eq)]
pub struct PacketFlow<'a> {
    /// Data link layer (mandatory).
    #[serde(flatten)]
    pub data_link: DataLink<'a>,

    /// Internet layer (optional).
    #[serde(flatten)]
    pub internet: Option<Internet<'a>>,

    /// Transport layer (optional).
    #[serde(flatten)]
    pub transport: Option<Transport<'a>>,

    /// Application layer (optional, best-effort).
    #[serde(flatten)]
    pub application: Option<Application>,

    /// Encapsulated packet (optional). When this flow is a tunnel (e.g. CAPWAP,
    /// carried as the application protocol), `inner` holds the packet parsed
    /// from inside the tunnel — recursively, from the outermost to the
    /// innermost. See [`PacketFlow::flatten`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner: Option<Box<PacketFlow<'a>>>,

    /// Present when a recognized layer carried invalid bytes: that layer and
    /// the ones below are `None`, the layers above stay filled. `None` on
    /// healthy packets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrupted: Option<CorruptedLayer>,
}

impl<'a> TryFrom<&'a [u8]> for PacketFlow<'a> {
    type Error = ParsedPacketError;

    #[inline(always)]
    fn try_from(packets: &'a [u8]) -> Result<Self, Self::Error> {
        Self::parse_impl(packets)
    }
}

impl<'a> PartialEq for PacketFlow<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data_link == other.data_link
            && self.internet == other.internet
            && self.transport == other.transport
            && self.application == other.application
            && self.inner == other.inner
            && self.corrupted == other.corrupted
    }
}

use std::hash::{Hash, Hasher};

impl<'a> Hash for PacketFlow<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_link.hash(state);
        self.internet.hash(state);
        self.transport.hash(state);
        self.application.hash(state);
        self.inner.hash(state);
        self.corrupted.hash(state);
    }
}

impl<'a> PacketFlow<'a> {
    fn parse_application_from_transport(transport: &Transport<'a>) -> Option<Application> {
        let payload = transport.payload?;
        if payload.is_empty() {
            return None;
        }

        if (is_snmp_udp_port(transport.source_port) || is_snmp_udp_port(transport.destination_port))
            && SnmpPacket::try_from(payload).is_ok()
        {
            return Some(Application {
                application_protocol: "SNMP",
            });
        }

        // Protocoles à signature faible : uniquement détectés sur leurs ports
        // standards pour éviter les faux positifs du probing à l'aveugle.
        if transport.protocol == TransportProtocol::Udp
            && (is_dhcpv6_udp_port(transport.source_port)
                || is_dhcpv6_udp_port(transport.destination_port))
            && Dhcpv6Packet::try_from(payload).is_ok()
        {
            return Some(Application {
                application_protocol: "DHCPv6",
            });
        }

        if transport.protocol == TransportProtocol::Tcp
            && (is_iso_tsap_tcp_port(transport.source_port)
                || is_iso_tsap_tcp_port(transport.destination_port))
            && CotpHeader::try_from(payload).is_ok()
        {
            return Some(Application {
                application_protocol: "COTP",
            });
        }

        if (is_ams_port(transport.source_port) || is_ams_port(transport.destination_port))
            && AmsPacket::try_from(payload).is_ok()
        {
            return Some(Application {
                application_protocol: "AMS",
            });
        }

        if transport.protocol == TransportProtocol::Tcp && is_likely_postgresql_payload(payload) {
            return Some(Application {
                application_protocol: "PostgreSQL",
            });
        }

        let parsed = Application::try_from(payload).ok();
        if matches!(
            parsed.as_ref().map(|app| app.application_protocol),
            Some("OPC UA")
        ) {
            return parsed;
        }

        if is_opcua_tcp_port(transport.source_port) || is_opcua_tcp_port(transport.destination_port)
        {
            return Some(Application {
                application_protocol: "OPC UA",
            });
        }

        parsed
    }

    /// Converts this borrowed [`PacketFlow`] into an owned version.
    ///
    /// This performs the necessary allocations to detach from the original
    /// packet buffer and is suitable for storage, serialization or cross-thread
    /// usage.
    pub fn to_owned(&self) -> PacketFlowOwned {
        PacketFlowOwned::from(self)
    }

    /// Returns this flow and every encapsulated flow, from the outermost to the
    /// innermost. A non-tunneled packet yields a single entry; a tunneled one
    /// yields several (outer tunnel + inner conversation(s)).
    pub fn flatten(&self) -> Vec<&PacketFlow<'a>> {
        let mut out = Vec::new();
        let mut current = self;
        loop {
            out.push(current);
            match current.inner.as_deref() {
                Some(next) => current = next,
                None => break,
            }
        }
        out
    }

    #[inline(always)]
    fn parse_impl(packets: &'a [u8]) -> Result<Self, ParsedPacketError> {
        let data_link = DataLink::try_from(packets)?;
        Self::parse_layers(data_link, 0)
    }

    /// Parses the internet layer from the data-link layer, dispatching on the
    /// EtherType. An unknown EtherType yields `(None, None)`; a known
    /// EtherType with a corrupt payload yields `(None, Some(corruption))` —
    /// never a hard error, so the data-link information is preserved.
    #[inline(always)]
    fn parse_l3(data_link: &DataLink<'a>) -> (Option<Internet<'a>>, Option<CorruptedLayer>) {
        match Internet::try_from_parts(data_link.ethertype, data_link.payload) {
            Ok(internet) => (Some(internet), None),
            Err(InternetError::UnsupportedProtocol) => (None, None),
            Err(e) => (
                None,
                Some(CorruptedLayer {
                    layer: CorruptedLayerKind::Internet,
                    error: e.to_string(),
                }),
            ),
        }
    }

    #[inline(always)]
    fn parse_l4(
        internet: Option<&Internet<'a>>,
    ) -> (Option<Transport<'a>>, Option<CorruptedLayer>) {
        match internet {
            Some(internet) => {
                match Transport::try_from_parts(internet.payload_protocol, internet.payload) {
                    Ok(transport) => (Some(transport), None),
                    Err(TransportError::UnsupportedProtocol) => (None, None),
                    Err(e) => (
                        None,
                        Some(CorruptedLayer {
                            layer: CorruptedLayerKind::Transport,
                            error: e.to_string(),
                        }),
                    ),
                }
            }
            None => (None, None),
        }
    }

    /// If the transport layer encapsulates a tunnel (e.g. CAPWAP), record the
    /// tunnel name as THIS flow's application protocol and parse the inner
    /// packet into `inner`. Otherwise, best-effort application detection.
    #[inline(always)]
    fn parse_l7_and_inner(
        transport: Option<&Transport<'a>>,
        depth: u8,
    ) -> (Option<Application>, Option<Box<PacketFlow<'a>>>) {
        match transport {
            Some(transport) => match tunnel::detect_inner(transport, depth) {
                Some((tunnel_name, inner_flow)) => (
                    Some(Application {
                        application_protocol: tunnel_name,
                    }),
                    Some(Box::new(inner_flow)),
                ),
                None => (Self::parse_application_from_transport(transport), None),
            },
            None => (None, None),
        }
    }

    /// Parses the L3/L4/L7 layers from a data-link layer, then recurses into any
    /// tunnel. `depth` bounds tunnel nesting. Shared by the top-level entry and
    /// by tunnel peeling (which supplies a rebuilt inner data-link layer).
    pub(crate) fn parse_layers(
        data_link: DataLink<'a>,
        depth: u8,
    ) -> Result<Self, ParsedPacketError> {
        let (internet, l3_corruption) = Self::parse_l3(&data_link);
        let (transport, l4_corruption) = Self::parse_l4(internet.as_ref());
        let (application, inner) = Self::parse_l7_and_inner(transport.as_ref(), depth);

        Ok(PacketFlow {
            data_link,
            internet,
            transport,
            application,
            inner,
            corrupted: l3_corruption.or(l4_corruption),
        })
    }

    // -------------------------------------------------------------------------
    // Timed parsing (feature-gated) — does NOT change PacketFlow API/fields.
    // Uses crate::timing helpers so "feature off" has zero impact elsewhere.
    // -------------------------------------------------------------------------

    #[cfg(feature = "parse_timing")]
    #[inline(always)]
    fn parse_impl_timed(
        packets: &'a [u8],
        timing: &mut crate::timing::ParseTiming,
    ) -> Result<Self, ParsedPacketError> {
        use crate::timing::{elapsed_ns, now};

        let total_t0 = now();

        // Same layer functions as parse_layers(): the timed path returns
        // exactly what the normal path returns (tunnels included) and only
        // adds instrumentation around each layer.
        let result = (|| {
            let t0 = now();
            let data_link = match DataLink::try_from(packets) {
                Ok(data_link) => data_link,
                Err(e) => {
                    timing.l2_ns = elapsed_ns(t0);
                    return Err(e.into());
                }
            };
            timing.l2_ns = elapsed_ns(t0);

            let t0 = now();
            let internet = Self::parse_l3(&data_link);
            timing.l3_ns = elapsed_ns(t0);
            let (internet, l3_corruption) = internet;

            let t0 = now();
            let (transport, l4_corruption) = Self::parse_l4(internet.as_ref());
            timing.l4_ns = elapsed_ns(t0);

            // l7_ns includes tunnel detection and the recursive parsing of
            // any encapsulated packet.
            let t0 = now();
            let (application, inner) = Self::parse_l7_and_inner(transport.as_ref(), 0);
            timing.l7_ns = elapsed_ns(t0);

            Ok(PacketFlow {
                data_link,
                internet,
                transport,
                application,
                inner,
                corrupted: l3_corruption.or(l4_corruption),
            })
        })();

        timing.total_ns = elapsed_ns(total_t0);
        result
    }

    /// Parses a raw packet buffer into a [`PacketFlow`] and fills timing data.
    ///
    /// This is feature-gated (`parse_timing`) and does not affect normal parsing.
    ///
    /// Convention:
    /// - `l*_ns` is the cost of the *attempt* (so it may be >0 even if unsupported).
    #[cfg(feature = "parse_timing")]
    #[inline(always)]
    pub fn try_from_timed(
        packets: &'a [u8],
        timing: &mut crate::timing::ParseTiming,
    ) -> Result<Self, ParsedPacketError> {
        *timing = crate::timing::ParseTiming::default();
        Self::parse_impl_timed(packets, timing)
    }
}

fn is_opcua_tcp_port(port: Option<u16>) -> bool {
    matches!(port, Some(4840 | 12001))
}

fn is_snmp_udp_port(port: Option<u16>) -> bool {
    matches!(port, Some(161 | 162))
}

fn is_dhcpv6_udp_port(port: Option<u16>) -> bool {
    matches!(port, Some(546 | 547))
}

/// ISO-TSAP (TPKT/COTP, notamment S7comm).
fn is_iso_tsap_tcp_port(port: Option<u16>) -> bool {
    matches!(port, Some(102))
}

/// Beckhoff AMS/ADS : TCP 48898, UDP 48899.
fn is_ams_port(port: Option<u16>) -> bool {
    matches!(port, Some(48898 | 48899))
}

#[cfg(test)]
mod tests {
    use crate::parse::application::protocols::postgresql::{
        PostgreSqlMessageBody, PostgreSqlMessageType, PostgreSqlPacket,
    };
    use crate::parse::transport::protocols::TransportProtocol;

    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::net::{IpAddr, Ipv4Addr};

    fn sample_ipv6_tcp_packet() -> Vec<u8> {
        hex::decode("646ee0eafa83feaa818ec86486dd6500000004d8063d2a0014504007080a00000000000020012a04cec011a3a3971842f3918f5dec7d01bbe8007dc6e5a6f851bd128018007a628400000101080abd9295c7f7f13851edd897cab0251b0fcd0ce6976e5ffc596564fa9d1986ad4dcd59e5481c0ffa357590175e992da0a3ec8d32403b4ebb23b181e8916f5aa518eef4e126efe31b847f56868867cf26b0acd92680833ceefa8fb7fe1c3f1d96d1693b677b26d76acc7ff4e0e9ff9f5a6b5176689100891de5596ed15c93ec2d87570b13c73c95881562dcebad7acacf6dee4f8872ab2e07dacc00abf8534f8465b3f70a9362dd466bed097dd3943c49e60254c2d1d11e8a43db7b2bb20fac75bd2d12e61a135fc08fb817cf2779363052d5b8698712a0681510513bcd0d3095af28c63ae243006d44d792faa21d5a866c88e8948074e1bf9969d6bf965a796346553d7b64384ccf6a8ac5203aa1820ed3f46a3b656c5bb4670c6240da14f82c8e27cb1be60439c9aa0f9f58f716194deaf5ca10bba3f71d3beb73d878e0768f8ac20e7d1d984bdcdcd44bf861ae99c12a7307fc4ede845580bb97903da6b640403bdfd317b65b97d279d8b9ca5df881b305cf0ebe82d1aa4fd32fb9463653d11ede2327dbbf82453870017c4b6f69daeca416bbe5a1138c62e0da69dd8568b017b1c6b9def2ad9f5ae04bab9add00ddc790ebda970a5c80d44334c1a0a03ce1428efa2d6c260cf78e6442313fd5eacdf578572ef6ab4df6d6b6d9b889b2be67f0c8ae5d87923ce89df59386b8ecb29aeb1e6c5c5be465e3ad4b62c167443068a268ff6067be0f637f5e9994635c09d73c2bc5c5bc76f8fb2b1f00417ee67792cea34ab05451468de91524dbfe127463824e1d3fcc03fda2ff01ae8d21c242b996bc9b138a0ce211166af40b21a32b0b202aa8587430f03a46e9fe87f5991c132cc9c09ce36757888d913da28261e07e537d66f8c76abbd0cfd60236c880dfe49ad48a8ba9dfefe0efede8b004c7fc86b914fc4f4dd067f8dee3c8ecd89a47eafd438523d8ffd9fd1fa5797cd446fe019f8b5cd4cf0bb6e6800f1c06f04dafbc009b558abeb5821cd5c5a6f9b24fe47606ca098290845ba5ac42aa994844553f7522efa08f99b7e62a858cdd1d7376b552fa2ddd87d4f8945292a31654f4032a9e6dc86584bc882bfd063e439fb701da038b23791a0706a1672bd6d70234ceef5340c975a473f8f524743672a284e22098d525b6ff48c54c0d79fe2d67ea4b5619536ef182fe181def5c640961138ed1e7bbb795475295ca3418b8ab5b594307f7338e5689b2fea6aed83a08c356f4e4d072dad9b5b3e38bd9a4c5a632c5f024e892e85341da285eb2098a7d1d114ba8662e6f5c33513cc0d5d0d0186ae7aadab3334d03a8644c3774a16bd985cc198f48012bbe5d9c952472936e7b06c9e663ddb0cdc0fdbcf07e19d11064fe5f9e6f81d7440981331f2faab3f69466af1cd7d8a28c99f680ed88a24e27e53ae2b6d2323aa7592a0d169094eaf5134d421f66934a21e75a6d6532caa0c2c86697ba0b4c3cc484081ef8c94f2609a8b648527ae6926d72eecba718f51e61ce405f36c25e20978e40d5d9dc76dec606e73d2056c15a69fbe16963a09e1ac0a4fcbf922d747d8f29e708f241f565b5a18832a65ff7e41a7ec7ec8b903d7ce05cf298beac641d1c94d8f8eeb7c3622b84a50dfb8df3db8d121ebda13838104f129150d8e8f07804295d30e59e184c4f4b007e3e62420a4fc8e293144f38f828de4ff74c888589252d1de11bc017fc772a183240f682")
            .expect("invalid test hex fixture")
    }

    fn sample_ipv4_tcp_ethernet_ip_register_session_non_standard_port() -> Vec<u8> {
        hex::decode(
            "00112233445566778899aabb0800450000440001400040060000c0a8000ac0a80014303904d20000000100000000501820000000000065000400000000000000000001020304050607080000000001000000",
        )
        .expect("invalid test hex fixture")
    }

    fn sample_ipv4_tcp_postgresql_startup_frame_7() -> Vec<u8> {
        hex::decode(
            "00000000000000000000000008004500005aff0b400040063d907f0000017f000001b36a1538c95bf221c946b53e80187fff8b6e00000101080a13420d2c13420d29000000260003000075736572006f727978006461746162617365006d61696c73746f72650000",
        )
        .expect("invalid test hex fixture")
    }

    fn sample_ipv4_tcp_postgresql_backend_frame_19() -> Vec<u8> {
        hex::decode(
            "0000000000000000000000000800450000d5ef3a400040064ce67f0000017f0000011538b36bc949e22ac901b10c80187fffda6900000101080a13420d7613420d2c520000000800000000530000001c636c69656e745f656e636f64696e6700554e49434f4445005300000017446174655374796c650049534f2c204d445900530000001569735f737570657275736572006f66660053000000197365727665725f76657273696f6e00372e342e3600530000001f73657373696f6e5f617574686f72697a6174696f6e006f727978004b0000000c000058e9679266075a0000000549",
        )
        .expect("invalid test hex fixture")
    }

    fn ethernet_ipv4_udp_packet(ip_flags_fragment: u16, udp_length: u16) -> Vec<u8> {
        let udp_payload = [0xde, 0xad, 0xbe, 0xef];
        let udp_actual_len = 8 + udp_payload.len();
        let ip_total_len = 20 + udp_actual_len;

        let mut packet = Vec::with_capacity(14 + ip_total_len);
        packet.extend_from_slice(&[
            0x00,
            0x11,
            0x22,
            0x33,
            0x44,
            0x55, // Destination MAC
            0x66,
            0x77,
            0x88,
            0x99,
            0xaa,
            0xbb, // Source MAC
            0x08,
            0x00, // IPv4 EtherType
            0x45, // Version + IHL
            0x00, // DSCP/ECN
            (ip_total_len >> 8) as u8,
            ip_total_len as u8,
            0x12,
            0x34, // Identification
            (ip_flags_fragment >> 8) as u8,
            ip_flags_fragment as u8,
            64, // TTL
            17, // UDP
            0x00,
            0x00, // Header checksum
            192,
            168,
            1,
            10, // Source IP
            192,
            168,
            1,
            20, // Destination IP
            0x30,
            0x39, // UDP source port
            0x00,
            0x35, // UDP destination port
            (udp_length >> 8) as u8,
            udp_length as u8,
            0x00,
            0x00, // UDP checksum
        ]);
        packet.extend_from_slice(&udp_payload);
        packet
    }

    fn postgresql_parse_bind_execute_sync_payload() -> Vec<u8> {
        let mut payload = Vec::new();

        payload.push(b'P');
        payload.extend_from_slice(&81u32.to_be_bytes());
        payload.push(0);
        payload.extend_from_slice(
            b"SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL READ COMMITTED",
        );
        payload.push(0);
        payload.extend_from_slice(&0u16.to_be_bytes());

        payload.push(b'B');
        payload.extend_from_slice(&12u32.to_be_bytes());
        payload.push(0);
        payload.push(0);
        payload.extend_from_slice(&0u16.to_be_bytes());
        payload.extend_from_slice(&0u16.to_be_bytes());
        payload.extend_from_slice(&0u16.to_be_bytes());

        payload.push(b'E');
        payload.extend_from_slice(&9u32.to_be_bytes());
        payload.push(0);
        payload.extend_from_slice(&1u32.to_be_bytes());

        payload.push(b'S');
        payload.extend_from_slice(&4u32.to_be_bytes());

        payload
    }

    fn ethernet_ipv4_tcp_packet(
        source_port: u16,
        destination_port: u16,
        tcp_payload: &[u8],
    ) -> Vec<u8> {
        let tcp_header_len = 20usize;
        let ip_total_len = 20 + tcp_header_len + tcp_payload.len();

        let mut packet = Vec::with_capacity(14 + ip_total_len);
        packet.extend_from_slice(&[
            0x00,
            0x10,
            0x6f,
            0x19,
            0x02,
            0xe4, // Destination MAC
            0x00,
            0x10,
            0x6f,
            0x1a,
            0xe4,
            0x42, // Source MAC
            0x08,
            0x00, // IPv4 EtherType
            0x45, // Version + IHL
            0x00, // DSCP/ECN
            (ip_total_len >> 8) as u8,
            ip_total_len as u8,
            0x34,
            0x18, // Identification
            0x40,
            0x00, // Don't fragment
            64,   // TTL
            6,    // TCP
            0x00,
            0x00, // Header checksum
            172,
            19,
            90,
            10, // Source IP
            172,
            19,
            90,
            2, // Destination IP
        ]);

        packet.extend_from_slice(&source_port.to_be_bytes());
        packet.extend_from_slice(&destination_port.to_be_bytes());
        packet.extend_from_slice(&15526u32.to_be_bytes());
        packet.extend_from_slice(&5876u32.to_be_bytes());
        packet.extend_from_slice(&[
            0x50, 0x18, // Data offset 5, ACK+PSH
            0x20, 0x00, // Window size
            0x00, 0x00, // Checksum
            0x00, 0x00, // Urgent pointer
        ]);
        packet.extend_from_slice(tcp_payload);

        packet
    }

    // fn sample_ipv6_udp_dhcpv6_silicit() -> Vec<u8> {
    //     hex::decode("333300010002080027fe8f9586dd60000000003c1101fe800000000000000a0027fffefe8f95ff02000000000000000000000001000202220223003cad08011008740001000e000100011c39cf88080027fe8f9500060004001700180008000200000019000c27fe8f9500000e1000001518")
    //         .expect("invalid test hex fixture")
    // }

    #[test]
    fn packetflow_try_from_valid_packet_should_succeed() {
        let packet = sample_ipv6_tcp_packet();

        let result = PacketFlow::try_from(packet.as_slice());

        assert!(result.is_ok());
    }

    #[test]
    fn packetflow_try_from_empty_packet_should_fail() {
        let packet: &[u8] = &[];

        let result = PacketFlow::try_from(packet);

        assert!(result.is_err());
    }

    #[test]
    fn packetflow_should_parse_data_link_layer() {
        let packet = sample_ipv6_tcp_packet();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        assert!(!flow.data_link.payload.is_empty());
    }

    #[test]
    fn packetflow_should_parse_internet_layer_for_known_fixture() {
        let packet = sample_ipv6_tcp_packet();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        assert!(flow.internet.is_some());

        let internet = flow.internet.as_ref().unwrap();
        assert_eq!(internet.protocol_name, "IPv6");
        assert!(internet.source.is_some());
        assert!(internet.destination.is_some());
        assert!(internet.payload_protocol.is_some());
        assert!(!internet.payload.is_empty());
    }

    #[test]
    fn packetflow_should_parse_transport_layer_for_known_fixture() {
        let packet = sample_ipv6_tcp_packet();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        assert!(flow.transport.is_some());

        let transport = flow.transport.as_ref().unwrap();
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
    }

    #[test]
    fn packetflow_application_layer_is_best_effort() {
        let packet = sample_ipv6_tcp_packet();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        if let Some(transport) = &flow.transport {
            match transport.payload {
                Some(payload) => {
                    if flow.application.is_some() {
                        assert!(!payload.is_empty());
                    }
                }
                None => {
                    assert!(flow.application.is_none());
                }
            }
        }
    }

    #[test]
    fn packetflow_detects_ethernet_ip_without_standard_port() {
        let packet = sample_ipv4_tcp_ethernet_ip_register_session_non_standard_port();

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.source_port, Some(12345));
        assert_eq!(transport.destination_port, Some(1234));

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "EtherNet/IP");
    }

    #[test]
    fn packetflow_detects_postgresql_from_payload_without_standard_port() {
        let payload = postgresql_parse_bind_execute_sync_payload();
        let packet = ethernet_ipv4_tcp_packet(51845, 15432, &payload);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.source_port, Some(51845));
        assert_eq!(transport.destination_port, Some(15432));
        assert_eq!(transport.payload, Some(payload.as_slice()));

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "PostgreSQL");
    }

    #[test]
    fn packetflow_detects_postgresql_startup_frame_7() {
        let packet = sample_ipv4_tcp_postgresql_startup_frame_7();

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer");
        assert_eq!(internet.protocol_name, "IPv4");
        assert_eq!(
            internet.source,
            Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
        );
        assert_eq!(
            internet.destination,
            Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
        );

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.source_port, Some(45930));
        assert_eq!(transport.destination_port, Some(5432));

        let payload = transport.payload.expect("tcp payload");
        assert_eq!(payload.len(), 38);

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "PostgreSQL");

        let pg = PostgreSqlPacket::try_from(payload).expect("postgresql startup packet");
        assert_eq!(pg.messages.len(), 1);
        assert_eq!(
            pg.messages[0].message_type,
            PostgreSqlMessageType::StartupMessage
        );
        assert_eq!(pg.messages[0].length, 38);

        match &pg.messages[0].body {
            PostgreSqlMessageBody::Startup(startup) => {
                assert_eq!(startup.protocol_version, 196_608);
                assert_eq!(
                    startup.parameters,
                    vec![("user", "oryx"), ("database", "mailstore")]
                );
            }
            other => panic!("expected Startup body, got {other:?}"),
        }
    }

    #[test]
    fn packetflow_detects_postgresql_backend_frame_19() {
        let packet = sample_ipv4_tcp_postgresql_backend_frame_19();

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer");
        assert_eq!(internet.protocol_name, "IPv4");
        assert_eq!(
            internet.source,
            Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
        );
        assert_eq!(
            internet.destination,
            Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
        );

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.source_port, Some(5432));
        assert_eq!(transport.destination_port, Some(45931));

        let payload = transport.payload.expect("tcp payload");
        assert_eq!(payload.len(), 161);

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "PostgreSQL");

        let pg = PostgreSqlPacket::try_from(payload).expect("postgresql backend packet");
        assert_eq!(pg.messages.len(), 8);
        assert_eq!(
            pg.messages[0].message_type,
            PostgreSqlMessageType::Authentication
        );
        assert_eq!(
            pg.messages[1].message_type,
            PostgreSqlMessageType::ParameterStatusOrSync
        );
        assert_eq!(
            pg.messages[6].message_type,
            PostgreSqlMessageType::BackendKeyData
        );
        assert_eq!(
            pg.messages[7].message_type,
            PostgreSqlMessageType::ReadyForQuery
        );
        assert_eq!(pg.messages[7].payload, b"I");
    }

    #[test]
    fn packetflow_does_not_detect_postgresql_from_weak_shape_on_standard_port() {
        let payload = [b'S', 0x00, 0x00, 0x00, 0x04];
        let packet = ethernet_ipv4_tcp_packet(51845, 5432, &payload);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.destination_port, Some(5432));

        let application = flow.application.as_ref().expect("application layer");
        assert_ne!(application.application_protocol, "PostgreSQL");
    }

    #[test]
    fn packetflow_partial_eq_same_packet_should_be_equal() {
        let packet = sample_ipv6_tcp_packet();

        let flow_a = PacketFlow::try_from(packet.as_slice()).unwrap();
        let flow_b = PacketFlow::try_from(packet.as_slice()).unwrap();

        assert_eq!(flow_a, flow_b);
    }

    #[test]
    fn packetflow_hash_same_packet_should_match() {
        let packet = sample_ipv6_tcp_packet();

        let flow_a = PacketFlow::try_from(packet.as_slice()).unwrap();
        let flow_b = PacketFlow::try_from(packet.as_slice()).unwrap();

        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();

        flow_a.hash(&mut hasher_a);
        flow_b.hash(&mut hasher_b);

        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn packetflow_to_owned_should_preserve_semantic_content() {
        let packet = sample_ipv6_tcp_packet();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let owned = flow.to_owned();

        assert_eq!(owned.data_link, flow.to_owned().data_link);

        match (&owned.internet, &flow.internet) {
            (Some(owned_internet), Some(flow_internet)) => {
                assert_eq!(owned_internet.source_ip, flow_internet.source);
                assert_eq!(owned_internet.ip_source_type, flow_internet.source_type);
                assert_eq!(owned_internet.destination_ip, flow_internet.destination);
                assert_eq!(
                    owned_internet.ip_destination_type,
                    flow_internet.destination_type
                );
                assert_eq!(owned_internet.protocol, flow_internet.protocol_name);
            }
            (None, None) => {}
            _ => panic!("owned.internet and flow.internet differ"),
        }

        match (&owned.transport, &flow.transport) {
            (Some(owned_transport), Some(flow_transport)) => {
                assert_eq!(
                    owned_transport.protocol,
                    format!("{:?}", flow_transport.protocol)
                );
                assert_eq!(owned_transport.source_port, flow_transport.source_port);
                assert_eq!(
                    owned_transport.destination_port,
                    flow_transport.destination_port
                );
            }
            (None, None) => {}
            _ => panic!("owned.transport and flow.transport differ"),
        }

        assert_eq!(owned.application, flow.to_owned().application);
    }

    #[cfg(feature = "parse_timing")]
    fn timed_parse(
        packet: &[u8],
    ) -> (
        Result<PacketFlow<'_>, ParsedPacketError>,
        crate::timing::ParseTiming,
    ) {
        let mut timing = crate::timing::ParseTiming::default();
        let result = PacketFlow::try_from_timed(packet, &mut timing);

        (result, timing)
    }

    #[cfg(feature = "parse_timing")]
    fn assert_total_timing_is_recorded(timing: crate::timing::ParseTiming) {
        assert!(timing.total_ns > 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn packetflow_timing_records_total_on_success() {
        let packet = ethernet_ipv4_udp_packet(0, 12);
        let (result, timing) = timed_parse(packet.as_slice());

        assert!(result.is_ok());
        assert_total_timing_is_recorded(timing);
        assert!(timing.l2_ns > 0);
        assert!(timing.l3_ns > 0);
        assert!(timing.l4_ns > 0);
        assert!(timing.l7_ns > 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn packetflow_timing_records_total_on_l2_error() {
        let (result, timing) = timed_parse(&[]);

        assert!(matches!(result, Err(ParsedPacketError::InvalidDataLink(_))));
        assert_total_timing_is_recorded(timing);
        assert!(timing.l2_ns > 0);
        assert_eq!(timing.l3_ns, 0);
        assert_eq!(timing.l4_ns, 0);
        assert_eq!(timing.l7_ns, 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn packetflow_timing_records_total_on_l3_corruption() {
        let packet = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, // Source MAC
            0x08, 0x00, // IPv4 EtherType, but no IP payload
        ];
        let (result, timing) = timed_parse(&packet);

        // Dégradation gracieuse : le L2 est conservé, la corruption signalée.
        let flow = result.expect("corrupt L3 must not fail parsing");
        assert!(flow.internet.is_none());
        assert_eq!(
            flow.corrupted.as_ref().map(|c| c.layer),
            Some(CorruptedLayerKind::Internet)
        );
        assert_total_timing_is_recorded(timing);
        assert!(timing.l2_ns > 0);
        assert!(timing.l3_ns > 0);
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn packetflow_timing_records_total_on_l4_corruption() {
        let packet = ethernet_ipv4_udp_packet(0, 16);
        let (result, timing) = timed_parse(packet.as_slice());

        // Dégradation gracieuse : L2/L3 conservés, corruption au transport.
        let flow = result.expect("corrupt L4 must not fail parsing");
        assert!(flow.internet.is_some());
        assert!(flow.transport.is_none());
        assert_eq!(
            flow.corrupted.as_ref().map(|c| c.layer),
            Some(CorruptedLayerKind::Transport)
        );
        assert_total_timing_is_recorded(timing);
        assert!(timing.l2_ns > 0);
        assert!(timing.l3_ns > 0);
        assert!(timing.l4_ns > 0);
    }

    #[test]
    fn packetflow_parses_transport_for_non_fragmented_ipv4_udp() {
        let packet = ethernet_ipv4_udp_packet(0, 12);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer");
        assert_eq!(internet.payload_protocol, Some(TransportProtocol::Udp));
        assert!(flow.transport.is_some());
    }

    #[test]
    fn packetflow_skips_transport_for_initial_ipv4_fragment() {
        let more_fragments = 0x2000;
        let packet = ethernet_ipv4_udp_packet(more_fragments, 16);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer");
        assert_eq!(internet.protocol_name, "IPv4");
        assert_eq!(internet.payload_protocol, None);
        assert!(flow.transport.is_none());
    }

    #[test]
    fn packetflow_skips_transport_for_non_initial_ipv4_fragment() {
        let packet = ethernet_ipv4_udp_packet(1, 16);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer");
        assert_eq!(internet.protocol_name, "IPv4");
        assert_eq!(internet.payload_protocol, None);
        assert!(flow.transport.is_none());
    }

    /// Trame réelle : Ethernet → IPv4 → UDP:5247 → CAPWAP-Data → IEEE 802.11 →
    /// LLC/SNAP → IPv4 → TCP:445 (SMB2). Un seul paquet, deux niveaux de flux.
    fn sample_capwap_ieee80211_inner_tcp() -> Vec<u8> {
        hex::decode(
            "c464138f9e04442b0302172c080045080138b7e04000ff115967ac18086aac1808ca2174147f0124000000200320000000000104e5440000000001082c00003a9a5af450e0c2642fa3b4000c29967ca43980aaaa030000000800453800eca89440008006651464ac911e64ac91b4dd7b01bda58ecb952483ab67501800fec87e0000000000c0fe534d4240000100030000000500000030000000000000006704090000000000fffe0000966e611ec3ba49eb000000000000000000000000000000000000000039000000020000000000000000000000000000000000000080000000000000000700000001000000000020007800120090000000300000005200530058005f00440052002d0053004600000000000000180000001000040000001800000000004d78416300000000000000001000040000001800000000005146696400000000",
        )
        .expect("invalid test hex fixture")
    }

    #[test]
    fn packetflow_recurses_into_capwap_ieee80211_tunnel() {
        use crate::parse::data_link::mac_addres::MacAddress;

        let packet = sample_capwap_ieee80211_inner_tcp();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        // --- Flux externe : le tunnel CAPWAP ---
        let outer_ip = flow.internet.as_ref().expect("outer internet");
        assert_eq!(
            outer_ip.source,
            Some(IpAddr::V4(Ipv4Addr::new(172, 24, 8, 106)))
        );
        assert_eq!(
            outer_ip.destination,
            Some(IpAddr::V4(Ipv4Addr::new(172, 24, 8, 202)))
        );
        let outer_udp = flow.transport.as_ref().expect("outer transport");
        assert_eq!(outer_udp.protocol, TransportProtocol::Udp);
        assert_eq!(outer_udp.destination_port, Some(5247));
        assert_eq!(
            flow.application
                .as_ref()
                .expect("outer application")
                .application_protocol,
            "CAPWAP",
            "le tunnel est reporté comme protocole applicatif de la ligne externe"
        );

        // --- Flux interne : la vraie conversation, extraite du tunnel ---
        let inner = flow.inner.as_deref().expect("inner tunnel flow");
        let inner_ip = inner.internet.as_ref().expect("inner internet");
        assert_eq!(
            inner_ip.source,
            Some(IpAddr::V4(Ipv4Addr::new(100, 172, 145, 30)))
        );
        assert_eq!(
            inner_ip.destination,
            Some(IpAddr::V4(Ipv4Addr::new(100, 172, 145, 180)))
        );
        let inner_tcp = inner.transport.as_ref().expect("inner transport");
        assert_eq!(inner_tcp.protocol, TransportProtocol::Tcp);
        assert_eq!(inner_tcp.source_port, Some(56699));
        assert_eq!(inner_tcp.destination_port, Some(445));

        // MAC internes = adresses 802.11 (station Intel -> destination VMware).
        assert_eq!(
            inner.data_link.source_mac,
            MacAddress([0xe0, 0xc2, 0x64, 0x2f, 0xa3, 0xb4])
        );
        assert_eq!(
            inner.data_link.destination_mac,
            MacAddress([0x00, 0x0c, 0x29, 0x96, 0x7c, 0xa4])
        );

        // Un paquet -> deux niveaux de flux aplatis.
        assert_eq!(flow.flatten().len(), 2);
    }

    /// EtherType IPv4 annoncé mais en-tête IP invalide : le L2 doit être
    /// conservé et la corruption signalée, sans échec du parsing.
    #[test]
    fn corrupt_ipv4_keeps_data_link_and_reports_corruption() {
        let mut packet = vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // dst MAC
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, // src MAC
            0x08, 0x00, // EtherType IPv4
        ];
        packet.extend_from_slice(&[0xFF; 6]); // octets invalides pour IPv4

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        assert_eq!(flow.data_link.ethertype.0, 0x0800);
        assert!(flow.internet.is_none());
        assert!(flow.transport.is_none());
        let corrupted = flow.corrupted.as_ref().expect("corruption report");
        assert_eq!(corrupted.layer, CorruptedLayerKind::Internet);
        assert!(!corrupted.error.is_empty());
    }

    /// EtherType inconnu (LLDP) : pas de L3, mais pas de corruption non plus.
    #[test]
    fn unknown_ethertype_is_not_corruption() {
        let mut packet = vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0x88,
            0xCC, // EtherType LLDP
        ];
        packet.extend_from_slice(&[0xFF; 6]);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        assert!(flow.internet.is_none());
        assert!(flow.corrupted.is_none());
    }

    /// IPv4 valide annonçant du TCP, mais segment TCP tronqué : le L3 doit
    /// être conservé et la corruption signalée au niveau transport.
    #[test]
    fn corrupt_tcp_keeps_internet_and_reports_corruption() {
        let mut packet = vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // dst MAC
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, // src MAC
            0x08, 0x00, // EtherType IPv4
        ];
        // IPv4 minimal : total length 24 = 20 d'en-tête + 4 octets de "TCP"
        // (bien trop court pour un en-tête TCP de 20 octets).
        packet.extend_from_slice(&[
            0x45, 0x00, 0x00, 0x18, // Version/IHL, DSCP, Total Length = 24
            0x12, 0x34, 0x00, 0x00, // Id, Flags/Fragment
            64, 6, 0x00, 0x00, // TTL, Protocol = TCP, Checksum
            192, 168, 1, 10, // Source IP
            192, 168, 1, 20, // Destination IP
            0xde, 0xad, 0xbe, 0xef, // pseudo-payload TCP tronqué
        ]);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer kept");
        assert_eq!(internet.protocol_name, "IPv4");
        assert!(flow.transport.is_none());
        let corrupted = flow.corrupted.as_ref().expect("corruption report");
        assert_eq!(corrupted.layer, CorruptedLayerKind::Transport);
    }

    /// La corruption survit à to_owned().
    #[test]
    fn to_owned_preserves_corruption() {
        let mut packet = vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0x08, 0x00,
        ];
        packet.extend_from_slice(&[0xFF; 6]);

        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
        let owned = flow.to_owned();

        assert_eq!(owned.corrupted, flow.corrupted);
    }

    #[test]
    fn to_owned_preserves_tunnel_inner() {
        let packet = sample_capwap_ieee80211_inner_tcp();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let owned = flow.to_owned();

        assert_eq!(
            owned
                .application
                .as_ref()
                .expect("outer application")
                .protocol,
            "CAPWAP"
        );
        let inner = owned.inner.as_deref().expect("inner owned flow");
        let inner_transport = inner.transport.as_ref().expect("inner transport");
        assert_eq!(inner_transport.source_port, Some(56699));
        assert_eq!(inner_transport.destination_port, Some(445));
        assert!(inner.inner.is_none());
    }

    #[cfg(feature = "parse_timing")]
    #[test]
    fn try_from_timed_returns_same_flow_as_try_from() {
        let packet = sample_capwap_ieee80211_inner_tcp();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let mut timing = crate::timing::ParseTiming::default();
        let timed_flow = PacketFlow::try_from_timed(packet.as_slice(), &mut timing).unwrap();

        assert_eq!(flow, timed_flow);
        assert!(timed_flow.inner.is_some(), "timed path must keep tunnels");
    }

    /// Sens retour de la même conversation : CAPWAP avec en-tête minimal
    /// (HLEN = 8 octets, pas d'info sans-fil) et 802.11 **FromDS** (descente
    /// vers la station). Complète le premier test qui était ToDS / HLEN 16.
    fn sample_capwap_ieee80211_fromds_inner_tcp() -> Vec<u8> {
        hex::decode(
            "442b0302172cc464138f9e04080045100160751a4000ff119bfdac1808caac18086a147f2174014c000000100300751a000002080000e0c2642fa3b4003a9a5af450000c29967ca40000aaaa0300000008004510011ce9f54000400663ab64ac91b464ac911e01bddd7b22759d73a3a432b450187ed43e350000000000f0fe534d424000010000000000050001003100000000000000120a060000000000fffe0000966e611ec3ba49eb0000000000000000000000000000000000000000590000000100000080a3cbc22d69d80100e1f6d4ba47db0100437f0f2e46db0100437f0f2e46db01000000000000000000000000000000001000000000000000c8edb59c0000000058d03e7b000000009800000058000000200000001000040000001800080000004d7841630000000000000000a9001200000000001000040000001800200000005146696400000000500158020000000001fe00000000000000000000000000000000000000000000",
        )
        .expect("invalid test hex fixture")
    }

    #[test]
    fn packetflow_recurses_into_capwap_fromds_short_header() {
        use crate::parse::data_link::mac_addres::MacAddress;

        let packet = sample_capwap_ieee80211_fromds_inner_tcp();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        // Flux externe : tunnel CAPWAP, sens serveur -> AP (UDP src 5247).
        let outer_ip = flow.internet.as_ref().expect("outer internet");
        assert_eq!(
            outer_ip.source,
            Some(IpAddr::V4(Ipv4Addr::new(172, 24, 8, 202)))
        );
        assert_eq!(
            outer_ip.destination,
            Some(IpAddr::V4(Ipv4Addr::new(172, 24, 8, 106)))
        );
        let outer_udp = flow.transport.as_ref().expect("outer transport");
        assert_eq!(outer_udp.source_port, Some(5247));
        assert_eq!(
            flow.application
                .as_ref()
                .expect("outer application")
                .application_protocol,
            "CAPWAP"
        );

        // Flux interne : réponse SMB (TCP src 445), IP inversées vs le test ToDS.
        let inner = flow.inner.as_deref().expect("inner tunnel flow");
        let inner_ip = inner.internet.as_ref().expect("inner internet");
        assert_eq!(
            inner_ip.source,
            Some(IpAddr::V4(Ipv4Addr::new(100, 172, 145, 180)))
        );
        assert_eq!(
            inner_ip.destination,
            Some(IpAddr::V4(Ipv4Addr::new(100, 172, 145, 30)))
        );
        let inner_tcp = inner.transport.as_ref().expect("inner transport");
        assert_eq!(inner_tcp.protocol, TransportProtocol::Tcp);
        assert_eq!(inner_tcp.source_port, Some(445));
        assert_eq!(inner_tcp.destination_port, Some(56699));

        // FromDS : DA = Address1 (station), SA = Address3 (hôte serveur).
        assert_eq!(
            inner.data_link.destination_mac,
            MacAddress([0xe0, 0xc2, 0x64, 0x2f, 0xa3, 0xb4])
        );
        assert_eq!(
            inner.data_link.source_mac,
            MacAddress([0x00, 0x0c, 0x29, 0x96, 0x7c, 0xa4])
        );

        assert_eq!(flow.flatten().len(), 2);
    }
}
