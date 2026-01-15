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
//! - **Multi-layer analysis**: Supports data link, network, transport, and application layers.
//! - **Error management**: Detailed error handling to facilitate debugging.
//! - **Packet validation**: Built-in verification mechanisms to ensure data integrity.
//! - **Modular architecture**: Easily extendable to support new protocols.
//!
//! ## Usage Example
//!
//! ```rust
//! use packet_parser::DataLink;
//! use hex::decode;
//!
//! let hex_dump_data = "feaa81e86d1efeaa818ec864080045500034000000003d06206b36e6700dac140a0201bbc1087d7f02aa4e2b998e80100081748300000101080a9373c9c207ef14e3";
//! let packet = decode(hex_dump_data).expect("Hexadecimal conversion failed");
//!
//! match DataLink::try_from(packet.as_slice()) {
//!     Ok(datalink) => println!("{:?}", datalink),
//!     Err(e) => eprintln!("Parsing error: {:?}", e),
//! }
//! ```
//! ## Packet Structure
//!
//! The library parses packets into a hierarchical structure that can be represented as:
//!
//! ```text
//! Packet
//! ├── DataLink (Ethernet II)
//! │   ├── source_mac: String
//! │   ├── destination_mac: String
//! │   └── ether_type: EtherType
//! │
//! ├── Network (IPv4/IPv6)
//! │   ├── source_ip: IpAddr
//! │   ├── destination_ip: IpAddr
//! │   ├── protocol: IpProtocol
//! │   └── ttl: u8
//! │
//! ├── Transport (TCP/UDP/ICMP)
//! │   ├── source_port: Option<u16>
//! │   ├── destination_port: Option<u16>
//! │   └── flags: TransportFlags
//! │
//! └── Application
//!     ├── protocol: ApplicationProtocol
//!     └── payload: Vec<u8>
//! ```
//!
//! ## Example Flattened Struct
//!
//! For easier access to all fields, you can use a flattened structure:
//!
//! ```rust
//! use std::net::IpAddr;
//! use pnet::packet::ethernet::EtherType;
//! use pnet::packet::ip::IpNextHeaderProtocol as IpProtocol;
//! use packet_parser::parse::application::protocols::ApplicationProtocol;
//!
//! pub struct FlattenedPacket<'a> {
//!     // Data Link Layer (Ethernet)
//!     pub source_mac: String,
//!     pub destination_mac: String,
//!     pub ether_type: EtherType,
//!     
//!     // Network Layer
//!     pub source_ip: IpAddr,
//!     pub destination_ip: IpAddr,
//!     pub ip_protocol: IpProtocol,
//!     pub ttl: u8,
//!     
//!     // Transport Layer
//!     pub source_port: Option<u16>,
//!     pub destination_port: Option<u16>,
//!     pub transport_flags: u16,  // Using u16 as a simple representation of flags
//!     
//!     // Application Layer
//!     pub application_protocol: Option<ApplicationProtocol<'a>>,
//!     pub payload: Vec<u8>,
//! }
//! ```
//!
//! This flattened structure provides direct access to all packet fields without
//! having to navigate through multiple layers of nested enums and structs.

/// Module handling format and integrity checks for packets.
pub mod checks;

/// Module for converting packet formats.
pub mod convert;

pub mod owned;

/// Module for displaying parsed data (internal use).
mod displays;

/// Centralized error management for the crate.
mod errors;
pub use errors::ParsedPacketError;

/// Main module for packet analysis.
pub mod parse;

/// Exports data link layer parsing functionality.
pub use parse::data_link::DataLink;

/// Exports MAC address parsing functionality.
pub use parse::data_link::mac_addres::MacAddress;

pub use parse::application::Application;
pub use parse::internet::Internet;
pub use parse::internet::ip_type::IpType;
pub use parse::transport::Transport;

/// Exports data link layer parsing functionality.
pub use parse::PacketFlow;
