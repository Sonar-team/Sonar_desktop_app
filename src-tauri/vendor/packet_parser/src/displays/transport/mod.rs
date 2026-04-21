use crate::parse::transport::{Transport, protocols::TransportProtocol};
use std::fmt;

impl<'a> fmt::Display for Transport<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source_port = self
            .source_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let dest_port = self
            .destination_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let payload_len = self.payload.map(|p| p.len()).unwrap_or(0);

        write!(
            f,
            r#"
    protocol: {},
    source_port: {},
    destination_port: {},
    payload_length: {},
    "#,
            self.protocol, source_port, dest_port, payload_len,
        )
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol_str = match self {
            TransportProtocol::Hopopt => "IPv6 Hop-by-Hop",
            TransportProtocol::Ipv6Route => "IPv6 Routing Header",
            TransportProtocol::Ipv6Frag => "IPv6 Fragment Header",
            TransportProtocol::Ipv6Opts => "IPv6 Destination Options",
            TransportProtocol::MobilityHeader => "IPv6 Mobility Header",
            TransportProtocol::Ipv6NoNxt => "No Next Header",
            TransportProtocol::Tcp => "Tcp",
            TransportProtocol::Udp => "UDP",
            TransportProtocol::Icmp => "ICMP",
            TransportProtocol::Ipv6Icmp => "ICMPv6",
            TransportProtocol::Igmp => "IGMP",
            TransportProtocol::Pim => "PIM",
            TransportProtocol::Vrrp => "VRRP",
            TransportProtocol::Egp => "EGP",
            TransportProtocol::Ospfigp => "OSPFIGP",
            TransportProtocol::Eigrp => "EIGRP",
            TransportProtocol::Gre => "GRE",
            TransportProtocol::Ipip => "IP-in-IP",
            TransportProtocol::Ah => "AH",
            TransportProtocol::Esp => "ESP",
            TransportProtocol::Rdp => "RDP",
            TransportProtocol::Dccp => "DCCP",
            TransportProtocol::Rsvp => "RSVP",
            TransportProtocol::Sctp => "SCTP",
            TransportProtocol::Ipv4 => "IPv4",
            TransportProtocol::St => "ST",
            TransportProtocol::Cbt => "CBT",
            TransportProtocol::Igp => "IGP",
            TransportProtocol::BbnRccMon => "BBN-RCC-MON",
            TransportProtocol::NvpIi => "NVP-II",
            TransportProtocol::Pup => "PUP",
            TransportProtocol::Argus => "ARGUS",
            TransportProtocol::Emcon => "EMCON",
            TransportProtocol::Xnet => "XNET",
            TransportProtocol::Chaos => "CHAOS",
            TransportProtocol::Mux => "MUX",
            TransportProtocol::DcnMeas => "DCN-MEAS",
            TransportProtocol::Hmp => "HMP",
            TransportProtocol::Prm => "PRM",
            TransportProtocol::XnsIdp => "XNS-IDP",
            TransportProtocol::Trunk1 => "TRUNK-1",
            TransportProtocol::Trunk2 => "TRUNK-2",
            TransportProtocol::Leaf1 => "LEAF-1",
            TransportProtocol::Leaf2 => "LEAF-2",
            TransportProtocol::Irtp => "IRTP",
            TransportProtocol::IsoTp4 => "ISO-TP4",
            TransportProtocol::Netblt => "NETBLT",
            TransportProtocol::MfeNsp => "MFE-NSP",
            TransportProtocol::MeritInp => "MERIT-INP",
            TransportProtocol::ThreePc => "3PC",
            TransportProtocol::Idpr => "IDPR",
            TransportProtocol::Xtp => "XTP",
            TransportProtocol::Ddp => "DDP",
            TransportProtocol::IdprCmtp => "IDPR-CMTP",
            TransportProtocol::TpPlusPlus => "TP++",
            TransportProtocol::Il => "IL",
            TransportProtocol::Ipv6 => "IPv6",
            TransportProtocol::Sdrp => "SDRP",
            TransportProtocol::Idrp => "IDRP",
            TransportProtocol::Dsr => "DSR",
            TransportProtocol::Bna => "BNA",
            TransportProtocol::INlsp => "INLSP",
            TransportProtocol::Swipe => "SWIPE",
            TransportProtocol::Narp => "NARP",
            TransportProtocol::MinIpv4 => "MIN-IPv4",
            TransportProtocol::Tlsp => "TLSP",
            TransportProtocol::Skip => "SKIP",
            TransportProtocol::AnyHostInternalProtocol => "ANY-HOST-INTERNAL-PROTOCOL",
            TransportProtocol::Cftp => "CFTP",
            TransportProtocol::AnyLocalNetwork => "ANY-LOCAL-NETWORK",
            TransportProtocol::SatExpak => "SAT-EXPAK",
            TransportProtocol::Kryptolan => "KRYPTOLAN",
            TransportProtocol::Rvd => "RVD",
            TransportProtocol::Ippc => "IPPC",
            TransportProtocol::AnyDistributedFileSystem => "ANY-DISTRIBUTED-FILE-SYSTEM",
            TransportProtocol::SatMon => "SAT-MON",
            TransportProtocol::Visa => "VISA",
            TransportProtocol::Ipcv => "IPCV",
            TransportProtocol::Cpnx => "CPNX",
            TransportProtocol::Cphb => "CPHB",
            TransportProtocol::Wsn => "WSN",
            TransportProtocol::Pvp => "PVP",
            TransportProtocol::BrSatMon => "BR-SAT-MON",
            TransportProtocol::SunNd => "SUN-ND",
            TransportProtocol::WbMon => "WB-MON",
            TransportProtocol::WbExpak => "WB-EXPAK",
            TransportProtocol::IsoIp => "ISO-IP",
            TransportProtocol::Vmtp => "VMTP",
            TransportProtocol::SecureVmtp => "SECURE-VMTP",
            TransportProtocol::Vines => "VINES",
            TransportProtocol::Iptm => "IPTM",
            TransportProtocol::NsfnetIgp => "NSFNET-IGP",
            TransportProtocol::Dgp => "DGP",
            TransportProtocol::Tcf => "TCF",
            TransportProtocol::SpriteRpc => "SPRITE-RPC",
            TransportProtocol::Larp => "LARP",
            TransportProtocol::Mtp => "MTP",
            TransportProtocol::Ax25 => "AX.25",
            TransportProtocol::Micp => "MICP",
            TransportProtocol::SccSp => "SCC-SP",
            TransportProtocol::Etherip => "ETHERIP",
            TransportProtocol::Encap => "ENCAP",
            TransportProtocol::AnyPrivateEncryptionScheme => "ANY-PRIVATE-ENCRYPTION-Scheme",
            TransportProtocol::Gmtp => "GMTP",
            TransportProtocol::Ifmp => "IFMP",
            TransportProtocol::Pnni => "PNNI",
            TransportProtocol::Aris => "ARIS",
            TransportProtocol::Scps => "SCPS",
            TransportProtocol::Qnx => "QNX",
            TransportProtocol::AN => "AN",
            TransportProtocol::Ipcomp => "IPCOMP",
            TransportProtocol::Snp => "SNP",
            TransportProtocol::CompaqPeer => "COMPAQ-PEER",
            TransportProtocol::IpxInIp => "IPX-IN-IP",
            TransportProtocol::Pgm => "PGM",
            TransportProtocol::Any0HopProtocol => "ANY-0-HOP-PROTOCOL",
            TransportProtocol::L2tp => "L2TP",
            TransportProtocol::Ddx => "DDX",
            TransportProtocol::Iatp => "IATP",
            TransportProtocol::Stp => "STP",
            TransportProtocol::Srp => "SRP",
            TransportProtocol::Uti => "UTI",
            TransportProtocol::Smp => "SMP",
            TransportProtocol::Sm => "SM",
            TransportProtocol::Ptp => "PTP",
            TransportProtocol::IsisOverIpv4 => "ISIS-OVER-IPV4",
            TransportProtocol::Fire => "FIRE",
            TransportProtocol::Crtp => "CRTP",
            TransportProtocol::Crudp => "CRUDP",
            TransportProtocol::Sscopmce => "SSCOPMCE",
            TransportProtocol::Iplt => "IPLT",
            TransportProtocol::Sps => "SPS",
            TransportProtocol::Pipe => "PIPE",
            TransportProtocol::Fc => "FC",
            TransportProtocol::RsvpE2eIgnore => "RSVP-E2E-IGNORE",
            TransportProtocol::Udplite => "UDPLITE",
            TransportProtocol::MplsInIp => "MPLS-IN-IP",
            TransportProtocol::Manet => "MANET",
            TransportProtocol::Hip => "HIP",
            TransportProtocol::Shim6 => "SHIM6",
            TransportProtocol::Wesp => "WESP",
            TransportProtocol::Rohc => "ROHC",
            TransportProtocol::Ethernet => "ETHERNET",
            TransportProtocol::Aggfrag => "AGGFRAG",
            TransportProtocol::Nsh => "NSH",
            TransportProtocol::Homa => "HOMA",
            TransportProtocol::BitEmu => "BIT-EMU",
            TransportProtocol::Experimentation253 => "EXPERIMENTATION-253",
            TransportProtocol::Experimentation254 => "EXPERIMENTATION-254",
            TransportProtocol::Reserved255 => "RESERVED-255",
            TransportProtocol::Ggp => "GGP",
            TransportProtocol::Unknown(value) => {
                write!(f, "Unknown ({})", value)?;
                return Ok(());
            }
        };
        write!(f, "{protocol_str}")
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::transport::{Transport, protocols::TransportProtocol};

    #[test]
    fn test_transport_display_with_all_fields() {
        let payload = [0xde, 0xad, 0xbe, 0xef];
        let transport = Transport {
            protocol: TransportProtocol::Tcp,
            source_port: Some(12345),
            destination_port: Some(80),
            payload: Some(&payload),
        };

        let displayed = format!("{transport}");

        assert!(displayed.contains("protocol: Tcp"));
        assert!(displayed.contains("source_port: 12345"));
        assert!(displayed.contains("destination_port: 80"));
        assert!(displayed.contains("payload_length: 4"));
    }

    #[test]
    fn test_transport_display_with_missing_ports_and_payload() {
        let transport = Transport {
            protocol: TransportProtocol::Udp,
            source_port: None,
            destination_port: None,
            payload: None,
        };

        let displayed = format!("{transport}");

        assert!(displayed.contains("protocol: UDP"));
        assert!(displayed.contains("source_port: N/A"));
        assert!(displayed.contains("destination_port: N/A"));
        assert!(displayed.contains("payload_length: 0"));
    }

    #[test]
    fn test_transport_display_with_only_source_port() {
        let payload = [0x01, 0x02];
        let transport = Transport {
            protocol: TransportProtocol::Icmp,
            source_port: Some(42),
            destination_port: None,
            payload: Some(&payload),
        };

        let displayed = format!("{transport}");

        assert!(displayed.contains("protocol: ICMP"));
        assert!(displayed.contains("source_port: 42"));
        assert!(displayed.contains("destination_port: N/A"));
        assert!(displayed.contains("payload_length: 2"));
    }

    #[test]
    fn test_transport_display_with_only_destination_port() {
        let payload = [0x01];
        let transport = Transport {
            protocol: TransportProtocol::Icmp,
            source_port: None,
            destination_port: Some(443),
            payload: Some(&payload),
        };

        let displayed = format!("{transport}");

        assert!(displayed.contains("protocol: ICMP"));
        assert!(displayed.contains("source_port: N/A"));
        assert!(displayed.contains("destination_port: 443"));
        assert!(displayed.contains("payload_length: 1"));
    }

    #[test]
    fn test_transport_protocol_display_tcp() {
        assert_eq!(TransportProtocol::Tcp.to_string(), "Tcp");
    }

    #[test]
    fn test_transport_protocol_display_udp() {
        assert_eq!(TransportProtocol::Udp.to_string(), "UDP");
    }

    #[test]
    fn test_transport_protocol_display_icmp() {
        assert_eq!(TransportProtocol::Icmp.to_string(), "ICMP");
    }

    #[test]
    fn test_transport_protocol_display_icmpv6() {
        assert_eq!(TransportProtocol::Ipv6Icmp.to_string(), "ICMPv6");
    }

    #[test]
    fn test_transport_protocol_display_ipv6_hop_by_hop() {
        assert_eq!(TransportProtocol::Hopopt.to_string(), "IPv6 Hop-by-Hop");
    }

    #[test]
    fn test_transport_protocol_display_unknown() {
        assert_eq!(TransportProtocol::Unknown(143).to_string(), "Unknown (143)");
    }

    #[test]
    fn test_transport_protocol_display_reserved255() {
        assert_eq!(TransportProtocol::Reserved255.to_string(), "RESERVED-255");
    }

    #[test]
    fn test_transport_protocol_display_ethernet() {
        assert_eq!(TransportProtocol::Ethernet.to_string(), "ETHERNET");
    }
}
