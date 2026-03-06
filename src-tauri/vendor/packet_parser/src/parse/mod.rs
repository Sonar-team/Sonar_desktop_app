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
//! - Deterministic, allocation-free parsing using `&[u8]` references
//! - Clear separation between protocol layers
//! - Robust handling of unknown or unsupported protocols
//! - Suitable for network auditing, traffic analysis and post-capture inspection
//!
//! This module does **not** perform stream reassembly or session tracking.
//! It expects a complete packet buffer (e.g. from PCAP capture).

use application::Application;
use internet::Internet;
use serde::Serialize;
use transport::Transport;

use crate::{
    DataLink,
    errors::{ParsedPacketError, internet::InternetError, transport::TransportError},
    owned::PacketFlowOwned,
    parse::transport::protocols::TransportProtocol,
};

pub mod application;
pub mod data_link;
pub mod internet;
pub mod transport;

/// A fully or partially parsed network packet flow.
///
/// `PacketFlow` represents a packet parsed across protocol layers.
/// Each layer is optional except for the data link layer, which is mandatory.
///
/// Unsupported or unrecognized protocols do **not** fail parsing and instead
/// result in `None` for the corresponding layer.
///
/// The structure borrows from the original packet buffer (`&[u8]`) and is
/// therefore zero-copy.
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
    }
}

use std::hash::{Hash, Hasher};

impl<'a> Hash for PacketFlow<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_link.hash(state);
        self.internet.hash(state);
        self.transport.hash(state);
        self.application.hash(state);
    }
}

impl<'a> PacketFlow<'a> {
    /// Converts this borrowed [`PacketFlow`] into an owned version.
    ///
    /// This performs the necessary allocations to detach from the original
    /// packet buffer and is suitable for storage, serialization or cross-thread
    /// usage.
    pub fn to_owned(&self) -> PacketFlowOwned {
        PacketFlowOwned::from(self.clone())
    }

