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
