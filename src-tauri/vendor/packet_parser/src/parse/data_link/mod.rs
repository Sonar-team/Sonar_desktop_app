// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

// parsed_packet/data_link/mod.rs

//! The `DataLink` module provides functionality to parse and analyze data link layer packets,
//! specifically Ethernet frames. It extracts MAC addresses, Ethertype, and the payload from
//! a raw byte slice.
//!
//! # Overview
//!
//! The `DataLink` structure represents an Ethernet frame with the following fields:
//! - `destination_mac`: The destination MAC address of the packet as a string.
//! - `source_mac`: The source MAC address of the packet as a string.
//! - `ethertype`: The Ethertype value, which indicates the protocol used in the payload.
//! - `payload`: The remaining packet data after the Ethernet header.
//!
//! This module includes:
//! - A `TryFrom<&[u8]>` implementation to parse an Ethernet frame from a raw byte slice.
//! - A validation step to ensure the packet length is sufficient before parsing.
//!
//! # Example
//!
//! ```rust
//! use packet_parser::parse::data_link::DataLink;
//!
//! let raw_packet: [u8; 18] = [
//!     0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
//!     0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, // Source MAC
//!     0x08, 0x00, // Ethertype (IPv4)
//!     0x45, 0x00, 0x00, 0x54, // Payload (IPv4 Header fragment)
//! ];
//!
//! let datalink = DataLink::try_from(raw_packet.as_ref()).expect("Failed to parse valid packet");
//! println!("{:?}", datalink);
//! ```
//!
//! # Errors
//!
//! The `TryFrom<&[u8]>` implementation can return a `DataLinkError` if:
//! - The packet is too short to contain a valid Ethernet frame.
//! - The MAC addresses or Ethertype are invalid.
//!
//! # See Also
//! - [`MacAddress`]
//! - [`Ethertype`]

pub mod mac_addres;
use mac_addres::MacAddress;
use serde::Serialize;

pub mod ethertype;
pub mod vlan_tag;

use crate::{
    checks::data_link::validate_data_link_length, errors::data_link::DataLinkError,
    parse::data_link::vlan_tag::VlanTag,
};

use ethertype::Ethertype;

/// Represents a parsed Ethernet frame, containing source and destination MAC addresses,
/// an Ethertype, and the payload.
#[derive(Debug, Clone, Serialize, Eq)]
pub struct DataLink<'a> {
    /// The destination MAC address as a string.
    pub destination_mac: String,
    /// The source MAC address as a string.
    pub source_mac: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<VlanTag>,
    /// The Ethertype of the packet, indicating the protocol in the payload.
    pub ethertype: String,
    /// The payload of the Ethernet frame.
    #[serde(skip_serializing)]
    pub payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for DataLink<'a> {
    type Error = DataLinkError;

    fn try_from(packets: &'a [u8]) -> Result<Self, Self::Error> {
        validate_data_link_length(packets)?;

        let destination_mac = MacAddress::try_from(&packets[0..6])?;
        let source_mac = MacAddress::try_from(&packets[6..12])?;

        // EtherType brut (peut être 0x8100 pour VLAN)
        let raw_ethertype = u16::from_be_bytes([packets[12], packets[13]]);

        let mut vlan: Option<VlanTag> = None;
        let ethertype: String;
        let payload: &'a [u8];

        if raw_ethertype == 0x8100 {
            // On a un tag 802.1Q.
            // Il nous faut au minimum 18 octets : header Ethernet + TCI + inner EtherType
            if packets.len() < 18 {
                return Err(DataLinkError::DataLinkTooShort(packets.len() as u8));
            }

            // TCI + inner EtherType : [14..18]
            let vlan_tag = VlanTag::try_from(&packets[14..18])?;

            ethertype = vlan_tag.inner_ethertype_name().to_string();
            payload = &packets[18..];
            vlan = Some(vlan_tag);
        } else {
            // Pas de VLAN
            ethertype = Ethertype::from(raw_ethertype).name().to_string();
            payload = &packets[14..];
        }

        Ok(DataLink {
            destination_mac: destination_mac.display_with_oui(),
            source_mac: source_mac.display_with_oui(),
            vlan,
            ethertype,
            payload,
        })
    }
}