    #[inline(always)]
    fn parse_impl(packets: &'a [u8]) -> Result<Self, ParsedPacketError> {
        let data_link = DataLink::try_from(packets)?;

        let mut internet = match Internet::try_from(data_link.payload) {
            Ok(internet) => Some(internet),
            Err(InternetError::UnsupportedProtocol) => None,
            Err(e) => return Err(e.into()),
        };
        let transport = match internet.as_mut() {
            Some(internet) => match Transport::try_from(internet.payload) {
                Ok(transport) => Some(transport),
                Err(TransportError::UnsupportedProtocol) => internet
                    .payload_protocol
                    .take()
                    .map(TransportProtocol::to_transport),
                Err(e) => return Err(e.into()),
            },
            None => None,
        };

        let application = match &transport {
            Some(t) => match t.payload {
                Some(p) => Application::try_from(p).ok(),
                None => None,
            },
            None => None,
        };

        Ok(PacketFlow {
            data_link,
            internet,
            transport,
            application,
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

        // L2
        let t0 = now();
        let data_link = DataLink::try_from(packets)?;
        timing.l2_ns = elapsed_ns(t0);

        // L3 (tentative; may become None if unsupported)
        let t0 = now();
        let mut internet = match Internet::try_from(data_link.payload) {
            Ok(internet) => Some(internet),
            Err(InternetError::UnsupportedProtocol) => None,
            Err(e) => return Err(e.into()),
        };
        timing.l3_ns = elapsed_ns(t0);

        // L4 (tentative only if L3 exists)
        let t0 = now();
        let transport = match internet.as_mut() {
            Some(internet) => match Transport::try_from(internet.payload) {
                Ok(transport) => Some(transport),
                Err(TransportError::UnsupportedProtocol) => internet
                    .payload_protocol
                    .take()
                    .map(TransportProtocol::to_transport),
                Err(e) => return Err(e.into()),
            },
            None => None,
        };
        timing.l4_ns = elapsed_ns(t0);

        // L7 (best-effort; attempt only if L4 payload exists)
        let t0 = now();
        let application = match &transport {
            Some(t) => match t.payload {
                Some(p) => Application::try_from(p).ok(),
                None => None,
            },
            None => None,
        };
        timing.l7_ns = elapsed_ns(t0);

        timing.total_ns = elapsed_ns(total_t0);

        Ok(PacketFlow {
            data_link,
            internet,
            transport,
            application,
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
        packets: &'a [u8],
        timing: &mut crate::timing::ParseTiming,
    ) -> Result<Self, ParsedPacketError> {
        *timing = crate::timing::ParseTiming::default();
        Self::parse_impl_timed(packets, timing)
    }
}

// #[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn ethertype_ipv6_tcp() {
        use crate::PacketFlow;
        let packet = hex::decode("646ee0eafa83feaa818ec86486dd6500000004d8063d2a0014504007080a00000000000020012a04cec011a3a3971842f3918f5dec7d01bbe8007dc6e5a6f851bd128018007a628400000101080abd9295c7f7f13851edd897cab0251b0fcd0ce6976e5ffc596564fa9d1986ad4dcd59e5481c0ffa357590175e992da0a3ec8d32403b4ebb23b181e8916f5aa518eef4e126efe31b847f56868867cf26b0acd92680833ceefa8fb7fe1c3f1d96d1693b677b26d76acc7ff4e0e9ff9f5a6b5176689100891de5596ed15c93ec2d87570b13c73c95881562dcebad7acacf6dee4f8872ab2e07dacc00abf8534f8465b3f70a9362dd466bed097dd3943c49e60254c2d1d11e8a43db7b2bb20fac75bd2d12e61a135fc08fb817cf2779363052d5b8698712a0681510513bcd0d3095af28c63ae243006d44d792faa21d5a866c88e8948074e1bf9969d6bf965a796346553d7b64384ccf6a8ac5203aa1820ed3f46a3b656c5bb4670c6240da14f82c8e27cb1be60439c9aa0f9f58f716194deaf5ca10bba3f71d3beb73d878e0768f8ac20e7d1d984bdcdcd44bf861ae99c12a7307fc4ede845580bb97903da6b640403bdfd317b65b97d279d8b9ca5df881b305cf0ebe82d1aa4fd32fb9463653d11ede2327dbbf82453870017c4b6f69daeca416bbe5a1138c62e0da69dd8568b017b1c6b9def2ad9f5ae04bab9add00ddc790ebda970a5c80d44334c1a0a03ce1428efa2d6c260cf78e6442313fd5eacdf578572ef6ab4df6d6b6d9b889b2be67f0c8ae5d87923ce89df59386b8ecb29aeb1e6c5c5be465e3ad4b62c167443068a268ff6067be0f637f5e9994635c09d73c2bc5c5bc76f8fb2b1f00417ee67792cea34ab05451468de91524dbfe127463824e1d3fcc03fda2ff01ae8d21c242b996bc9b138a0ce211166af40b21a32b0b202aa8587430f03a46e9fe87f5991c132cc9c09ce36757888d913da28261e07e537d66f8c76abbd0cfd60236c880dfe49ad48a8ba9dfefe0efede8b004c7fc86b914fc4f4dd067f8dee3c8ecd89a47eafd438523d8ffd9fd1fa5797cd446fe019f8b5cd4cf0bb6e6800f1c06f04dafbc009b558abeb5821cd5c5a6f9b24fe47606ca098290845ba5ac42aa994844553f7522efa08f99b7e62a858cdd1d7376b552fa2ddd87d4f8945292a31654f4032a9e6dc86584bc882bfd063e439fb701da038b23791a0706a1672bd6d70234ceef5340c975a473f8f524743672a284e22098d525b6ff48c54c0d79fe2d67ea4b5619536ef182fe181def5c640961138ed1e7bbb795475295ca3418b8ab5b594307f7338e5689b2fea6aed83a08c356f4e4d072dad9b5b3e38bd9a4c5a632c5f024e892e85341da285eb2098a7d1d114ba8662e6f5c33513cc0d5d0d0186ae7aadab3334d03a8644c3774a16bd985cc198f48012bbe5d9c952472936e7b06c9e663ddb0cdc0fdbcf07e19d11064fe5f9e6f81d7440981331f2faab3f69466af1cd7d8a28c99f680ed88a24e27e53ae2b6d2323aa7592a0d169094eaf5134d421f66934a21e75a6d6532caa0c2c86697ba0b4c3cc484081ef8c94f2609a8b648527ae6926d72eecba718f51e61ce405f36c25e20978e40d5d9dc76dec606e73d2056c15a69fbe16963a09e1ac0a4fcbf922d747d8f29e708f241f565b5a18832a65ff7e41a7ec7ec8b903d7ce05cf298beac641d1c94d8f8eeb7c3622b84a50dfb8df3db8d121ebda13838104f129150d8e8f07804295d30e59e184c4f4b007e3e62420a4fc8e293144f38f828de4ff74c888589252d1de11bc017fc772a183240f682").expect("Invalid hex string");
        assert!(PacketFlow::try_from(packet.as_slice()).is_ok());
    }
}
