use std::net::IpAddr;

use serde::Serialize;
// Définition de l'énumération `IpType`
#[derive(Debug, Serialize, Clone, Eq, Hash, PartialEq)]
#[derive(Default)]
pub enum IpType {
    Private,
    Multicast,
    Loopback,
    Apipa,
    LinkLocal,
    Ula,
    Public,
    #[default]
    Unknown,
}

// Implémentation des méthodes pour `IpType`
impl IpType {
    pub fn from_ip(ip: &str) -> Self {
        match ip.parse::<IpAddr>() {
            Ok(IpAddr::V4(ipv4_addr)) if ipv4_addr.is_private() => Self::Private,
            Ok(IpAddr::V4(ipv4_addr)) if ipv4_addr.is_loopback() => Self::Loopback,
            Ok(IpAddr::V4(ipv4_addr)) if is_apipa_ip(&ipv4_addr) => Self::Apipa,
            Ok(IpAddr::V4(ipv4_addr)) if ipv4_addr.is_multicast() => Self::Multicast,
            Ok(IpAddr::V6(ipv6_addr)) if ipv6_addr.is_multicast() => Self::Multicast,
            Ok(IpAddr::V6(ipv6_addr)) if ipv6_addr.is_loopback() => Self::Loopback,
            Ok(IpAddr::V6(ipv6_addr)) if is_ipv6_unicast_link_local(&ipv6_addr) => Self::LinkLocal,
            Ok(IpAddr::V6(ipv6_addr)) if is_ula(&ipv6_addr) => Self::Ula,
            Ok(_) => Self::Public, // Cette ligne devrait être la dernière condition pour IPv6
            Err(_) => Self::Unknown,
        }
    }
}

// Implémenter Default pour IpType


// Fonctions auxiliaires pour les vérifications spécifiques
fn is_apipa_ip(ip: &std::net::Ipv4Addr) -> bool {
    ip.octets()[0] == 169 && ip.octets()[1] == 254
}

fn is_ipv6_unicast_link_local(ip: &std::net::Ipv6Addr) -> bool {
    ip.segments()[0] == 0xfe80
}

fn is_ula(ip: &std::net::Ipv6Addr) -> bool {
    let first_byte = ip.octets()[0];
    first_byte & 0xfe == 0xfc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apipa_ipv4() {
        let ip = "169.254.1.1";
        assert_eq!(IpType::from_ip(ip), IpType::Apipa);
    }

    // Reprenons les tests pour les adresses privées, publiques et spéciales,
    // tout en s'assurant qu'ils utilisent la nouvelle logique.
    #[test]
    fn test_private_ipv4() {
        assert_eq!(IpType::from_ip("192.168.1.1"), IpType::Private);
        assert_eq!(IpType::from_ip("10.0.0.1"), IpType::Private);
        assert_eq!(IpType::from_ip("172.16.0.1"), IpType::Private);
    }

    #[test]
    fn test_public_ipv4() {
        assert_eq!(IpType::from_ip("8.8.8.8"), IpType::Public); // Google DNS
        assert_eq!(IpType::from_ip("1.1.1.1"), IpType::Public); // Cloudflare DNS
    }

    #[test]
    fn test_invalid_ipv4() {
        assert_eq!(IpType::from_ip("999.999.999.999"), IpType::Unknown);
        assert_eq!(IpType::from_ip("abcd"), IpType::Unknown);
    }

    #[test]
    fn test_ipv6_multicast() {
        assert_eq!(IpType::from_ip("ff02::1"), IpType::Multicast);
    }

    #[test]
    fn test_ipv6_unicast_link_local() {
        assert_eq!(IpType::from_ip("fe80::1"), IpType::LinkLocal);
    }

    #[test]
    fn test_ipv6_ula() {
        assert_eq!(IpType::from_ip("fd00::1"), IpType::Ula);
    }

    #[test]
    fn test_ipv6_public() {
        assert_eq!(
            IpType::from_ip("2001:0db8:85a3:0000:0000:8a2e:0370:7334"),
            IpType::Public
        );
    }

    #[test]
    fn test_ipv6_loopback() {
        assert_eq!(IpType::from_ip("::1"), IpType::Loopback);
    }

    #[test]
    fn test_ipv4_multicast() {
        assert_eq!(IpType::from_ip("224.0.0.1"), IpType::Multicast); // Adresse multicast de base
        assert_eq!(IpType::from_ip("239.255.255.255"), IpType::Multicast); // Fin de la plage multicast
    }
}
