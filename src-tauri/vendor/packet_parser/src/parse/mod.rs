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

use crate::checks::application::quic::is_plausible_short_header;
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
    LinkLayer, LinkType, NetworkProtocol, ParseError,
    errors::{ParsedPacketError, internet::InternetError, transport::TransportError},
    owned::PacketFlowOwned,
};

pub mod application;
pub mod data_link;
pub mod internet;
mod link;
pub mod link_layer;
pub mod transport;
pub(crate) mod tunnel;

/// Returns whether this build has a decoder for the canonical link type.
///
/// This can be used to reject an unsupported capture interface before reading
/// or mutating packet-derived state.
#[inline(always)]
pub const fn is_supported(link_type: LinkType) -> bool {
    link::is_supported(link_type)
}

/// Parses one packet using the decoder selected by its explicit link type.
///
/// [`LinkType`] uses canonical `LINKTYPE_*` values. The caller is responsible
/// for mapping its capture source to that namespace and passing only packet
/// bytes, without a PCAP or PCAPNG record header.
#[inline(always)]
pub fn parse(link_type: LinkType, bytes: &[u8]) -> Result<PacketFlow<'_>, ParseError> {
    PacketFlow::parse_decoded(link::decode(link_type, bytes)?, 0)
}

/// Timed counterpart of [`parse`], using the exact same link-type dispatcher.
#[cfg(feature = "parse_timing")]
#[inline(always)]
pub fn parse_timed<'a>(
    link_type: LinkType,
    bytes: &'a [u8],
    timing: &mut crate::timing::ParseTiming,
) -> Result<PacketFlow<'a>, ParseError> {
    use crate::timing::{elapsed_ns, now};

    *timing = crate::timing::ParseTiming::default();
    let total_t0 = now();
    let result = (|| {
        let decoded = link::decode_timed(link_type, bytes, timing)?;
        PacketFlow::parse_decoded_timed(decoded, timing, 0)
    })();
    timing.total_ns = elapsed_ns(total_t0);
    result
}

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
    /// Link layer (mandatory), tagged with its canonical LINKTYPE.
    pub data_link: LinkLayer<'a>,

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
    type Error = ParseError;

    #[inline(always)]
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        parse(LinkType::ETHERNET, bytes)
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

        // QUIC 1-RTT (Short Header) : l'en-tête est volontairement opaque
        // (RFC 9000 §17.3, pas de version ni de longueur de DCID), seul un
        // détecteur stateful façon Wireshark peut le décoder. En stateless on
        // se limite à une heuristique (form=0, fixed=1, taille minimale)
        // gardée par le port UDP 443. Les Long Headers, eux, sont détectés
        // sans port par le probing générique (Application::try_from).
        if transport.protocol == TransportProtocol::Udp
            && (is_quic_udp_port(transport.source_port)
                || is_quic_udp_port(transport.destination_port))
            && is_plausible_short_header(payload)
        {
            return Some(Application {
                application_protocol: "QUIC",
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

    /// Parses the internet layer from the data-link layer, dispatching on the
    /// EtherType. An unknown EtherType yields `(None, None)`; a known
    /// EtherType with a corrupt payload yields `(None, Some(corruption))` —
    /// never a hard error, so the data-link information is preserved.
    #[inline(always)]
    fn parse_l3(
        network_protocol: NetworkProtocol,
        network_payload: &'a [u8],
    ) -> (Option<Internet<'a>>, Option<CorruptedLayer>) {
        match Internet::try_from_network_parts(network_protocol, network_payload) {
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

    /// Parses the shared L3/L4/L7 pipeline from a normalized link decoder
    /// output. `depth` bounds tunnel nesting.
    pub(crate) fn parse_decoded(
        decoded: link::DecodedLink<'a>,
        depth: u8,
    ) -> Result<Self, ParsedPacketError> {
        let (data_link, network_protocol, network_payload) = decoded.into_parts();
        let (internet, l3_corruption) = Self::parse_l3(network_protocol, network_payload);
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
    pub(crate) fn parse_decoded_timed(
        decoded: link::DecodedLink<'a>,
        timing: &mut crate::timing::ParseTiming,
        depth: u8,
    ) -> Result<Self, ParsedPacketError> {
        use crate::timing::{elapsed_ns, now};

        let (data_link, network_protocol, network_payload) = decoded.into_parts();
        let t0 = now();
        let internet = Self::parse_l3(network_protocol, network_payload);
        timing.l3_ns = elapsed_ns(t0);
        let (internet, l3_corruption) = internet;

        let t0 = now();
        let (transport, l4_corruption) = Self::parse_l4(internet.as_ref());
        timing.l4_ns = elapsed_ns(t0);

        // l7_ns includes tunnel detection and the recursive parsing of any
        // encapsulated packet.
        let t0 = now();
        let (application, inner) = Self::parse_l7_and_inner(transport.as_ref(), depth);
        timing.l7_ns = elapsed_ns(t0);

        Ok(PacketFlow {
            data_link,
            internet,
            transport,
            application,
            inner,
            corrupted: l3_corruption.or(l4_corruption),
        })
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
        bytes: &'a [u8],
        timing: &mut crate::timing::ParseTiming,
    ) -> Result<Self, ParseError> {
        parse_timed(LinkType::ETHERNET, bytes, timing)
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

/// QUIC (HTTP/3) : UDP 443.
fn is_quic_udp_port(port: Option<u16>) -> bool {
    matches!(port, Some(443))
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

    fn sample_quic_hxdump() -> Vec<u8> {
        hex::decode(
            "44152420a564e0d55e289bd486dd600a438a04d61140200108613fc79b009880c22c9b6600ba2a001450400c0c1d0000000000000054b21901bb04d689acce00000001089fdef188f3a0f2fe00404600e779f8fab6a42cbf19764ac834844fe3f9270a8a18d0fd0267b8cef3ab69c34028dcb8fc5e446f5ac727df40f9e40c7508b22db7d22df4ee4c750c6c2004fa7a0aad8c68cc447583631755cac8662d0da45f1499dbcea03c92e0f2c03066c0ae3be00fc828f42b01158d3f6b264c5d0f77f06651db0ce664bd646ac02e26ca6598f1cdee8195fd954a1358686b1fbbebcc597971f197f45e44078ebfb9e9a5c67d33446b3ef24d80fdb5e523b798bc877e1e55d71c23c68bf27788ef09793bf1bf0579179508bbf092c878424b868d8f7de251d4340b88a5389a60a9fc10a678e7c8e02341c0182963f9e5bf23e7d410a28587ad4529a7fd832c11dbf21d235ed3a87b75360c0b00a44ec5994e7fbf7c41c0541a66f578a96dabc285c1441ffc84b5493362b02c79778a59e229e37261212945a35243829d1d83cf2f48740029b1cc9b772cae528448dd9256886b7a021afdd4acf5b9cbc5deb62d53f8768a28d9e11a64dc9bcee1ce1c1ae971539028b96ea555f2c10fb8b145ec6bd349d2ff7bda1b848232cc3dd8f14058112379d71a2b843dbff6fc94d447f2c7ca81af20f68cd0a1ffc915dc9660c6c0ae552c8e34747c95e550bc769d09ad838165bd94b8cd356831a8a4b05ca255af060faee93df66e5d94c6f8e6677c8e6dc3144351a8c620f5801596e872a3b4b68d1b84f491e12e134eb7d24a57511b96ec6c98bd8e36091e49f6d28c8205a90b531680bcec215b425c09211be4afd1219ae67a19551752b862d539d4c2fc2a1256323a7c43f945bca8d69dde9a038d910ff43abc03b678a6d1d74dfdd9eb37de48e4b9926bd1b16bf0c4d33d99a9a1b06d181c156aa7b125543bfc74bbf3e9d0a42ad45e4563adcc090330fb688a414d4ba1550e8924f221879d4ac6bf62678a3972653aa8be4f5248a6a0dbd9e9fd7e4f8d6c74f36931f8ee64539739a5436fbf91bceef94c79b3dedcf6697575b846d67e75ac4f0ee6d3237b9fda36cf482455113e8228f90527e1ef9a039896d6f878aec3b7364d650872a8bed0e94192bbce7f3e5f159a311d6a4a88bfb82f36e6a15340ee462a1bef98f7ecf05ee6368773e4879b8aebd917925f8284c37084cbc75a0c115f2145e529fa1082ebd709273257ce5da188b50e4011ee1d0adf6dbbb1da455b34b298772c3001bcb633f58923437bc23b433cc3a7395ee486ef167194118ec902373eec3ded222289857b068651a52d99a3a7bc0e6311bcee18c8a276ce23c41ed015c4588d157673f7da67654110abac9c518141eb9dfbd30496b7996e11c4e36ad165dc8fa04a43b6ff9ab92381e93d25e148ab63fee22ea8076f7ff517be1d6bdd3e63de82fa8756f39687d86cdd9e2e491814ededbc407e210cf9c83b28663520a182935f970424de3f7edcf10db4a54d059157c6d0412d3f17bd7836c64453e3b90a96feef93c2f2de7cd9b2845abf58984a924b343ab530252036d51ffee2a7260c1422ec3574e4269a7763265f070fcfbc54d3fbccd28f1ae632e19eef1f330b01eaf214ec109d202a6248e33750d8d6137a3b4d5b41ecc324a5227f830afc1be368f2f61f6f71443995c078dababc9fe31e43d06e39b714b986709d15619e50b102454be9065d8c52401c52d5de2836287c90ff046708823ba9a780c622682c4d15702a78b723543bb2fe75584e58dd9ed25de06c710e3f",
        )
        .expect("invalid test hex fixture")
    }

    fn sample_quic_hxdump_1() -> Vec<u8> {
        hex::decode(
            "44152420a564e0d55e289bd486dd600a438a04d61140200108613fc79b009880c22c9b6600ba2a001450400c0c1d0000000000000054b21901bb04d689acce00000001089fdef188f3a0f2fe00404600e779f8fab6a42cbf19764ac834844fe3f9270a8a18d0fd0267b8cef3ab69c34028dcb8fc5e446f5ac727df40f9e40c7508b22db7d22df4ee4c750c6c2004fa7a0aad8c68cc4475b9c6b8c185164b7d25dd840dc4f760eee1c2d541908004ce7306e34069ef0f290cdd426a3dcafffdf81c4adedbc4a07665aaa153da60c5e33636035c7cfdb318db381046c66c39c8d1493629d5a188fd890fb220c8d344a67082cb87e82c24a46e9bf7815b42e6f2c1e5ac7a702d8dc7441c407fa726c1e434c106b2d3e4c9be188f77bffcadc12e879bfd53be6f5bff74060c4360495cfce26558550f320131e741e851b9139f16645c4a12c0a4c66cd75e1b12b17601c77f1656f9c359d17588276aee82ff7e9a6f84f80ad6c2c1f22f7f0a56685601b24ffe263a77a5417a067dbf706b800842039184ac6eba534fc95d7107ebd3914f0a5f3244bf2a8556ca7782d744bbb403679aa5c6725c6b7e62d4d3d34cf59f8bc5d4a323c14a2e1a5c0378f716fd5e4d59ed0cb727c1787d6c4b3f22845d3c8bb62ec1085218e1212c1db442a80c0be8863910cf1c0658f5d850604ad622205dfa325fc7bdb057be21df84411e71adb67daf495d2de0ff24d401c12d76a6a1662e5cdaff75cd969e0c243a0bf479061309fd5ad07ca642840f6c0fa6c39c46472112fa4bbdf4cf257e5ce205f04de503313035ac7f92a3237c4d2a199b29bb7d22c96a54c001ce1b42fd88b7a851c5cd3cd9b31ab75fd1bd4129134f592977bb16911f4c0e0f4469134412711d288657f3935cf4789cfa593a4878cb5f5b8135568ebe8cca39c50e317045b8fc10e805a130eb3d82546d6810195bfbdcf93dda639fa5bb7e50983a5a78ae92ecef702c635dabbdb49f35b08bbea4f78cba3329e197218e992266e8e9f65a37a9716ece10c90e52c655bfc28ee39d40b4f0c5f07d3c9d5250ad774e4cc5b4f7059a0ec95ae897fa823dc81c9646510de9d9d43fadb293c3c2afb36de10369d1d955bf9e13f39e14876291b22cff253f0a2cd882678c951b2e6211c47054657d9967f33abd72e28118194da7e4a6e612ff1aafee37bedaf8791c0e9cf8d3b63eb78bdd4bc912cb7012dc16ecacf41cd27d91868c1e9edfa83f0c1dc4037585e7e36ae7b2f789468e769d3f8247bc0e583022cc52b50a3d836bf9081e898d3917e62ba02b9169c8924c3061bb56655aa69b698dd78c780f9fc129c0423620a335ef57400fdbe3b96b92c733f567db07f8602f0753457e2c275034586b7fde510f466e452c480ea820c71b51b6ec114fbfeb71352c576db2f16d7b64ca8271ceae5a8007ef3995401f8e919071b0c813b72fb4a62e05ea3f424ed4e43d9b81ee99a083bac1a898916033e06c916d16e39871f4ac686846c8be6eda9f70968d2947af9fa020056c2931bf466722caef7241ddc2e7b8af84815788ced088363a968947ac3d770cbf1e6d81fc74eb20d83b3f8b07931fb062990dde84f6e5461ff0d33da496a2c69a9299bdcc4031473bfa4d738776dcc1d27dddb46279d3a4b23b058aa3248390dc79c676aa59f30e3497a4bca6caa9b9693bbf6648142dba1768c274e6c1bd40efcf9b1949ca88e5895e126359eb55ef317b0a458f8177cb6714223ca60cbdc59b5b85f0b3a43ee92054f8ff47e9bd133e732e2c801c0bd4cbddad4d1664ac6bbfaf6aff4053ed7830e65fab",
        )
        .expect("invalid test hex fixture")
    }

    fn sample_quic_hxdump_2() -> Vec<u8> {
        hex::decode(
            "44152420a564e0d55e289bd486dd600a438a00551140200108613fc79b009880c22c9b6600ba2a001450400c0c1d0000000000000054b21901bb0055852bd200000001089fdef188f3a0f2fe00403c36b482d0ad59a93db4091aca5a682718349c76bf3a7775c0479baa46c4ef61df76f763a4a620667fd6cc6bb70e3f9fdcaf4b12729dd60afef3da0008",
        )
        .expect("invalid test hex fixture")
    }

    fn sample_quic_hxdump_handshake() -> Vec<u8> {
        hex::decode(
            "44152420a564e0d55e289bd486dd600a438a00561140200108613fc79b009880c22c9b6600ba2a001450400c0c1d0000000000000054b21901bb0056852cec0000000108ffdef188f3a0f2fe00403d41ad9c41d1f28a94fa9403324682b513751c6ed7c2772fea3984260e174f1dd4ab0b080d81bdc8496bceaa653bde8c9af2dd7be4e93402e8cb7e77c455",
        )
        .expect("invalid test hex fixture")
    }

    fn sample_quic_hxdump_protected_payload() -> Vec<u8> {
        hex::decode(
            "44152420a564e0d55e289bd486dd600a438a00271140200108613fc79b009880c22c9b6600ba2a001450400c0c1d0000000000000054b21901bb002784fd4cffdef188f3a0f2fe823c3cf50ccc89c2d8016464794f376ddb705c5d1deb",
        )
        .expect("invalid test hex fixture")
    }

    #[test]
    fn packetflow_detects_quic() {
        let packet = sample_quic_hxdump();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "QUIC");

        let packet = sample_quic_hxdump_1();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "QUIC");

        let packet = sample_quic_hxdump_2();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "QUIC");
    }

    #[test]
    fn packetflow_detects_quic_protected_payload() {
        let packet = sample_quic_hxdump_protected_payload();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "QUIC");
    }

    #[test]
    fn packetflow_detects_quic_handshake() {
        let packet = sample_quic_hxdump_handshake();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "QUIC");
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

        assert!(!flow.data_link.network_payload().is_empty());
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
        let inner_wifi = inner
            .data_link
            .as_ieee80211()
            .expect("native IEEE 802.11 inner link");
        assert_eq!(inner.data_link.link_type(), LinkType::IEEE802_11);
        assert!(inner.data_link.as_ethernet().is_none());
        assert_eq!(
            inner_wifi.source_mac,
            MacAddress([0xe0, 0xc2, 0x64, 0x2f, 0xa3, 0xb4])
        );
        assert_eq!(
            inner_wifi.destination_mac,
            MacAddress([0x00, 0x0c, 0x29, 0x96, 0x7c, 0xa4])
        );

        // Un paquet -> deux niveaux de flux aplatis.
        assert_eq!(flow.flatten().len(), 2);
    }

    /// Trames réelles : pcaps_exemple/protocols/mqtt/mqtt_packets_tcpdump.pcap
    /// — session MQTT v3.1 ("MQIsdp") d'un client Paho vers
    /// m2m.eclipse.org:1883. Trames 1 (CONNECT), 2 (CONNACK), 3 (SUBSCRIBE),
    /// 5 (PUBLISH retain) et 6 (PINGREQ).
    #[test]
    fn packetflow_detects_mqtt_v31_session_from_real_capture() {
        let frames: &[(&str, &str)] = &[
            (
                "CONNECT (trame 1)",
                "24a2e1e6ee9b28cfe921148f08004500005b3ac6400040060fb90a000104c6291ef1c0af075bc1e1ff30793c11e38018203128b800000101080a3821d18838acc29c102500064d51497364700302000500177061686f2f333441414535344137354438333935363645",
            ),
            (
                "CONNACK (trame 2)",
                "28cfe921148f24a2e1e6ee9b080045200038172c000029068a56c6291ef10a000104075bc0af793c11e3c1e1ff57801800e34c1300000101080a38acc2e03821d18820020000",
            ),
            (
                "SUBSCRIBE (trame 3)",
                "24a2e1e6ee9b28cfe921148f080045000046ecbb400040065dd80a000104c6291ef1c0af075bc1e1ff57793c11e780182031749e00000101080a3821d27238acc2e082100001000b53616d706c65546f70696300",
            ),
            (
                "PUBLISH (trame 5)",
                "28cfe921148f24a2e1e6ee9b080045200066a7a000002906f9b3c6291ef10a000104075bc0af793c11ecc1e1ff69801800e382ac00000101080a38acc3563821d3613130000b53616d706c65546f70696348656c6c6f2066726f6d20746865205061686f20626c6f636b696e6720636c69656e74",
            ),
            (
                "PINGREQ (trame 6)",
                "24a2e1e6ee9b28cfe921148f080045000036b1ca4000400698d90a000104c6291ef1c0af075bc1e1ff69793c121e8018202d75d300000101080a3821e7be38acc356c000",
            ),
        ];

        for (name, hex_frame) in frames {
            let packet = hex::decode(hex_frame).expect("invalid test hex fixture");
            let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
            assert_eq!(
                flow.application
                    .as_ref()
                    .unwrap_or_else(|| panic!("{name}: pas d'application"))
                    .application_protocol,
                "MQTT",
                "{name} doit être détecté MQTT"
            );
        }
    }

    /// Régression faux positifs MQTT : trames réelles dont le premier octet
    /// de payload mime un fixed header MQTT (PUBCOMP/PUBACK/DISCONNECT/
    /// SUBSCRIBE) mais qui sont du trafic applicatif quelconque. Avant le
    /// durcissement des règles (remaining length par type, reason codes,
    /// consommation exacte du buffer), elles sortaient étiquetées "MQTT".
    #[test]
    fn mqtt_lookalike_noise_is_not_detected_as_mqtt() {
        let frames: &[(&str, &str)] = &[
            (
                // pcaps_exemple/protocols/tcp/tcp_retransmissions.pcapng, trame 1
                // (payload 70 3E… : PUBCOMP avec remaining length 62)
                "tcp_retransmissions trame 1",
                "00006510221b002078e15a800800450002b0690c40008006162e0a031e010a0347070418041300527b39006c9fdb501822389b5f0000703ece4b27b1db9282dea4181d0d86f0735041b579cd2edd961e1357db1defdb1b668eb17210b0a6663d17b65492d72b664c8f9de86b247f8b2bc0f0aafa6b8bdedcba1560529a7939f6d2a42da7fe2decea0d51f510255c14c187a900f63ce38e0bd47f845c4f9a8259a9f23b2dd059d17cb919338654b2395ded51577497383ff89b3041efc825878d9ed9ecee713c793a7c898e43a30923be326e3ec53cce3ca2f60e955a80a116ecdee89d7b9e565661f7adfcf313bd0ba244ea7b4e15a78a25b488479a53d12e3a71faff44f3abddbfb59bc115afc29edfce433c8fd9e7c99a2e1bfaa877a9e6ea2599e76a1fd33fb3e61a8e3c7dd21115a40636e259257fe293570596c48c69d327448ee7a9c38c81c63f2c5a09ab74568e85342c201f2f7c2809d01636d83eb0e0a1ceb712c418eff3de35a74bb10cb37a0e9a2bfd076ecfbdaf6af2b703bcbddf58681c1a6c7dd9ff2bbbe3e5d91575000017dfff9e8abeb77cf3cccaef06831a96197293d09f137ebc09279400939264d006060c19c3757b823da2295383ac2ea5ed594e2a79ceaa8456bab6159b2b2f35e2f870eb3e49b57ac41aa8fd229d17e1c13ab1ee1aeeddbfc88d872b3a97ce0a9eb65ef8c1b3bc5f4a18577c7b06cca4b1f7abd34db6997b59ab5bac875a130c94e9ccc988ddf95cc697135dc5b48021e90b0dbee5772d1e3c6892ec0bb9d7e97e2cc4f7f3b0f8d48534ffd232eaed990559a3c213957a5ba45798171af77a1d2dae890fba31a9d47ae45791a08c4a9df8950a3c00386fdc5671abf4f5ab63b84c66573877fbddb97d50a9b41198d5d5f89919d90f7d8a60571c3094fbe45f0723aeca2b45aa8cae0731c5c9594b47aff69025f7e21f230f6907ee74bbfb4c8d16aee2861efb0b19535a207600000000",
            ),
            (
                // pcaps_exemple/protocols/tcp/tcp_ports.pcapng, trame 322
                // (payload 40 66… : PUBACK avec remaining length 102)
                "tcp_ports trame 322",
                "00216a5b7d4a00055d21994c0800450005a649fc00003106cb6943e46e78ac10108000500b10884bad20dafa2dae5010003605bb000040664ca3039798a038b04189d2450009c428c6367cd1a1621463077618822fac919a19526f4f0f325d0a74980e1ef6f08cf38b63fe66e3400832e38767538638584132000001087bc8170e48e805468ce30a1af2e221ed338134446004796964193534c1e2218f926c7c2426658639b0d18f7eb04ac30d9ef19442ca0d91d48b80a48821cb3266ce1d5fb0c3048b38cb5d664e4376a0c111b44903617861087960002ef6202e208c6091c1614570ce399c3408c00ec7090000e4294fb8c8850601c8c32bd5f1ffba7ece939ef30ca83f07ea4f7e45a31cc620073292f1371a20a31ac69059010fb8ae8aae0b09363c991df8813f5e5e6e1ae200c01ee0d18021b4621a2420404a2f89c15d82f0a53005a1cc96610c75ec600f36dd432b76600dc460d090160d6a4547f04421ca2c00fffac10ff6b00375e4210d6ed84100984a4e9719c665d3198638b4cad5ad6e954cd120474fcbd1536b98351a3c04ea45d76a51eca183156940c2327041869f7cc02734b0c11492c2d70fb44143e872a63533279b682cc3b0862d4734ac81560da955a86dc5a037aed6035c7ce0ae777542146c7004be7a162f4eabe660f3d78cc39ad6b08c652c6ab8f058a162d4561c0ba21dea7ad919ff1c210d5168856793f2d761045697a3f5adae8e118bc31e63b18b45d030809a82d7beb6ad466c0e179a61071b60c2274e98820d68b05bde827696b41c6dbd8445de632c431bc85d6d6b210bdbe63083183b28800d463083178e802cddeded6f21185c99c9a6bcc28a067ab5a1dcf5b2755d636c8e867ef081294460b343e82e5fdbf05dd18ab7b4e6cdf079058cd674b09686157d2e74f3e2b46130781e9b15401e24cc5bc08616b8a3fdaf868f3be0027f11b2af4d7089197c591b44600b2c9e0185156ce1c18e77c61b46af7a41fc5c11b777c7b42d444f82acdf17f3b7bfa565999603ac64c78298bd088eee8e2f3be5200bb9c282bdf096b54ce30e7f98ff95607ef282697b573357b9c883c5702cf2520e721058c03686f39d0e6c2b322df8b2b5b5339a616ccdf19223003b20474e77700c021bd33e4e0eaa717c4b8e7498b8aeb635b3905583aef274f5d45f3db23ac8c16a75a8c31802a6e50ce33c544f23f1720cf6499959dc06ccd91a93310df6d9d6ec0d611d77368c33b05047606b4f1360074748b4997f20b396023bd8301df69a037cdc882e57d0601ec1566f4d805c4b9bd7beb626b6b33dd36de7251ac8be86b235dd6c99ed800cd11635c1aaadee75cbb4dd6b3e365a250a6ef68a5b66cee6b14f44ddeb5b5fdbdf121cc6b68d0def88cafbcbe1aef730f6d0933a4f9bdf3f4c0739452a9769c43473f8ff4b792e07feeda1d1fae0e4e6f1073a6be7744f9000c7880bc969208e93cf54e2403736af928d71836bfcde981535b591484ba42a55a9530d80d4a72ef574445ce5447ca0b7674debfb8c3be14967b8cd8771595e7cc0ec3fe045da19ac76b3374382589738042d3ed1a2d31be132dbc3cccf2de1a5a746663c36bbda9f3e78a5eee5ea41cf655e08c08d4bd0270c1828c108c2fd755cd7d6e3e876b889d1fe01a5b6ddf369ffc0db6546026e508271e6c50b31cc468e2358a004189845e4231f0614d4aad67827401b3ec001ccb37807986b2933d8def9c2af9df0c690e009b67084425c81069858c4115c7f062be420076118810358310256442107b02f41ff0e069007123523a5bcdc01a2699edf3dcc00130100e13540f8f41f1c61011f907afd3dbf8c9412e00cd78701076005182080567006747004ea300c97900ecd7009e2900e97400f00e00160000939600520800190800947b0078e340c5c103c448417e5100064800974800156400790f08230f88239800139e0031600069880096db017c680095b800927800db1a00e40700460600556500a00103eee2074b4a40cd8d0067bb00c944000e2100010508007608039000937",
            ),
            (
                // pcaps_exemple/protocols/tcp/tcp_ports.pcapng, trame 298
                // (payload E0 1E… : DISCONNECT avec remaining length 30)
                "tcp_ports trame 298",
                "00216a5b7d4a00055d21994c0800450005a63b1700003106da4e43e46e78ac10108000500b0c87b03364c304ad3a501000509f700000e01ef191ff5e1cbec7fd0ad34b1f7e82af7ee6bffc39bc38b965c57406ca68c132a98438a994be9296436c3dbb4903cd4ce41136a2cf52a0d0d4cfb9f578a16e977c6ea3c4bbaee231faf0a565e0e6aa89c9561c0f7c94eb9cda8769a771e846419e81ebce33f38bc88794e7e75d3d17de71935b0bb9d709620606ed856b54382f03c18d79c77571556422f2cd2ce2e4332a969c3299a69377f1da0c59c8cbe8ca7ce02f9a4faee536ec1935a605f65d2668f96cab3a8dec75439c1556f687b0479f3dca947350bf74f6f41ce67c0cfa7c8ea1f9ebfa9807f8f0f17a3ea6ffff61acdffe6ac33f38fa13d549cd2d16898a21e4b327f4506a73038d786833fa18d0a99741fed22eb05bfebdfdfee21ebf93fa1e996ae9b9b8f475f0046c241f07daaf8a9cd847e4e81ca24a9424ca2deeda454ea814059bccfc20d5b9f10d45516a6654ec0beed580effdd420637997716390d273e36628397c63280906abb50d2f7c1401346a42ca1a0045cc2a73bb624537f067909aa028d220388d9a3b94897b86b632d12d939b3e01ed3d895f75ac8e65f42c6ac80ebc1ae6384096c4f196706c1661113e4e8444d3d6ca9e9498d21acb9f0304c893ce810c3c0088d70571b9c7f010a314e50583de6fcbe3744a7f2feb793f5dbc0c496071784a7f6bffc9d685321fcf841de73a93362a2d4fbacca2164d6bd84ec45af204ba8b2c46b198db0f3cfd850f9fbbaffec0df7fea3fb9db1e36c00b5bcb83a20f1d671bedc8cc1ef5c239df8951499e08ec250f79629fb8a44f1c2741c97d4328b032edea047f392cbec1b52908e9c9fc28b9045dfa15f2732adbaa100d423c21ac741aaabe3d0d5f0ad959548c9b73bdefbf96cb5d2326ebf447ff13f965af9e2c2daf5c4d8617a0a971a7598b9d93aeaffd913fab4064a17b0bd0a45f47338b3f5de85b7999f5b521fa75234cc93377b802cc341d17929f12650e547cdf78992257a59a4cbc56d3fd01cebffdd64720c1059f759fbf9594cf5f0f2af4f9df6fc8d97870669626289abc01d31a5a0f87ebb2d9628b197a17d034ac35cf1785945a33caff3b7ebfcca007549a022c9fe4a5509092e4a16ca81d2f780060e531d560a53c0c912dc251a23a31bf1be4a78029fa43fd01901347b522e0cd9f7eeff1ea999f25752ab55d5f7b5e4b7cbbad0627354aa446d53c5563628801dd567fba6b7a508ea71190d48e4ab001eca53b8e8232193655184a79a513a12b7e2256f51b4d3b0899768c9b97df08993785dc0e2a92f706a087aabc645e27c35c99163cc76b20d51dc229a3cfb230af076a2ba9f4eb1a4a066930c3867a82aab51555c9912a30b16c08f5252896f68d195fa8eb69a42f81cac01db4a18c31821484b9134f829f7b14adede944505637b55f16a7bc4d59ae4a2d959819a543461b735ed747a2c19669a1c5ae9bf3d3256e77debf2634f058bbc9df31705c2c005d862b34fd3d27a4418b490e003a854def70bb74f895ff5df9b344824238c1f4157ef107d3cb9fe4ee47eb955a593a46ec7bcddd24470eab39f1094f7adeb2823585d7d4c335a25d057d3a496e8f40657cc65eb002aa1dba714b730194097c26b83a271ccc2cf5260944b559576113b7ad98b86bb4ac9bf674f54fb598ee618eb8d616ea03e0521e1da3a17c4c1053aaf4b7c1d6a9479c6c0caaf737052bd3d864455c81ea33f7741ac9508da164c69ab29df9ec3d5587102dc16f09ec4c50d2f9539ea0dac93dbd5581b5f88128bc16e394f1a83d79e04337ffdcaf831bc6a2b9d5bbd5454eb1f3c28ef30251f2726518f79c36ff0c23feaa335ebf61c0f7f60eadb5dfc3e5d1297aab3a66a0148d1955b90aac044c35a8d5f6454574fc9fba04bf90da9861020be26b54b78034793a34b8c759d05d1b1228c45ebba343dd50f5b029eb7206143abba310dacfd6fd7106757b7112e50da23f9ca4943fa1a2e5bb5d9ff72e7f0115",
            ),
            (
                // pcaps_exemple/protocols/arp/arppoison.pcapng, trame 110
                // (payload 82 4F… : SUBSCRIBE avec topic length délirante)
                "arppoison trame 110",
                "002170c056f00025b3bf91ee0800450000b9aa3400003506847f4a7d5f93ac10006b0050b27b6234c5312459e51980180138663d00000101080aa8efa0dd0008fbf0824f8252a034b6e014f7e6ad4e494bf9d4dd8257702952cc2d581c6c5aec6fc1202b1d7c731b071f876f3c78779363731be960962c6ee3e0a7258b5b3018c2f20b759b2d46af3c2c371e7de2af5db4ae16b096479234e71afc59964b47c41784b5044d0a1c6bb7282ede1040c22a1e05a3f29dfa884465517f5fa9bcafff030000ffff0d0a",
            ),
        ];

        for (name, hex_frame) in frames {
            let packet = hex::decode(hex_frame).expect("invalid test hex fixture");
            let flow = PacketFlow::try_from(packet.as_slice()).unwrap();
            let label = flow
                .application
                .as_ref()
                .map(|app| app.application_protocol)
                .unwrap_or("<aucune>");
            assert_ne!(label, "MQTT", "{name} ne doit pas être détecté MQTT");
        }
    }

    /// Trames réelles : pcaps_exemple/sll.pcap (Linux cooked v1, capture
    /// `tcpdump -i any`) — requêtes DNS vers 192.168.1.254:53 et
    /// systemd-resolved (127.0.0.53). Trames 98 (IPv4, sortante, hatype
    /// Ethernet), 96 (IPv6, sortante), 93 (IPv4 via loopback, hatype 772).
    #[test]
    fn sll_v1_real_capture_dispatches_ipv4_ipv6_and_loopback_to_dns() {
        use crate::{LinuxArphrdType, LinuxCookedPacketType};
        use std::net::Ipv6Addr;

        struct Expected {
            name: &'static str,
            hex: &'static str,
            packet_type: LinuxCookedPacketType,
            hardware_type: LinuxArphrdType,
            source_address: &'static [u8],
            sll_protocol: u16,
            source: IpAddr,
            destination: IpAddr,
            source_port: u16,
        }

        let frames = [
            Expected {
                name: "trame 98 (IPv4 sortante)",
                hex: "000400010006e0d55e289bd4000008004500004c717200004011842bc0a801b5c0a801fe8ef100350038854d3a4d0100000100000000000107756e6c6561736807636f646569756d03636f6d000041000100002905c0000000000000",
                packet_type: LinuxCookedPacketType::OUTGOING,
                hardware_type: LinuxArphrdType::ETHERNET,
                source_address: &[0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4],
                sll_protocol: 0x0800,
                source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 181)),
                destination: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)),
                source_port: 36593,
            },
            Expected {
                name: "trame 96 (IPv6 sortante)",
                hex: "000400010006e0d55e289bd4000086dd600f037800381140200108613fc79b00d48220dc0d4c71e6200108613fc79b00461524fffe20a5648db30035003889c8dee70100000100000000000107756e6c6561736807636f646569756d03636f6d000041000100002905ac000000000000",
                packet_type: LinuxCookedPacketType::OUTGOING,
                hardware_type: LinuxArphrdType::ETHERNET,
                source_address: &[0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4],
                sll_protocol: 0x86dd,
                source: IpAddr::V6(Ipv6Addr::new(
                    0x2001, 0x0861, 0x3fc7, 0x9b00, 0xd482, 0x20dc, 0x0d4c, 0x71e6,
                )),
                destination: IpAddr::V6(Ipv6Addr::new(
                    0x2001, 0x0861, 0x3fc7, 0x9b00, 0x4615, 0x24ff, 0xfe20, 0xa564,
                )),
                source_port: 36275,
            },
            Expected {
                name: "trame 93 (loopback vers systemd-resolved)",
                hex: "0000030400060000000000000000080045000041445540004011f8207f0000017f00003541470035002dfe74c1470100000100000000000007756e6c6561736807636f646569756d03636f6d0000410001",
                packet_type: LinuxCookedPacketType::HOST,
                hardware_type: LinuxArphrdType::LOOPBACK,
                source_address: &[0, 0, 0, 0, 0, 0],
                sll_protocol: 0x0800,
                source: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                destination: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 53)),
                source_port: 16711,
            },
        ];

        for expected in &frames {
            let name = expected.name;
            let packet = hex::decode(expected.hex).expect("invalid test hex fixture");
            let flow = parse(LinkType::LINUX_SLL, packet.as_slice()).unwrap();

            let sll = flow
                .data_link
                .as_linux_sll()
                .unwrap_or_else(|| panic!("{name}: lien SLL attendu"));
            assert_eq!(flow.data_link.link_type(), LinkType::LINUX_SLL, "{name}");
            assert_eq!(sll.packet_type, expected.packet_type, "{name}");
            assert_eq!(sll.hardware_type, expected.hardware_type, "{name}");
            assert_eq!(sll.address_length, 6, "{name}");
            assert_eq!(sll.source_address, Some(expected.source_address), "{name}");
            assert_eq!(sll.protocol, expected.sll_protocol, "{name}");

            let internet = flow
                .internet
                .as_ref()
                .unwrap_or_else(|| panic!("{name}: pas de L3"));
            assert_eq!(internet.source, Some(expected.source), "{name}");
            assert_eq!(internet.destination, Some(expected.destination), "{name}");

            let transport = flow
                .transport
                .as_ref()
                .unwrap_or_else(|| panic!("{name}: pas de L4"));
            assert_eq!(transport.protocol, TransportProtocol::Udp, "{name}");
            assert_eq!(transport.source_port, Some(expected.source_port), "{name}");
            assert_eq!(transport.destination_port, Some(53), "{name}");

            assert_eq!(
                flow.application
                    .as_ref()
                    .unwrap_or_else(|| panic!("{name}: pas de L7"))
                    .application_protocol,
                "DNS",
                "{name} doit être détecté DNS"
            );
        }
    }

    /// Trame réelle : pcaps_exemple/sll.pcap, trame 1864 — requête ARP
    /// « Who has 192.168.1.108? Tell 192.168.1.254 » vue à travers
    /// l'en-tête cooked v1 (le padding d'adresse non nul du noyau ne doit
    /// pas fuiter dans la vue de l'adresse).
    #[test]
    fn sll_v1_real_capture_dispatches_arp_without_transport() {
        use crate::parse::internet::InternetDetails;
        use crate::{LinuxArphrdType, LinuxCookedPacketType};

        let packet = hex::decode(
            "00000001000644152420a564742d0806000108000604000144152420a564c0a801fe000000000000c0a8016c00000000000000000000",
        )
        .expect("invalid test hex fixture");
        let flow = parse(LinkType::LINUX_SLL, packet.as_slice()).unwrap();

        let sll = flow.data_link.as_linux_sll().expect("lien SLL attendu");
        assert_eq!(sll.packet_type, LinuxCookedPacketType::HOST);
        assert_eq!(sll.hardware_type, LinuxArphrdType::ETHERNET);
        // Le slot d'adresse sur le fil contient 44:15:24:20:a5:64 suivi de
        // deux octets de padding non nuls (74:2d) : address_length = 6 doit
        // borner la vue.
        assert_eq!(
            sll.source_address,
            Some(&[0x44, 0x15, 0x24, 0x20, 0xa5, 0x64][..])
        );
        assert_eq!(sll.protocol, 0x0806);

        let internet = flow.internet.as_ref().expect("pas de L3");
        let InternetDetails::Arp(arp) = internet.details.as_ref().expect("détails ARP") else {
            panic!("détails ARP attendus, obtenu {:?}", internet.details);
        };
        assert_eq!(arp.operation, 1);
        assert_eq!(
            arp.sender_hardware_addr,
            [0x44, 0x15, 0x24, 0x20, 0xa5, 0x64]
        );
        assert_eq!(
            internet.source,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)))
        );
        assert_eq!(
            internet.destination,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 108)))
        );

        assert!(flow.transport.is_none());
        assert!(flow.application.is_none());
        assert!(flow.corrupted.is_none());
    }

    /// Trames réelles : pcaps_exemple/capture_sll2.pcap (Linux cooked v2,
    /// capture dumpcap `-i any`) — requêtes DNS vers 192.168.1.254:53 et
    /// systemd-resolved (127.0.0.53). Trames 87 (IPv4, sortante, ifindex 2),
    /// 86 (IPv6, sortante, ifindex 2), 83 (IPv4 via loopback, hatype 772,
    /// ifindex 1).
    #[test]
    fn sll2_real_capture_dispatches_ipv4_ipv6_and_loopback_to_dns() {
        use crate::{LinuxArphrdType, LinuxCookedPacketType};
        use std::net::Ipv6Addr;

        struct Expected {
            name: &'static str,
            hex: &'static str,
            sll_protocol: u16,
            interface_index: u32,
            packet_type: LinuxCookedPacketType,
            hardware_type: LinuxArphrdType,
            source_address: &'static [u8],
            source: IpAddr,
            destination: IpAddr,
            source_port: u16,
        }

        let frames = [
            Expected {
                name: "trame 87 (IPv4 sortante)",
                hex: "080000000000000200010406e0d55e289bd400004500004c6e84000040118719c0a801b5c0a801fed8e100350038854d0c5a01000001000000000001086163636f756e747306676f6f676c6503636f6d000041000100002905c0000000000000",
                sll_protocol: 0x0800,
                interface_index: 2,
                packet_type: LinuxCookedPacketType::OUTGOING,
                hardware_type: LinuxArphrdType::ETHERNET,
                source_address: &[0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4],
                source: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 181)),
                destination: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)),
                source_port: 55521,
            },
            Expected {
                name: "trame 86 (IPv6 sortante)",
                hex: "86dd00000000000200010406e0d55e289bd4000060073f1c00381140200108613fc79b00d48220dc0d4c71e6200108613fc79b00461524fffe20a56498140035003889c8575801000001000000000001086163636f756e747306676f6f676c6503636f6d000041000100002905ac000000000000",
                sll_protocol: 0x86dd,
                interface_index: 2,
                packet_type: LinuxCookedPacketType::OUTGOING,
                hardware_type: LinuxArphrdType::ETHERNET,
                source_address: &[0xe0, 0xd5, 0x5e, 0x28, 0x9b, 0xd4],
                source: IpAddr::V6(Ipv6Addr::new(
                    0x2001, 0x0861, 0x3fc7, 0x9b00, 0xd482, 0x20dc, 0x0d4c, 0x71e6,
                )),
                destination: IpAddr::V6(Ipv6Addr::new(
                    0x2001, 0x0861, 0x3fc7, 0x9b00, 0x4615, 0x24ff, 0xfe20, 0xa564,
                )),
                source_port: 38932,
            },
            Expected {
                name: "trame 83 (loopback vers systemd-resolved)",
                hex: "08000000000000010304000600000000000000004500004191e640004011aa8f7f0000017f000035aaf60035002dfe74136201000001000000000000086163636f756e747306676f6f676c6503636f6d0000410001",
                sll_protocol: 0x0800,
                interface_index: 1,
                packet_type: LinuxCookedPacketType::HOST,
                hardware_type: LinuxArphrdType::LOOPBACK,
                source_address: &[0, 0, 0, 0, 0, 0],
                source: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                destination: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 53)),
                source_port: 43766,
            },
        ];

        for expected in &frames {
            let name = expected.name;
            let packet = hex::decode(expected.hex).expect("invalid test hex fixture");
            let flow = parse(LinkType::LINUX_SLL2, packet.as_slice()).unwrap();

            let sll2 = flow
                .data_link
                .as_linux_sll2()
                .unwrap_or_else(|| panic!("{name}: lien SLL2 attendu"));
            assert_eq!(flow.data_link.link_type(), LinkType::LINUX_SLL2, "{name}");
            assert_eq!(sll2.protocol, expected.sll_protocol, "{name}");
            assert!(sll2.reserved_is_zero(), "{name}");
            assert_eq!(sll2.interface_index, expected.interface_index, "{name}");
            assert_eq!(sll2.packet_type, expected.packet_type, "{name}");
            assert_eq!(sll2.hardware_type, expected.hardware_type, "{name}");
            assert_eq!(sll2.address_length, 6, "{name}");
            assert_eq!(sll2.source_address, Some(expected.source_address), "{name}");

            let internet = flow
                .internet
                .as_ref()
                .unwrap_or_else(|| panic!("{name}: pas de L3"));
            assert_eq!(internet.source, Some(expected.source), "{name}");
            assert_eq!(internet.destination, Some(expected.destination), "{name}");

            let transport = flow
                .transport
                .as_ref()
                .unwrap_or_else(|| panic!("{name}: pas de L4"));
            assert_eq!(transport.protocol, TransportProtocol::Udp, "{name}");
            assert_eq!(transport.source_port, Some(expected.source_port), "{name}");
            assert_eq!(transport.destination_port, Some(53), "{name}");

            assert_eq!(
                flow.application
                    .as_ref()
                    .unwrap_or_else(|| panic!("{name}: pas de L7"))
                    .application_protocol,
                "DNS",
                "{name} doit être détecté DNS"
            );
        }
    }

    /// Trame réelle : pcaps_exemple/capture_sll2.pcap, trame 121 — requête
    /// ARP « Who has 192.168.1.181? Tell 192.168.1.254 » reçue sur
    /// l'ifindex 2 à travers l'en-tête cooked v2.
    #[test]
    fn sll2_real_capture_dispatches_arp_without_transport() {
        use crate::parse::internet::InternetDetails;
        use crate::{LinuxArphrdType, LinuxCookedPacketType};

        let packet = hex::decode(
            "08060000000000020001000644152420a5640000000108000604000144152420a564c0a801fe000000000000c0a801b5000000000000000000000000000000000000",
        )
        .expect("invalid test hex fixture");
        let flow = parse(LinkType::LINUX_SLL2, packet.as_slice()).unwrap();

        let sll2 = flow.data_link.as_linux_sll2().expect("lien SLL2 attendu");
        assert_eq!(sll2.protocol, 0x0806);
        assert!(sll2.reserved_is_zero());
        assert_eq!(sll2.interface_index, 2);
        assert_eq!(sll2.packet_type, LinuxCookedPacketType::HOST);
        assert_eq!(sll2.hardware_type, LinuxArphrdType::ETHERNET);
        assert_eq!(
            sll2.source_address,
            Some(&[0x44, 0x15, 0x24, 0x20, 0xa5, 0x64][..])
        );

        let internet = flow.internet.as_ref().expect("pas de L3");
        let InternetDetails::Arp(arp) = internet.details.as_ref().expect("détails ARP") else {
            panic!("détails ARP attendus, obtenu {:?}", internet.details);
        };
        assert_eq!(arp.operation, 1);
        assert_eq!(
            arp.sender_hardware_addr,
            [0x44, 0x15, 0x24, 0x20, 0xa5, 0x64]
        );
        assert_eq!(
            internet.source,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)))
        );
        assert_eq!(
            internet.destination,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 181)))
        );

        assert!(flow.transport.is_none());
        assert!(flow.application.is_none());
        assert!(flow.corrupted.is_none());
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

        assert_eq!(flow.data_link.as_ethernet().unwrap().ethertype.0, 0x0800);
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
        let borrowed_inner = flow.inner.as_deref().expect("inner borrowed flow");
        assert_eq!(inner.data_link.link_type(), LinkType::IEEE802_11);
        assert!(inner.data_link.as_ieee80211().is_some());
        assert_eq!(
            serde_json::to_value(&borrowed_inner.data_link).unwrap(),
            serde_json::to_value(&inner.data_link).unwrap()
        );
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
        let inner_wifi = inner
            .data_link
            .as_ieee80211()
            .expect("native IEEE 802.11 inner link");
        assert_eq!(inner.data_link.link_type(), LinkType::IEEE802_11);
        assert_eq!(
            inner_wifi.destination_mac,
            MacAddress([0xe0, 0xc2, 0x64, 0x2f, 0xa3, 0xb4])
        );
        assert_eq!(
            inner_wifi.source_mac,
            MacAddress([0x00, 0x0c, 0x29, 0x96, 0x7c, 0xa4])
        );

        assert_eq!(flow.flatten().len(), 2);
    }

    // Trames réelles : pcaps_exemple/protocols/tls/tls1.3.pcapng — session
    // TLS 1.3 openssl s_client -> s_server sur loopback (127.0.0.1:4443),
    // capture issue de github.com/tex2e/openssl-playground.

    /// Trame 4 : ClientHello TLSv1.3 (record layer en version legacy 0x0301,
    /// extension supported_versions = TLS 1.3 uniquement).
    fn sample_tlsv3_client_hello() -> Vec<u8> {
        hex::decode(
            "00000000000000000000000008004500010b94e440004006a7067f0000017f00000184dc115bd5b503fbebbcb42b80180200feff00000101080a64b097c664b097c616030100d2010000ce0303987b596feff4ea77cd7743d5c52192a1e95eee1b926c86e4d6ada4b39e087c2320b98aa442054ba9403f8c5fec4c6da74771b7e0cae70e703758d43280a0c2c1b8000813021303130100ff0100007d000b000403000102000a000c000a001d0017001e00190018002300000016000000170000000d001e001c040305030603080708080809080a080b080408050806040105010601002b0003020304002d00020101003300260024001d0020f9d63defe78ed9a66cef42ab56e27a97562a534ae6186fb564a98d044e8ac665",
        )
        .expect("invalid test hex fixture")
    }

    /// Trame 6 : réponse serveur — six records TLS dans le même segment TCP :
    /// ServerHello, ChangeCipherSpec, puis quatre records chiffrés
    /// (EncryptedExtensions, Certificate, CertificateVerify, Finished)
    /// annoncés comme Application Data.
    fn sample_tlsv3_server_hello() -> Vec<u8> {
        hex::decode(
            "00000000000000000000000008004500056dbaf3400040067c957f0000017f000001115b84dcebbcb42bd5b504d280180200036200000101080a64b097cd64b097c6160303007a020000760303f1075544aed0578146368af51ebca7755b4057296f8b887c1576295ad529dc5f20b98aa442054ba9403f8c5fec4c6da74771b7e0cae70e703758d43280a0c2c1b8130200002e002b0002030400330024001d00206f830422af5e6f22368d20607c67c3956d8b8a5bbd8cbc71c92c92efaab2c91c1403030001011703030017b0891cce8c1b5dbb3710ab331ed44441cc8319d817280f170303032b2b622969394121ba16170f3fedd3cee7a5855df6dcd78196574827560e0cec51bd40cb3c4eee46ddee30e16c20e71d593f57de3b807e4a9100193aa68056c92bf8fb493c5b1183a2e8a7dc5f722bb307f8d7c069fbefe6494dda7f3982a2b1c7959d935e3f18830e4b26bd1f9e34ea7421c0819e964cc8b674a106f57f5e8a3b49f3cbed90513902928a5cd9dbdbb2ab3b6a3af44d5cfdfcbfaa14805315ca5274d1f449ce52f346f363b2564248ccf31c17fc476faf8708060145eedafa6f0ee96c0bb6fa38d41c7d2990a75d6b7414af876b935066dac69df387e0a3477ef53b253720af1112d557d895ee2b7936433ca35ab37052dd7b349bb42169a793e2b06b7f73014c820313d9c8653ca4b46459ab65f06746430784324e0dacbedd80859c69062e48adbaed5a22f9bd32f0fa475956b8976fa91e6f74297c6ad0e76f901bd3fbe8e1550de4de3d7a70df0992bba9735878a8263e710b07cbb0bbde8a278fa2b8772f5e0f763414dfe3ebd573d7ad9deb929cf84a5944ada4d960c61cd1b2625670272a2d7d16d1148efb186cfc909914b10f4e8aa3ce4dbcc2704e576141d06cae7bce11acebc3b9b091060f23a6d787d0c7d3dc269f8331e70b1e99d4ce60feb22e3294b51cfd8ba75dfb62dcb120934ecc7d4b37e499a99d3cfdc0e67633a42f3c72b034567e17dfd847301dcae4bccccbea7f500e3c49e564404618ada6567006d19409ac7e055a269c0649a24c17c0f007a4c7488f6037535ee83c010a46df77ad69403f3fa2cbfe7d50cbcd3ba730702f1b68e72bf9a95a6d688dd7ac9aae3434207bae61b7bc4549832c3aef0c2913c989790c8231bd365e32381292b351716d78aaf88169312a122374bcfc829bfd6af05c7494e23809945d285bd6a684f783513483f506374000799ed900e92036ab4f6ed871987ddbb35a0a6033772a4e7b50300e5ecc7981eedd84384a0d1b51fdb080af3abe28a034ae5149a42a48753fa882a7271ed6dcd5800b685b046ff9794584f6ae6a79c71f5f1edc9d233f85d580bf37c85bcc021d45d340f1ebff1ef4b3df6e27fff60c70b127818e91ade8ad704877f36e65f030698aaa9815b41faee698a40e2dc779f911b347bbf3bc9d5f812595b317030301196f7d7a403e3173b32b1dcf87bc7a8331f89fd5bd98f65788eb70c11e032a48dded8694b8a381bb2d3f4f159878fc2c020f97a38178ecf8c16add420da4abfea59e16e7b5c128f2b1eab6ce49c41f8141379db20f05ee8f701b48ed26e67a81a0b1c4d191ebf560f9774d1c399d61187454f718d519eca300dd4f491bb6905a123495f250301af22bebd90d3f946dd206494f5ae2d7abcb6664d3620bd43a4c4521e75147e2583585e4cf21fc2824695ee68e512126361bdffbecc2505ede998ac8499b5ca5d42a9eeb614174c0b3a67d01bc2c66837ab6644a02ab34ec7e13f66e6fe66e1589a0894b60724cf83eaf25cc69d7b67b07b1ddee1c4ac5a62bb8491340db21f760abb34851940d5030a5f96c90d47a8db1c62f381703030045262610454da3638060106eb4581be0b529977e7df24b996730ab7fa1880db9007cc4d6d826393a603e08655c28a6cdb5aa57cc4249a86216dad14742857451dc39e1f8c889",
        )
        .expect("invalid test hex fixture")
    }

    /// Trame 8 : fin de handshake côté client — ChangeCipherSpec puis
    /// Finished chiffré (Application Data) dans le même segment TCP.
    fn sample_tlsv3_change_cypher_spec() -> Vec<u8> {
        hex::decode(
            "00000000000000000000000008004500008494e640004006a78b7f0000017f00000184dc115bd5b504d2ebbcb96480180200fe7800000101080a64b097ce64b097cd1403030001011703030045591262704518d3d8b4792fd81a61a233aae2be7b7d1454ff722087ecf85aef68f6c07625ebcd6ec250754280e6ac73b96c36ff492b82962a63aba1a7750c55c09e68f23ea2",
        )
        .expect("invalid test hex fixture")
    }

    /// Trame 19 : record Application Data applicatif (données chiffrées).
    fn sample_tlsv3_application_data() -> Vec<u8> {
        hex::decode(
            "00000000000000000000000008004500004c94eb40004006a7be7f0000017f00000184dc115bd5b50547ebbccc1080180200fe4000000101080a64b097d864b097d5170303001351e67564011ed1bebcb4cf70f19c845b0e8920",
        )
        .expect("invalid test hex fixture")
    }

    /// tls1.3.pcapng, trame 4 : le ClientHello TLSv1.3 traverse tout le
    /// pipeline (IPv4 -> TCP -> application "TLS") et le record se décode
    /// champ à champ comme dans la dissection tshark.
    #[test]
    fn packetflow_detects_tlsv3() {
        use crate::parse::application::protocols::tls::{
            TlsContentType, TlsVersion, parse_tls_records,
        };

        let packet = sample_tlsv3_client_hello();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let internet = flow.internet.as_ref().expect("internet layer");
        assert_eq!(internet.protocol_name, "IPv4");
        assert_eq!(internet.source, Some(IpAddr::V4(Ipv4Addr::LOCALHOST)));
        assert_eq!(internet.destination, Some(IpAddr::V4(Ipv4Addr::LOCALHOST)));

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.source_port, Some(34012));
        assert_eq!(transport.destination_port, Some(4443));

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "TLS");

        // Le payload TCP est exactement un record TLS complet de 215 octets :
        // 5 d'en-tête + 210 de ClientHello.
        let payload = transport.payload.expect("tcp payload");
        assert_eq!(payload.len(), 215);

        let records = parse_tls_records(payload);
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.content_type, TlsContentType::Handshake);
        // Legacy version du record layer : 0x0301. La version TLS 1.3 réelle
        // se négocie dans l'extension supported_versions du ClientHello.
        assert_eq!(record.version, TlsVersion { major: 3, minor: 1 });
        assert_eq!(record.length, 210);
        // Handshake Type: Client Hello (1), Length: 206.
        assert_eq!(record.payload[0], 1);
        assert_eq!(record.payload[1..4], [0x00, 0x00, 0xce]);
    }

    /// tls1.3.pcapng, trame 6 : la réponse serveur porte six records TLS
    /// dans le même segment TCP, tous décodés par parse_tls_records.
    #[test]
    fn packetflow_detects_tlsv3_server_hello_multi_records() {
        use crate::parse::application::protocols::tls::{
            TlsContentType, TlsVersion, parse_tls_records,
        };

        let packet = sample_tlsv3_server_hello();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.source_port, Some(4443));
        assert_eq!(transport.destination_port, Some(34012));

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "TLS");

        let records = parse_tls_records(transport.payload.expect("tcp payload"));
        assert_eq!(records.len(), 6);

        // ServerHello (type 2) : supported_versions = TLS 1.3 (0x0304).
        assert_eq!(records[0].content_type, TlsContentType::Handshake);
        assert_eq!(records[0].length, 122);
        assert_eq!(records[0].payload[0], 2);
        let supported_versions_tls13 = [0x00, 0x2b, 0x00, 0x02, 0x03, 0x04];
        assert!(
            records[0]
                .payload
                .windows(supported_versions_tls13.len())
                .any(|w| w == supported_versions_tls13),
            "le ServerHello doit négocier TLS 1.3 via supported_versions"
        );

        assert_eq!(records[1].content_type, TlsContentType::ChangeCipherSpec);
        assert_eq!(records[1].payload, &[0x01]);

        // EncryptedExtensions, Certificate, CertificateVerify et Finished
        // chiffrés : annoncés Application Data en version legacy 0x0303.
        let encrypted_lengths = [23, 811, 281, 69];
        for (record, expected_length) in records[2..].iter().zip(encrypted_lengths) {
            assert_eq!(record.content_type, TlsContentType::ApplicationData);
            assert_eq!(record.version, TlsVersion { major: 3, minor: 3 });
            assert_eq!(record.length, expected_length);
        }
    }

    /// tls1.3.pcapng, trame 8 : fin de handshake client — ChangeCipherSpec
    /// puis Finished chiffré dans le même segment TCP.
    #[test]
    fn packetflow_detects_tlsv3_change_cypher_spec() {
        use crate::parse::application::protocols::tls::{
            TlsContentType, TlsVersion, parse_tls_records,
        };

        let packet = sample_tlsv3_change_cypher_spec();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.source_port, Some(34012));
        assert_eq!(transport.destination_port, Some(4443));

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "TLS");

        let records = parse_tls_records(transport.payload.expect("tcp payload"));
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].content_type, TlsContentType::ChangeCipherSpec);
        assert_eq!(records[0].length, 1);
        assert_eq!(records[0].payload, &[0x01]);

        assert_eq!(records[1].content_type, TlsContentType::ApplicationData);
        assert_eq!(records[1].version, TlsVersion { major: 3, minor: 3 });
        assert_eq!(records[1].length, 69);
    }

    /// tls1.3.pcapng, trame 19 : record Application Data applicatif ; en
    /// TLS 1.3 les records chiffrés s'annoncent en version legacy 0x0303.
    #[test]
    fn packetflow_detects_tlsv3_application_data() {
        use crate::parse::application::protocols::tls::{
            TlsContentType, TlsVersion, parse_tls_records,
        };

        let packet = sample_tlsv3_application_data();
        let flow = PacketFlow::try_from(packet.as_slice()).unwrap();

        let transport = flow.transport.as_ref().expect("transport layer");
        assert_eq!(transport.source_port, Some(34012));
        assert_eq!(transport.destination_port, Some(4443));

        let application = flow.application.as_ref().expect("application layer");
        assert_eq!(application.application_protocol, "TLS");

        let records = parse_tls_records(transport.payload.expect("tcp payload"));
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content_type, TlsContentType::ApplicationData);
        assert_eq!(records[0].version, TlsVersion { major: 3, minor: 3 });
        assert_eq!(records[0].length, 19);
    }
}
