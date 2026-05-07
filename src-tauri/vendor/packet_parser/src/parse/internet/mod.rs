pub mod protocols;

use std::convert::TryFrom;
use std::net::IpAddr;

use crate::errors::internet::InternetError;
use crate::parse::internet::protocols::profinet;
use crate::parse::transport::protocols::TransportProtocol;
use protocols::arp::ArpPacket;
use protocols::ipv4;
use protocols::ipv6;
use serde::Serialize;
pub mod ip_type;
use super::transport::Transport;
use ip_type::IpType;

#[derive(Debug, Clone, Serialize, Eq)]
pub struct Internet<'a> {
    pub source: Option<IpAddr>,
    pub source_type: Option<IpType>,
    pub destination: Option<IpAddr>,
    pub destination_type: Option<IpType>,
    pub protocol_name: String,
    pub payload_protocol: Option<TransportProtocol>,
    #[serde(skip_serializing)]
    pub payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for Internet<'a> {
    type Error = InternetError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        if packet.is_empty() {
            return Err(InternetError::EmptyPacket);
        }

        // Try to parse as ARP first
        if let Ok(arp_packet) = ArpPacket::try_from(packet) {
            return Ok(Internet {
                source: Some(arp_packet.sender_protocol_addr),
                source_type: Some(IpType::from_ip(
                    &arp_packet.sender_protocol_addr.to_string(),
                )),
                destination: Some(arp_packet.target_protocol_addr),
                destination_type: Some(IpType::from_ip(
                    &arp_packet.target_protocol_addr.to_string(),
                )),
                protocol_name: "ARP".to_string(),
                payload_protocol: None,
                payload: &[],
            });
        }

        if let Ok(ipv4_packet) = ipv4::Ipv4Packet::try_from(packet) {
            return Ok(Internet {
                source: Some(IpAddr::V4(ipv4_packet.source_addr)),
                source_type: Some(IpType::from_ip(&ipv4_packet.source_addr.to_string())),
                destination: Some(IpAddr::V4(ipv4_packet.dest_addr)),
                destination_type: Some(IpType::from_ip(&ipv4_packet.dest_addr.to_string())),
                protocol_name: "IPv4".to_string(),
                payload_protocol: Some(Transport::transport_from_u8(&ipv4_packet.protocol)),
                payload: ipv4_packet.payload,
            });
        }

        if let Ok(ipv6_packet) = ipv6::Ipv6Packet::try_from(packet) {
            return Ok(Internet {
                source: Some(IpAddr::V6(ipv6_packet.source_addr)),
                source_type: Some(IpType::from_ip(&ipv6_packet.source_addr.to_string())),
                destination: Some(IpAddr::V6(ipv6_packet.dest_addr)),
                destination_type: Some(IpType::from_ip(&ipv6_packet.dest_addr.to_string())),
                protocol_name: "IPv6".to_string(),
                payload_protocol: Some(Transport::transport_from_u8(&ipv6_packet.next_header)),
                payload: ipv6_packet.payload,
            });
        }
        if profinet::ProfinetPacket::try_from(packet).is_ok() {
            return Ok(Internet {
                source: None,
                source_type: None,
                destination: None,
                destination_type: None,
                protocol_name: "Profinet".to_string(),
                payload_protocol: None,
                payload: &[],
            });
        }
        Err(InternetError::UnsupportedProtocol)
    }
}

// impl<'a> Internet<'a> {
//     pub fn to_transport(&self) -> Option<Transport<'a>> {
//        let protocol = match self.payload_protocol.as_deref()? {
//             "ICMPv6" => TransportProtocol::IcmpV6,
//             "ICMP" => TransportProtocol::Icmp,
//             "UDP" => TransportProtocol::Udp,
//             "TCP" => TransportProtocol::Tcp,
//             "IGMP" => TransportProtocol::Igmp,
//             "PIM" => TransportProtocol::Pim,
//             "PIMv2" => TransportProtocol::PimV2,
//             "VRRP" => TransportProtocol::Vrrp,
//             // Ajoutez d'autres correspondances si nécessaire
//             _ => return None,
//         };

//         Some(Transport {
//             protocol,
//             source_port: None,
//             destination_port: None,
//             payload: None,
//         })
//     }
// }

impl<'a> PartialEq for Internet<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.source_type == other.source_type
            && self.destination == other.destination
            && self.destination_type == other.destination_type
            && self.protocol_name == other.protocol_name
            && self.payload_protocol == other.payload_protocol
    }
}
use std::hash::{Hash, Hasher};

