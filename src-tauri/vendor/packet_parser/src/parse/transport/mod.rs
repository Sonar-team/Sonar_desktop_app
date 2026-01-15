// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::convert::TryFrom;

pub mod protocols;

use protocols::{TransportProtocol, tcp::TcpPacket, udp::UdpPacket};
use serde::Serialize;

use crate::errors::transport::TransportError;

/// Represents a transport layer packet (UDP, TCP, etc.)
#[derive(Debug, Clone, Serialize, Eq)]
pub struct Transport<'a> {
    /// The transport layer protocol name
    pub protocol: TransportProtocol,
    /// Source port
    pub source_port: Option<u16>,
    /// Destination port
    pub destination_port: Option<u16>,
    /// The payload of the transport packet
    #[serde(skip_serializing)]
    pub payload: Option<&'a [u8]>,
}

impl<'a> Transport<'a> {
    pub fn transport_from_u8(protocol: &u8) -> TransportProtocol {
        TransportProtocol::from_u8(*protocol)
    }
}

impl<'a> TryFrom<&'a [u8]> for Transport<'a> {
    type Error = TransportError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        // First try to parse as TCP (most common case)
        if let Ok(tcp_packet) = TcpPacket::try_from(packet) {
            return Ok(Transport {
                protocol: TransportProtocol::Tcp,
                source_port: Some(tcp_packet.header.source_port),
                destination_port: Some(tcp_packet.header.destination_port),
                payload: Some(tcp_packet.payload),
            });
        }

        // TODO: Add other protocol parsers here (UDP, etc.)
        if let Ok(udp_packet) = UdpPacket::try_from(packet) {
            return Ok(Transport {
                protocol: TransportProtocol::Udp,
                source_port: Some(udp_packet.source_port),
                destination_port: Some(udp_packet.destination_port),
                payload: Some(udp_packet.payload),
            });
        }
        // If we get here, no parser could handle the packet
        Err(TransportError::UnsupportedProtocol)
    }
}

impl<'a> PartialEq for Transport<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.protocol == other.protocol
            && self.source_port == other.source_port
            && self.destination_port == other.destination_port
    }
}
use std::hash::{Hash, Hasher};

impl<'a> Hash for Transport<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.protocol.hash(state);
        self.source_port.hash(state);
        self.destination_port.hash(state);
    }
}
