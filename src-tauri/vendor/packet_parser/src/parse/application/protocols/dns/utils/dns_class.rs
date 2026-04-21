use std::fmt;

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