impl<'a> Hash for Internet<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.source_type.hash(state);
        self.destination.hash(state);
        self.destination_type.hash(state);
        self.protocol_name.hash(state);
        self.payload_protocol.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::transport::protocols::TransportProtocol;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_internet_try_from_empty_packet() {
        let packet: &[u8] = &[];
        let result = Internet::try_from(packet);

        assert!(matches!(result, Err(InternetError::EmptyPacket)));
    }

    #[test]
    fn test_internet_try_from_arp() {
        // ARP request minimal valide :
        // HTYPE=1 (Ethernet), PTYPE=0x0800 (IPv4), HLEN=6, PLEN=4, OPER=1 (request)
        // Sender MAC = 00:11:22:33:44:55
        // Sender IP  = 192.168.1.10
        // Target MAC = 00:00:00:00:00:00
        // Target IP  = 192.168.1.1
        let packet = vec![
            0x00, 0x01, // HTYPE
            0x08, 0x00, // PTYPE
            0x06, // HLEN
            0x04, // PLEN
            0x00, 0x01, // OPER
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Sender MAC
            192, 168, 1, 10, // Sender IP
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Target MAC
            192, 168, 1, 1, // Target IP
        ];

        let result = Internet::try_from(packet.as_slice()).unwrap();

        assert_eq!(
            result.source,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)))
        );
        assert_eq!(
            result.destination,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
        );
        assert_eq!(result.source_type, Some(IpType::from_ip("192.168.1.10")));
        assert_eq!(
            result.destination_type,
            Some(IpType::from_ip("192.168.1.1"))
        );
        assert_eq!(result.protocol_name, "ARP");
        assert_eq!(result.payload_protocol, None);
        assert!(result.payload.is_empty());
    }

    #[test]
    fn test_internet_try_from_ipv4_tcp() {
        // Header IPv4 minimal valide (20 octets), protocol = TCP (6)
        // Version=4, IHL=5, Total Length=20
        // Source=192.168.1.10, Destination=192.168.1.20
        let packet = vec![
            0x45, // Version + IHL
            0x00, // DSCP/ECN
            0x00, 0x14, // Total Length = 20
            0x12, 0x34, // Identification
            0x00, 0x00, // Flags + Fragment offset
            64,   // TTL
            6,    // Protocol = TCP
            0x00, 0x00, // Header checksum
            192, 168, 1, 10, // Source IP
            192, 168, 1, 20, // Destination IP
        ];

        let result = Internet::try_from(packet.as_slice()).unwrap();

        assert_eq!(
            result.source,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)))
        );
        assert_eq!(
            result.destination,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)))
        );
        assert_eq!(result.source_type, Some(IpType::from_ip("192.168.1.10")));
        assert_eq!(
            result.destination_type,
            Some(IpType::from_ip("192.168.1.20"))
        );
        assert_eq!(result.protocol_name, "IPv4");
        assert_eq!(result.payload_protocol, Some(TransportProtocol::Tcp));
        assert!(result.payload.is_empty());
    }

    #[test]
    fn test_internet_try_from_ipv6_udp() {
        // Header IPv6 minimal valide (40 octets), next_header = UDP (17), payload length = 0
        let packet = vec![
            0x60, 0x00, 0x00, 0x00, // Version, Traffic Class, Flow Label
            0x00, 0x00, // Payload Length = 0
            17,   // Next Header = UDP
            64,   // Hop Limit
            // Source IP = 2001:db8::1
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, // Destination IP = 2001:db8::2
            0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02,
        ];

        let result = Internet::try_from(packet.as_slice()).unwrap();

        assert_eq!(
            result.source,
            Some(IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1)))
        );
        assert_eq!(
            result.destination,
            Some(IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 2)))
        );
        assert_eq!(result.source_type, Some(IpType::from_ip("2001:db8::1")));
        assert_eq!(
            result.destination_type,
            Some(IpType::from_ip("2001:db8::2"))
        );
        assert_eq!(result.protocol_name, "IPv6");
        assert_eq!(result.payload_protocol, Some(TransportProtocol::Udp));
        assert!(result.payload.is_empty());
    }

    #[test]
    fn test_internet_try_from_unsupported_protocol() {
        // Données volontairement invalides pour ARP / IPv4 / IPv6 / Profinet
        let packet = vec![0xff, 0xaa, 0xbb, 0xcc, 0xdd, 0xee];

        let result = Internet::try_from(packet.as_slice());

        assert!(matches!(result, Err(InternetError::UnsupportedProtocol)));
    }

    #[test]
    fn test_internet_partial_eq_ignores_payload() {
        let a = Internet {
            source: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            source_type: Some(IpType::from_ip("192.168.1.10")),
            destination: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))),
            destination_type: Some(IpType::from_ip("192.168.1.20")),
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Tcp),
            payload: &[1, 2, 3, 4],
        };

        let b = Internet {
            source: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            source_type: Some(IpType::from_ip("192.168.1.10")),
            destination: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))),
            destination_type: Some(IpType::from_ip("192.168.1.20")),
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Tcp),
            payload: &[9, 9, 9, 9],
        };

        assert_eq!(a, b);
    }

    #[test]
    fn test_internet_hash_ignores_payload() {
        let a = Internet {
            source: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            source_type: Some(IpType::from_ip("10.0.0.1")),
            destination: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2))),
            destination_type: Some(IpType::from_ip("10.0.0.2")),
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Udp),
            payload: &[1, 2, 3],
        };

        let b = Internet {
            source: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            source_type: Some(IpType::from_ip("10.0.0.1")),
            destination: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2))),
            destination_type: Some(IpType::from_ip("10.0.0.2")),
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Udp),
            payload: &[99, 88, 77],
        };

        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();

        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);

        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_internet_partial_eq_detects_difference() {
        let a = Internet {
            source: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            source_type: Some(IpType::from_ip("192.168.1.10")),
            destination: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))),
            destination_type: Some(IpType::from_ip("192.168.1.20")),
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Tcp),
            payload: &[],
        };

        let b = Internet {
            source: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 11))),
            source_type: Some(IpType::from_ip("192.168.1.11")),
            destination: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))),
            destination_type: Some(IpType::from_ip("192.168.1.20")),
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Tcp),
            payload: &[],
        };

        assert_ne!(a, b);
    }
}
