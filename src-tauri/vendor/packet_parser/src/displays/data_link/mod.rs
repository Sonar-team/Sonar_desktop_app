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
