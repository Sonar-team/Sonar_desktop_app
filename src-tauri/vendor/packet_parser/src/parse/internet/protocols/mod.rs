pub mod arp;
pub mod ipv4;
pub mod ipv6;
pub mod profinet;

#[derive(Debug)]
pub enum InternetProtocolType {
    Arp,
    Ipv4,
    Ipv6,
    Profinet,
    Unknown(u8),
}
