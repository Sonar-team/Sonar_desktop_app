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
///
/// ## Layer mapping
///
/// | Field        | OSI Layer | Description                         |
/// |--------------|-----------|-------------------------------------|
/// | `data_link`  | L2        | Ethernet / VLAN / etc.              |
/// | `internet`   | L3        | IPv4 / IPv6 (optional)              |
/// | `transport`  | L4        | TCP / UDP / ICMP (optional)         |
/// | `application`| L7        | Best-effort application decoding    |
///
/// ## Error handling
///
/// Parsing stops only on **structural errors**. Unsupported protocols are
/// handled gracefully.
///
/// This behavior makes `PacketFlow` suitable for offline analysis, auditing,
/// and security-oriented tooling.
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

    /// Attempts to parse a raw packet buffer into a [`PacketFlow`].
    ///
    /// Parsing proceeds layer by layer:
    ///
    /// 1. Data Link
    /// 2. Internet (if supported)
    /// 3. Transport (if supported)
    /// 4. Application (best-effort)
    ///
    /// Unsupported protocols do not cause an error and simply stop further
    /// decoding.
    ///
    /// # Errors
    ///
    /// Returns an error only when a layer is structurally invalid or malformed.
    fn try_from(packets: &'a [u8]) -> Result<Self, Self::Error> {
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
}
