use std::fmt;
pub mod ethertype;
pub mod mac_addres;
pub mod oui;
pub mod vlan;
use crate::parse::data_link::DataLink;

impl fmt::Display for DataLink<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n    Destination MAC: {},\n    Source MAC: {},\n    Ethertype: {},\n    VLAN: {},\n    Payload Length: {}\n",
            self.destination_mac,
            self.source_mac,
            self.ethertype.name(),
            match &self.vlan {
                Some(vlan) => vlan.to_string(),
                None => "None".to_string(),
            },
            self.payload.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::data_link::DataLink;
    use crate::parse::data_link::ethertype::Ethertype;
    use crate::parse::data_link::mac_addres::MacAddress;

    #[test]
    fn test_datalink_display_without_vlan() {
        let data_link = DataLink {
            destination_mac: MacAddress([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
            source_mac: MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
            ethertype: Ethertype(0x0800),
            vlan: None,
            payload: &[0x01, 0x02, 0x03, 0x04],
        };

        let expected = concat!(
            "\n    Destination MAC: aa:bb:cc:dd:ee:ff,\n",
            "    Source MAC: 11:22:33:44:55:66,\n",
            "    Ethertype: IPv4,\n",
            "    VLAN: None,\n",
            "    Payload Length: 4\n"
        );

        assert_eq!(data_link.to_string(), expected);
    }

    #[test]
    fn test_datalink_display_with_empty_payload() {
        let data_link = DataLink {
            destination_mac: MacAddress([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
            source_mac: MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            ethertype: Ethertype(0x0806),
            vlan: None,
            payload: &[],
        };

        let expected = concat!(
            "\n    Destination MAC: ff:ff:ff:ff:ff:ff,\n",
            "    Source MAC: 00:00:00:00:00:00,\n",
            "    Ethertype: ARP,\n",
            "    VLAN: None,\n",
            "    Payload Length: 0\n"
        );

        assert_eq!(data_link.to_string(), expected);
    }
}
