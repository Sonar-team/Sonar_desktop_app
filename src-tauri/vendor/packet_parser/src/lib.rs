// Copyright (c) 2024 Cyprien Avico <avicocyprien@yahoo.com>
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! # Packet Parser
//!
//! **Packet Parser** is a modular Rust library designed to analyze and decode raw network packets.
//!
//! This crate allows processing different layers of a network packet, starting from the data link layer
//! (Ethernet II) and moving down through the network, transport, and application layers.
//!
//! ## Features
//! - **Multi-layer analysis**: link, internet, transport and application layers.
//! - **Zero-copy**: [`PacketFlow`] borrows the input buffer; no payload is copied.
//! - **Fail-closed on the link layer**: the LINKTYPE is supplied by the caller,
//!   never guessed from the bytes.
//! - **Fail-soft above it**: an unsupported or malformed upper layer leaves that
//!   layer `None` instead of failing the whole parse.
//! - **Modular architecture**: easily extendable to support new protocols.
//!
//! ## Usage
//!
//! [`fn@parse`] is the entry point. It takes the LINKTYPE of the capture — the
//! value stored in the PCAP/PCAPNG file — and exactly one packet, without any
//! record header.
//!
//! ```rust
//! use packet_parser::{LinkType, parse};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let frame = hex::decode(
//!     "feaa81e86d1efeaa818ec864080045500034000000003d06206b36e6700d\
//!      ac140a0201bbc1087d7f02aa4e2b998e80100081748300000101080a9373\
//!      c9c207ef14e3",
//! )?;
//!
//! let flow = parse(LinkType::ETHERNET, &frame)?;
//!
//! println!("L2: {}", flow.data_link);
//!
//! if let Some(internet) = &flow.internet {
//!     println!("L3: {} {:?} -> {:?}",
//!         internet.protocol_name, internet.source, internet.destination);
//! }
//! if let Some(transport) = &flow.transport {
//!     println!("L4: {:?} {:?} -> {:?}",
//!         transport.protocol, transport.source_port, transport.destination_port);
//! }
//! if let Some(application) = &flow.application {
//!     println!("L7: {}", application.application_protocol);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Use [`is_supported`] to check a LINKTYPE before feeding a whole capture:
//!
//! ```rust
//! use packet_parser::{LinkType, is_supported};
//!
//! assert!(is_supported(LinkType::ETHERNET));
//! assert!(is_supported(LinkType::LINUX_SLL));
//! assert!(!is_supported(LinkType(0xdead)));
//! ```
//!
//! ### Do not guess the LINKTYPE
//!
//! [`PacketFlow::try_from`] and [`DataLink::try_from`] accept a bare `&[u8]` and
//! **assume [`LinkType::ETHERNET`]**. They are kept for compatibility and will be
//! removed in a future major release.
//!
//! Feeding them a non-Ethernet capture does not fail — it silently produces
//! wrong data. A `LINKTYPE_LINUX_SLL` frame (what a capture on the `any`
//! interface yields) has its 16 cooked-header bytes reinterpreted as an Ethernet
//! header, so the parse returns `Ok` with fabricated MAC addresses,
//! `internet: None` and `corrupted: None`. Nothing signals the mistake.
//!
//! Always pass the LINKTYPE the capture actually declares.
//!
//! ## What a parse returns
//!
//! ```text
//! PacketFlow<'a>
//! ├── data_link:   LinkLayer<'a>              (mandatory — Ethernet, SLL, SLL2, RAW)
//! ├── internet:    Option<Internet<'a>>       source / destination / protocol_name
//! │                                           / payload_protocol / payload / details
//! ├── transport:   Option<Transport<'a>>      protocol / source_port / destination_port
//! │                                           / payload / details
//! ├── application: Option<Application>        application_protocol: &'static str
//! ├── inner:       Option<Box<PacketFlow<'a>>>  packet carried inside a tunnel
//! └── corrupted:   Option<CorruptedLayer>     set when a recognized layer held
//!                                             invalid bytes
//! ```
//!
//! The application layer is a **classification, not a decode**: it reports the
//! detected protocol name, not a parsed message.
//!
//! ## Reading the outcome
//!
//! Only the link layer can fail the parse: an unsupported LINKTYPE yields
//! `Err(ParseError)`. Above it, three outcomes are distinct and must not be
//! confused:
//!
//! - **layer present** — it was recognized and decoded;
//! - **layer `None`, `corrupted` `None`** — the protocol is not supported, and
//!   nothing above it could be reached;
//! - **layer `None`, `corrupted` `Some`** — the layer *was* recognized but
//!   carried invalid bytes. [`CorruptedLayer`] names which one.
//!
//! ## Owned flows
//!
//! [`PacketFlow`] borrows the input buffer. To store a flow, send it across
//! threads or serialize it, convert it with [`PacketFlow::to_owned`], which
//! returns an [`owned::PacketFlowOwned`]. Note that the owned form drops the
//! payloads and the per-layer `details`.

/// Module handling format and integrity checks for packets.
pub mod checks;

/// Module for converting packet formats.
pub mod convert;

pub mod owned;

mod link_type;
pub use link_type::LinkType;

/// Module for displaying parsed data (internal use).
mod displays;

/// Centralized error management for the crate. Public so consumers can name
/// and match the per-layer error types (e.g.
/// `errors::internet::InternetError`, `errors::transport::TransportError`).
pub mod errors;
pub use errors::{LinkLayerError, ParseError, ParsedPacketError};

/// Main module for packet analysis.
pub mod parse;
#[cfg(feature = "parse_timing")]
pub use parse::parse_timed;
pub use parse::{is_supported, parse};

pub use parse::application::Application;
/// Exports data link layer parsing functionality.
pub use parse::data_link::DataLink;
pub use parse::data_link::mac_addres::MacAddress;
pub use parse::internet::dscp_ecn::{Dscp, Ecn};
pub use parse::internet::ip_type::IpType;
pub use parse::internet::{Internet, InternetDetails};
pub use parse::link_layer::{
    Ieee80211Link, LinkLayer, LinkLayerKind, LinuxArphrdType, LinuxCookedPacketType, LinuxSll2Link,
    LinuxSllLink, NetworkProtocol, RawIpLink,
};
pub use parse::transport::{Transport, TransportDetails};

/// Exports data link layer parsing functionality.
pub use parse::PacketFlow;
pub use parse::{CorruptedLayer, CorruptedLayerKind};

pub mod timing;
