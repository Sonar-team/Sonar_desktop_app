use std::fmt::{Display, Formatter};

use crate::parse::data_link::ethertype::Ethertype;

impl Display for Ethertype {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X} ({})", self.0, self.name())
    }
}

#[cfg(test)]
mod tests {

    use crate::parse::data_link::ethertype::Ethertype;

    #[test]
    fn test_ethertype_display() {
        let test_cases = vec![
            (0x0800, "0x0800 (IPv4)"),
            (0x86DD, "0x86DD (IPv6)"),
            (0x0806, "0x0806 (ARP)"),
            (0x8100, "0x8100 (VLAN-tagged frame)"),
            (0x88CC, "0x88CC (LLDP)"),
            (0x8892, "0x8892 (Profinet)"),
            (0x88E3, "0x88E3 (MRP)"),
            (0x88F7, "0x88F7 (PTP)"),
            (0x9100, "0x9100 (Q-in-Q)"),
            (0x88A8, "0x88A8 (PBridge)"),
            (0x22F3, "0x22F3 (Trill)"),
            (0x6003, "0x6003 (DECnet)"),
            (0x8035, "0x8035 (Rarp)"),
            (0x809B, "0x809B (AppleTalk)"),
            (0x80F3, "0x80F3 (Aarp)"),
            (0x8137, "0x8137 (Ipx)"),
            (0x8204, "0x8204 (Qnx)"),
            (0x8847, "0x8847 (MPLS Unicast)"),
            (0x8848, "0x8848 (MPLS Multicast)"),
            (0x8863, "0x8863 (Pppoe Discovery Stage)"),
            (0x8864, "0x8864 (Pppoe Session Stage)"),
            (0x8819, "0x8819 (CobraNet)"),
            (0x8902, "0x8902 (cfm)"),
            (0xFFFF, "0xFFFF (Unknown (0xFFFF))"), // Updated to show hex value for unknown
        ];

        for (code, expected_output) in test_cases {
            let ethertype = Ethertype::from(code);
            assert_eq!(
                ethertype.to_string(),
                expected_output,
                "Failed for Ethertype: {:#06X}",
                code
            );
        }
    }
}
