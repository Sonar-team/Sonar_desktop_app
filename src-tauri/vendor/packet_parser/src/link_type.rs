// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical `LINKTYPE_*` identifier for the link-layer format of a packet.
///
/// `LinkType` deliberately remains open: unknown values are preserved instead
/// of being collapsed into an `Unknown` enum variant. Values obtained from a
/// live capture as `DLT_*` must be normalized by the caller when their numeric
/// representation differs from the `LINKTYPE_*` value stored in capture files.
/// This keeps `packet_parser` independent from PCAP, PCAPNG, and libpcap.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LinkType(pub u32);

impl LinkType {
    /// Ethernet (LINKTYPE_ETHERNET / DLT_EN10MB).
    pub const ETHERNET: Self = Self(1);

    /// Raw IPv4 or IPv6 packet (LINKTYPE_RAW).
    pub const RAW: Self = Self(101);

    /// Native IEEE 802.11 wireless frame (LINKTYPE_IEEE802_11).
    pub const IEEE802_11: Self = Self(105);

    /// Linux cooked capture v1 (LINKTYPE_LINUX_SLL).
    pub const LINUX_SLL: Self = Self(113);

    /// Bluetooth HCI H4 with a direction pseudo-header.
    pub const BLUETOOTH_HCI_H4_WITH_PHDR: Self = Self(201);

    /// Linux cooked capture v2 (LINKTYPE_LINUX_SLL2).
    pub const LINUX_SLL2: Self = Self(276);
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for LinkType {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<LinkType> for u32 {
    fn from(value: LinkType) -> Self {
        value.0
    }
}

impl From<u16> for LinkType {
    fn from(value: u16) -> Self {
        Self(u32::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::LinkType;

    #[test]
    fn unknown_numeric_value_is_preserved() {
        let link_type = LinkType::from(0xdead_beef_u32);

        assert_eq!(link_type.0, 0xdead_beef);
        assert_eq!(u32::from(link_type), 0xdead_beef);
    }

    #[test]
    fn canonical_constants_keep_their_linktype_values() {
        assert_eq!(LinkType::ETHERNET.0, 1);
        assert_eq!(LinkType::RAW.0, 101);
        assert_eq!(LinkType::IEEE802_11.0, 105);
        assert_eq!(LinkType::LINUX_SLL.0, 113);
        assert_eq!(LinkType::BLUETOOTH_HCI_H4_WITH_PHDR.0, 201);
        assert_eq!(LinkType::LINUX_SLL2.0, 276);
    }

    #[test]
    fn serde_representation_is_the_numeric_linktype() {
        let json = serde_json::to_string(&LinkType::RAW).unwrap();
        assert_eq!(json, "101");
        assert_eq!(
            serde_json::from_str::<LinkType>(&json).unwrap(),
            LinkType::RAW
        );
    }
}
