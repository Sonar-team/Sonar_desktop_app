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
            self.ethertype,
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

    #[test]
    fn test_datalink_display_without_vlan() {
        let data_link = DataLink {
            destination_mac: "AA:BB:CC:DD:EE:FF".to_string(),
            source_mac: "11:22:33:44:55:66".to_string(),
            ethertype: "IPv4".to_string(),
            vlan: None,
            payload: &[0x01, 0x02, 0x03, 0x04],
        };

        let expected = concat!(
            "\n    Destination MAC: AA:BB:CC:DD:EE:FF,\n",
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
            destination_mac: "FF:FF:FF:FF:FF:FF".to_string(),
            source_mac: "00:00:00:00:00:00".to_string(),
            ethertype: "ARP".to_string(),
            vlan: None,
            payload: &[],
        };

        let expected = concat!(
            "\n    Destination MAC: FF:FF:FF:FF:FF:FF,\n",
            "    Source MAC: 00:00:00:00:00:00,\n",
            "    Ethertype: ARP,\n",
            "    VLAN: None,\n",
            "    Payload Length: 0\n"
        );

        assert_eq!(data_link.to_string(), expected);
    }
}
