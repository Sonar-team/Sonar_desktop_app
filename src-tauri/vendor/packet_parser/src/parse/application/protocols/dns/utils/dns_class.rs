// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

const MDNS_CLASS_FLAG: u16 = 0x8000;
const MDNS_CLASS_MASK: u16 = !MDNS_CLASS_FLAG;

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod DnsClasses {
    use super::DnsClass;

    pub const IN: DnsClass = DnsClass(1);
    pub const CS: DnsClass = DnsClass(2);
    pub const CH: DnsClass = DnsClass(3);
    pub const HS: DnsClass = DnsClass(4);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DnsClass(pub u16);

impl DnsClass {
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Strip the mDNS-only high bit from the actual 15-bit DNS class.
    ///
    /// RFC 6762 uses the top bit as QU in questions and cache-flush in
    /// resource records. Classic DNS must keep using [`DnsClass::new`], where
    /// all 16 bits remain part of the class value.
    pub(crate) fn from_mdns(value: u16) -> Self {
        Self(value & MDNS_CLASS_MASK)
    }
}

impl fmt::Display for DnsClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                DnsClasses::IN => "IN", // 1
                DnsClasses::CS => "CS", // 2
                DnsClasses::CH => "CH", // 3
                DnsClasses::HS => "HS", // 4
                _ => "unknown",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wraps_value() {
        assert_eq!(DnsClass::new(1), DnsClasses::IN);
        assert_eq!(DnsClass::new(42).0, 42);
        assert_eq!(DnsClass::new(0x8001), DnsClass(0x8001));
    }

    #[test]
    fn test_from_mdns_strips_the_class_flag() {
        assert_eq!(DnsClass::from_mdns(0x8001), DnsClasses::IN);
        assert_eq!(DnsClass::from_mdns(0x0001), DnsClasses::IN);
    }

    #[test]
    fn test_display_known_classes() {
        assert_eq!(DnsClasses::IN.to_string(), "IN");
        assert_eq!(DnsClasses::CS.to_string(), "CS");
        assert_eq!(DnsClasses::CH.to_string(), "CH");
        assert_eq!(DnsClasses::HS.to_string(), "HS");
    }

    #[test]
    fn test_display_unknown_class() {
        assert_eq!(DnsClass(0).to_string(), "unknown");
        assert_eq!(DnsClass(255).to_string(), "unknown");
    }
}
