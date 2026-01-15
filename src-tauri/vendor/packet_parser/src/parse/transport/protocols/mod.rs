use serde::Serialize;

use crate::Transport;

pub mod tcp;
pub mod udp;

/// Represents transport protocols AND IPv6 extension headers
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
pub enum TransportProtocol {
    // IPv6 Extension Headers
    HopByHop,           // 0
    Routing,            // 43
    Fragment,           // 44
    DestinationOptions, // 60
    Mobility,           // 135
    NoNextHeader,       // 59

    // Core transport protocols
    Tcp,
    Udp,
    Icmp,
    IcmpV6,
    Igmp,
    Pim,
    PimV2,
    Vrrp,

    // Routing protocols
    Egp,
    Igrp,
    Ospf,
    Eigrp,

    // Tunneling protocols
    Gre,
    IpInIp,

    // Security protocols
    Ah,
    Esp,

    // Other protocols
    Rdp,
    Dccp,
    Rsvp,
    Sctp,

    // Fallback
    Unknown,
}

impl TransportProtocol {
    /// Converts an IANA protocol number / IPv6 next-header number
    pub fn from_u8(value: u8) -> Self {
        match value {
            // IPv6 extension headers
            0 => TransportProtocol::HopByHop,
            43 => TransportProtocol::Routing,
            44 => TransportProtocol::Fragment,
            59 => TransportProtocol::NoNextHeader,
            60 => TransportProtocol::DestinationOptions,
            135 => TransportProtocol::Mobility,

            // Core transport protocols
            1 => TransportProtocol::Icmp,
            2 => TransportProtocol::Igmp,
            6 => TransportProtocol::Tcp,
            17 => TransportProtocol::Udp,
            58 => TransportProtocol::IcmpV6,
            103 => TransportProtocol::PimV2,
            112 => TransportProtocol::Vrrp,

            // Routing protocols
            8 => TransportProtocol::Egp,
            9 => TransportProtocol::Igrp,
            89 => TransportProtocol::Ospf,
            88 => TransportProtocol::Eigrp,

            // Tunneling
            47 => TransportProtocol::Gre,
            4 => TransportProtocol::IpInIp,

            // Security
            50 => TransportProtocol::Esp,
            51 => TransportProtocol::Ah,

            // Other protocols
            27 => TransportProtocol::Rdp,
            33 => TransportProtocol::Dccp,
            46 => TransportProtocol::Rsvp,
            132 => TransportProtocol::Sctp,

            _ => TransportProtocol::Unknown,
        }
    }
    pub fn to_transport(self) -> Transport<'static> {
        // This is a placeholder - in a real implementation, this would convert
        // the protocol enum to a Transport struct with appropriate fields
        Transport {
            protocol: self,
            source_port: None,
            destination_port: None,
            payload: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_conversion() {
        assert!(matches!(
            TransportProtocol::from_u8(6),
            TransportProtocol::Tcp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(17),
            TransportProtocol::Udp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(1),
            TransportProtocol::Icmp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(58),
            TransportProtocol::IcmpV6
        ));
        assert!(matches!(
            TransportProtocol::from_u8(47),
            TransportProtocol::Gre
        ));
        assert!(matches!(
            TransportProtocol::from_u8(50),
            TransportProtocol::Esp
        ));

        // IPv6 extension tests
        assert!(matches!(
            TransportProtocol::from_u8(0),
            TransportProtocol::HopByHop
        ));
        assert!(matches!(
            TransportProtocol::from_u8(43),
            TransportProtocol::Routing
        ));
        assert!(matches!(
            TransportProtocol::from_u8(44),
            TransportProtocol::Fragment
        ));
        assert!(matches!(
            TransportProtocol::from_u8(59),
            TransportProtocol::NoNextHeader
        ));
        assert!(matches!(
            TransportProtocol::from_u8(60),
            TransportProtocol::DestinationOptions
        ));
        assert!(matches!(
            TransportProtocol::from_u8(135),
            TransportProtocol::Mobility
        ));

        assert!(matches!(
            TransportProtocol::from_u8(255),
            TransportProtocol::Unknown
        ));
    }
}
