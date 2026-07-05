use core::fmt;
use std::fmt::{Display, Formatter};

use crate::parse::data_link::vlan_tag::VlanTag;

impl Display for VlanTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ID: {:02x}, PCP: {:02x}, DEI: {}",
            self.id, self.pcp, self.dei
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::data_link::ethertype::Ethertype;
    use crate::parse::data_link::vlan_tag::VlanTag;

    #[test]
    fn test_vlan_tag_display() {
        let vlan = VlanTag {
            id: 100,
            pcp: 5,
            dei: true,
            inner_ethertype: Ethertype(0x0800),
        };

        assert_eq!(vlan.to_string(), "ID: 64, PCP: 05, DEI: true");
    }
}
