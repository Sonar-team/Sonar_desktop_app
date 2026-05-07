// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::Serialize;

// ethertype.rs
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Hash)]
pub struct Ethertype(pub u16);

impl Ethertype {
    pub fn from(code: u16) -> Self {
        Ethertype(code)
    }

    pub fn name(&self) -> String {
        match self.0 {
            0x0800 => "IPv4".to_string(),
            0x86DD => "IPv6".to_string(),
            0x0806 => "ARP".to_string(),
            0x8100 => "VLAN-tagged frame".to_string(),
            0x88CC => "LLDP".to_string(),
            0x8892 => "Profinet".to_string(),
            0x88E3 => "MRP".to_string(),
            0x88F7 => "PTP".to_string(),
            0x9100 => "Q-in-Q".to_string(),
            0x88A8 => "PBridge".to_string(),
            0x22F3 => "Trill".to_string(),
            0x6003 => "DECnet".to_string(),
            0x8035 => "Rarp".to_string(),
            0x809B => "AppleTalk".to_string(),
            0x80F3 => "Aarp".to_string(),
            0x8137 => "Ipx".to_string(),
            0x8204 => "Qnx".to_string(),
            0x8847 => "MPLS Unicast".to_string(),
            0x8848 => "MPLS Multicast".to_string(),
            0x8863 => "Pppoe Discovery Stage".to_string(),
            0x8864 => "Pppoe Session Stage".to_string(),
            0x8819 => "CobraNet".to_string(),
            0x8902 => "cfm".to_string(),
            _ => format!("Unknown (0x{:04X})", self.0),
        }
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