impl<'a> PartialEq for DataLink<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.destination_mac == other.destination_mac
            && self.source_mac == other.source_mac
            && self.vlan == other.vlan
            && self.ethertype == other.ethertype
    }
}

use std::hash::{Hash, Hasher};

impl<'a> Hash for DataLink<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.destination_mac.hash(state);
        self.source_mac.hash(state);
        self.vlan.hash(state);
        self.ethertype.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use crate::errors::data_link::DataLinkError;
    use crate::parse::data_link::DataLink;
    use crate::parse::data_link::mac_addres::MacAddress;

    #[test]
    fn test_datalink_try_from_valid_packet() {
        let raw_packet: [u8; 18] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
            0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, // Source MAC
            0x08, 0x00, // Ethertype (IPv4)
            0x45, 0x00, 0x00, 0x54, // Payload (IPv4 Header fragment)
        ];

        let datalink =
            DataLink::try_from(raw_packet.as_ref()).expect("Failed to parse valid packet");

        assert_eq!(
            datalink.destination_mac,
            MacAddress::try_from(&raw_packet[0..6])
                .unwrap()
                .display_with_oui()
        );
        assert_eq!(
            datalink.source_mac,
            MacAddress::try_from(&raw_packet[6..12])
                .unwrap()
                .display_with_oui()
        );
        assert_eq!(datalink.ethertype, "IPv4"); // IPv4 Ethertype
    }

    #[test]
    fn test_datalink_try_from_invalid_length() {
        let short_packet: [u8; 10] = [0x00; 10];

        let result = DataLink::try_from(short_packet.as_ref());
        assert!(matches!(result, Err(DataLinkError::DataLinkTooShort(_))));
    }

    #[test]
    fn test_datalink_try_from_ethertype_parsing() {
        let raw_packet: [u8; 18] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
            0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, // Source MAC
            0x86, 0xDD, // Ethertype (IPv6)
            0x60, 0x00, 0x00, 0x00, // IPv6 Header fragment
        ];

        let datalink = DataLink::try_from(raw_packet.as_ref()).unwrap();
        assert_eq!(datalink.ethertype, "IPv6"); // IPv6 Ethertype
    }

    #[test]
    fn test_datalink_try_from_ethertype_unknown() {
        let raw_packet: [u8; 18] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
            0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, // Source MAC
            0xAB, 0xCD, // Inconnu Ethertype
            0x12, 0x34, 0x56, 0x78, // Payload quelconque
        ];

        let datalink = DataLink::try_from(raw_packet.as_ref()).unwrap();
        assert_eq!(datalink.ethertype, "Unknown (0xABCD)"); // Ethertype inconnu, mais accepté
    }
    #[test]
    fn test_datalink_try_from_empty_payload() {
        let raw_packet: [u8; 14] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Destination MAC
            0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, // Source MAC
            0xAB, 0xCD, // Inconnu Ethertype
        ];

        let datalink = DataLink::try_from(raw_packet.as_ref()).unwrap();
        assert_eq!(datalink.ethertype, "Unknown (0xABCD)"); // Ethertype inconnu, mais accepté
    }

    #[test]
    fn test_datalink_try_from_vlan_tagged() {
        // Dest MAC, Src MAC, TPID=0x8100, TCI (PCP=0, DEI=0, VLAN 10), inner EtherType=0x0800 (IPv4)
        let raw_packet: [u8; 22] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Dest
            0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, // Src
            0x81, 0x00, // TPID 802.1Q
            0x00, 0x0A, // TCI : VLAN 10
            0x08, 0x00, // Inner EtherType : IPv4
            0x45, 0x00, 0x00, 0x54, // Début header IPv4
        ];

        let datalink = DataLink::try_from(raw_packet.as_ref()).unwrap();

        assert_eq!(datalink.ethertype, "IPv4");
        assert!(datalink.vlan.is_some());
        let vlan = datalink.vlan.unwrap();
        assert_eq!(vlan.id, 10);
        assert_eq!(datalink.payload, &raw_packet[18..]);
    }
}
