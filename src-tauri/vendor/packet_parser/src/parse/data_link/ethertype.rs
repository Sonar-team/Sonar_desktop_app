// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::{Serialize, Serializer};

// ethertype.rs
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Hash)]
pub struct Ethertype(pub u16);

impl Ethertype {
    pub fn from(code: u16) -> Self {
        Ethertype(code)
    }

    /// Nom statique de l'Ethertype s'il est connu, sans allocation.
    pub fn static_name(&self) -> Option<&'static str> {
        match self.0 {
            0x0800 => Some("IPv4"),
            0x86DD => Some("IPv6"),
            0x0806 => Some("ARP"),
            0x8100 => Some("VLAN-tagged frame"),
            0x88CC => Some("LLDP"),
            0x8892 => Some("Profinet"),
            0x88E3 => Some("MRP"),
            0x88F7 => Some("PTP"),
            0x9100 => Some("Q-in-Q"),
            0x88A8 => Some("PBridge"),
            0x22F3 => Some("Trill"),
            0x6003 => Some("DECnet"),
            0x8035 => Some("Rarp"),
            0x809B => Some("AppleTalk"),
            0x80F3 => Some("Aarp"),
            0x8137 => Some("Ipx"),
            0x8204 => Some("Qnx"),
            0x8847 => Some("MPLS Unicast"),
            0x8848 => Some("MPLS Multicast"),
            0x8863 => Some("Pppoe Discovery Stage"),
            0x8864 => Some("Pppoe Session Stage"),
            0x8819 => Some("CobraNet"),
            0x8902 => Some("cfm"),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        match self.static_name() {
            Some(name) => name.to_string(),
            None => format!("Unknown (0x{:04X})", self.0),
        }
    }
}

/// Sérialise un Ethertype sous forme de nom lisible ("IPv4", "Unknown (0xABCD)"…),
/// comme l'ancien champ `String` de `DataLink`.
pub fn serialize_name<S: Serializer>(
    ethertype: &Ethertype,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match ethertype.static_name() {
        Some(name) => serializer.serialize_str(name),
        None => serializer.collect_str(&format_args!("Unknown (0x{:04X})", ethertype.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::Ethertype;

    #[test]
    fn test_ethertype_creation() {
        let ethertype = Ethertype::from(0x0800);
        assert_eq!(ethertype, Ethertype(0x0800));
    }

    #[test]
    fn test_ethertype_known_values() {
        let test_cases = vec![
            (0x0800, "IPv4"),
            (0x86DD, "IPv6"),
            (0x0806, "ARP"),
            (0x8100, "VLAN-tagged frame"),
            (0x88CC, "LLDP"),
            (0x8892, "Profinet"),
            (0x88E3, "MRP"),
            (0x88F7, "PTP"),
            (0x9100, "Q-in-Q"),
            (0x88A8, "PBridge"),
            (0x22F3, "Trill"),
            (0x6003, "DECnet"),
            (0x8035, "Rarp"),
            (0x809B, "AppleTalk"),
            (0x80F3, "Aarp"),
            (0x8137, "Ipx"),
            (0x8204, "Qnx"),
            (0x8847, "MPLS Unicast"),
            (0x8848, "MPLS Multicast"),
            (0x8863, "Pppoe Discovery Stage"),
            (0x8864, "Pppoe Session Stage"),
            (0x8819, "CobraNet"),
            (0x8902, "cfm"),
        ];

        for (code, expected_name) in test_cases {
            let ethertype = Ethertype::from(code);
            assert_eq!(
                ethertype.name(),
                expected_name,
                "Failed for Ethertype: {:#06X}",
                code
            );
        }
    }

    #[test]
    fn test_ethertype_unknown() {
        let unknown_ethertype = Ethertype::from(0xFFFF); // Random unknown value
        assert_eq!(unknown_ethertype.name(), "Unknown (0xFFFF)");
    }
}
