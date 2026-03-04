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
