use serde::Serialize;

use crate::Transport;

pub mod tcp;
pub mod udp;

/// Represents transport protocols AND IPv6 extension headers
#[derive(Debug, Clone, Copy, Serialize, Hash, PartialEq, Eq)]
pub enum TransportProtocol {
    // 0
    Hopopt,
    // 1
    Icmp,
    // 2
    Igmp,
    // 3
    Ggp,
    // 4
    Ipv4,
    // 5
    St,
    // 6
    Tcp,
    // 7
    Cbt,
    // 8
    Egp,
    // 9
    Igp,
    // 10
    BbnRccMon,
    // 11
    NvpIi,
    // 12
    Pup,
    // 13
    Argus,
    // 14
    Emcon,
    // 15
    Xnet,
    // 16
    Chaos,
    // 17
    Udp,
    // 18
    Mux,
    // 19
    DcnMeas,
    // 20
    Hmp,
    // 21
    Prm,
    // 22
    XnsIdp,
    // 23
    Trunk1,
    // 24
    Trunk2,
    // 25
    Leaf1,
    // 26
    Leaf2,
    // 27
    Rdp,
    // 28
    Irtp,
    // 29
    IsoTp4,
    // 30
    Netblt,
    // 31
    MfeNsp,
    // 32
    MeritInp,
    // 33
    Dccp,
    // 34
    ThreePc,
    // 35
    Idpr,
    // 36
    Xtp,
    // 37
    Ddp,
    // 38
    IdprCmtp,
    // 39
    TpPlusPlus,
    // 40
    Il,
    // 41
    Ipv6,
    // 42
    Sdrp,
    // 43
    Ipv6Route,
    // 44
    Ipv6Frag,
    // 45
    Idrp,
    // 46
    Rsvp,
    // 47
    Gre,
    // 48
    Dsr,
    // 49
    Bna,
    // 50
    Esp,
    // 51
    Ah,
    // 52
    INlsp,
    // 53
    Swipe,
    // 54
    Narp,
    // 55
    MinIpv4,
    // 56
    Tlsp,
    // 57
    Skip,
    // 58
    Ipv6Icmp,
    // 59
    Ipv6NoNxt,
    // 60
    Ipv6Opts,
    // 61
    AnyHostInternalProtocol,
    // 62
    Cftp,
    // 63
    AnyLocalNetwork,
    // 64
    SatExpak,
    // 65
    Kryptolan,
    // 66
    Rvd,
    // 67
    Ippc,
    // 68
    AnyDistributedFileSystem,
    // 69
    SatMon,
    // 70
    Visa,
    // 71
    Ipcv,
    // 72
    Cpnx,
    // 73
    Cphb,
    // 74
    Wsn,
    // 75
    Pvp,
    // 76
    BrSatMon,
    // 77
    SunNd,
    // 78
    WbMon,
    // 79
    WbExpak,
    // 80
    IsoIp,
    // 81
    Vmtp,
    // 82
    SecureVmtp,
    // 83
    Vines,
    // 84
    Iptm,
    // 85
    NsfnetIgp,
    // 86
    Dgp,
    // 87
    Tcf,
    // 88
    Eigrp,
    // 89
    Ospfigp,
    // 90
    SpriteRpc,
    // 91
    Larp,
    // 92
    Mtp,
    // 93
    Ax25,
    // 94
    Ipip,
    // 95
    Micp,
    // 96
    SccSp,
    // 97
    Etherip,
    // 98
    Encap,
    // 99
    AnyPrivateEncryptionScheme,
    // 100
    Gmtp,
    // 101
    Ifmp,
    // 102
    Pnni,
    // 103
    Pim, // PIM (la version se déduit ensuite dans le payload)
    // 104
    Aris,
    // 105
    Scps,
    // 106
    Qnx,
    // 107
    AN, // Active Networks
    // 108
    Ipcomp,
    // 109
    Snp,
    // 110
    CompaqPeer,
    // 111
    IpxInIp,
    // 112
    Vrrp,
    // 113
    Pgm,
    // 114
    Any0HopProtocol,
    // 115
    L2tp,
    // 116
    Ddx,
    // 117
    Iatp,
    // 118
    Stp,
    // 119
    Srp,
    // 120
    Uti,
    // 121
    Smp,
    // 122
    Sm,
    // 123
    Ptp,
    // 124
    IsisOverIpv4,
    // 125
    Fire,
    // 126
    Crtp,
    // 127
    Crudp,
    // 128
    Sscopmce,
    // 129
    Iplt,
    // 130
    Sps,
    // 131
    Pipe,
    // 132
    Sctp,
    // 133
    Fc,
    // 134
    RsvpE2eIgnore,
    // 135
    MobilityHeader,
    // 136
    Udplite,
    // 137
    MplsInIp,
    // 138
    Manet,
    // 139
    Hip,
    // 140
    Shim6,
    // 141
    Wesp,
    // 142
    Rohc,
    // 143
    Ethernet,
    // 144
    Aggfrag,
    // 145
    Nsh,
    // 146
    Homa,
    // 147
    BitEmu,

    // 253
    Experimentation253,
    // 254
    Experimentation254,
    // 255
    Reserved255,

    /// Anything else (including 148-252 currently unassigned)
    Unknown(u8),
}

impl TransportProtocol {
    /// Converts an IANA protocol number / IPv6 next-header number
    /// Converts an IANA protocol number / IPv6 next-header number
    pub fn from_u8(value: u8) -> Self {
        use TransportProtocol::*;
        match value {
            0 => Hopopt,
            1 => Icmp,
            2 => Igmp,
            3 => Ggp,
            4 => Ipv4,
            5 => St,
            6 => Tcp,
            7 => Cbt,
            8 => Egp,
            9 => Igp,
            10 => BbnRccMon,
            11 => NvpIi,
            12 => Pup,
            13 => Argus,
            14 => Emcon,
            15 => Xnet,
            16 => Chaos,
            17 => Udp,
            18 => Mux,
            19 => DcnMeas,
            20 => Hmp,
            21 => Prm,
            22 => XnsIdp,
            23 => Trunk1,
            24 => Trunk2,
            25 => Leaf1,
            26 => Leaf2,
            27 => Rdp,
            28 => Irtp,
            29 => IsoTp4,
            30 => Netblt,
            31 => MfeNsp,
            32 => MeritInp,
            33 => Dccp,
            34 => ThreePc,
            35 => Idpr,
            36 => Xtp,
            37 => Ddp,
            38 => IdprCmtp,
            39 => TpPlusPlus,
            40 => Il,
            41 => Ipv6,
            42 => Sdrp,
            43 => Ipv6Route,
            44 => Ipv6Frag,
            45 => Idrp,
            46 => Rsvp,
            47 => Gre,
            48 => Dsr,
            49 => Bna,
            50 => Esp,
            51 => Ah,
            52 => INlsp,
            53 => Swipe,
            54 => Narp,
            55 => MinIpv4,
            56 => Tlsp,
            57 => Skip,
            58 => Ipv6Icmp,
            59 => Ipv6NoNxt,
            60 => Ipv6Opts,
            61 => AnyHostInternalProtocol,
            62 => Cftp,
            63 => AnyLocalNetwork,
            64 => SatExpak,
            65 => Kryptolan,
            66 => Rvd,
            67 => Ippc,
            68 => AnyDistributedFileSystem,
            69 => SatMon,
            70 => Visa,
            71 => Ipcv,
            72 => Cpnx,
            73 => Cphb,
            74 => Wsn,
            75 => Pvp,
            76 => BrSatMon,
            77 => SunNd,
            78 => WbMon,
            79 => WbExpak,
            80 => IsoIp,
            81 => Vmtp,
            82 => SecureVmtp,
            83 => Vines,
            84 => Iptm,
            85 => NsfnetIgp,
            86 => Dgp,
            87 => Tcf,
            88 => Eigrp,
            89 => Ospfigp,
            90 => SpriteRpc,
            91 => Larp,
            92 => Mtp,
            93 => Ax25,
            94 => Ipip,
            95 => Micp,
            96 => SccSp,
            97 => Etherip,
            98 => Encap,
            99 => AnyPrivateEncryptionScheme,
            100 => Gmtp,
            101 => Ifmp,
            102 => Pnni,
            103 => Pim,
            104 => Aris,
            105 => Scps,
            106 => Qnx,
            107 => AN,
            108 => Ipcomp,
            109 => Snp,
            110 => CompaqPeer,
            111 => IpxInIp,
            112 => Vrrp,
            113 => Pgm,
            114 => Any0HopProtocol,
            115 => L2tp,
            116 => Ddx,
            117 => Iatp,
            118 => Stp,
            119 => Srp,
            120 => Uti,
            121 => Smp,
            122 => Sm,
            123 => Ptp,
            124 => IsisOverIpv4,
            125 => Fire,
            126 => Crtp,
            127 => Crudp,
            128 => Sscopmce,
            129 => Iplt,
            130 => Sps,
            131 => Pipe,
            132 => Sctp,
            133 => Fc,
            134 => RsvpE2eIgnore,
            135 => MobilityHeader,
            136 => Udplite,
            137 => MplsInIp,
            138 => Manet,
            139 => Hip,
            140 => Shim6,
            141 => Wesp,
            142 => Rohc,
            143 => Ethernet,
            144 => Aggfrag,
            145 => Nsh,
            146 => Homa,
            147 => BitEmu,
            253 => Experimentation253,
            254 => Experimentation254,
            255 => Reserved255,
            _ => Unknown(value),
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
            TransportProtocol::Ipv6Icmp
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
            TransportProtocol::Hopopt
        ));
        assert!(matches!(
            TransportProtocol::from_u8(43),
            TransportProtocol::Ipv6Route
        ));
        assert!(matches!(
            TransportProtocol::from_u8(44),
            TransportProtocol::Ipv6Frag
        ));
        assert!(matches!(
            TransportProtocol::from_u8(59),
            TransportProtocol::Ipv6NoNxt
        ));
        assert!(matches!(
            TransportProtocol::from_u8(60),
            TransportProtocol::Ipv6Opts
        ));
        assert!(matches!(
            TransportProtocol::from_u8(135),
            TransportProtocol::MobilityHeader
        ));
        assert!(matches!(
            TransportProtocol::from_u8(148),
            TransportProtocol::Unknown(148)
        ));
        assert!(matches!(
            TransportProtocol::from_u8(252),
            TransportProtocol::Unknown(252)
        ));
        assert!(matches!(
            TransportProtocol::from_u8(255),
            TransportProtocol::Reserved255
        ));
        assert!(matches!(
            TransportProtocol::from_u8(254),
            TransportProtocol::Experimentation254
        ));
        assert!(matches!(
            TransportProtocol::from_u8(253),
            TransportProtocol::Experimentation253
        ));
        assert!(matches!(
            TransportProtocol::from_u8(147),
            TransportProtocol::BitEmu
        ));
        assert!(matches!(
            TransportProtocol::from_u8(146),
            TransportProtocol::Homa
        ));
        assert!(matches!(
            TransportProtocol::from_u8(145),
            TransportProtocol::Nsh
        ));
        assert!(matches!(
            TransportProtocol::from_u8(144),
            TransportProtocol::Aggfrag
        ));
        assert!(matches!(
            TransportProtocol::from_u8(143),
            TransportProtocol::Ethernet
        ));
        assert!(matches!(
            TransportProtocol::from_u8(142),
            TransportProtocol::Rohc
        ));
        assert!(matches!(
            TransportProtocol::from_u8(141),
            TransportProtocol::Wesp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(140),
            TransportProtocol::Shim6
        ));
        assert!(matches!(
            TransportProtocol::from_u8(139),
            TransportProtocol::Hip
        ));
        assert!(matches!(
            TransportProtocol::from_u8(138),
            TransportProtocol::Manet
        ));
        assert!(matches!(
            TransportProtocol::from_u8(137),
            TransportProtocol::MplsInIp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(136),
            TransportProtocol::Udplite
        ));

        assert!(matches!(
            TransportProtocol::from_u8(134),
            TransportProtocol::RsvpE2eIgnore
        ));
        assert!(matches!(
            TransportProtocol::from_u8(133),
            TransportProtocol::Fc
        ));
        assert!(matches!(
            TransportProtocol::from_u8(132),
            TransportProtocol::Sctp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(131),
            TransportProtocol::Pipe
        ));
        assert!(matches!(
            TransportProtocol::from_u8(130),
            TransportProtocol::Sps
        ));
        assert!(matches!(
            TransportProtocol::from_u8(129),
            TransportProtocol::Iplt
        ));
        assert!(matches!(
            TransportProtocol::from_u8(128),
            TransportProtocol::Sscopmce
        ));
        assert!(matches!(
            TransportProtocol::from_u8(127),
            TransportProtocol::Crudp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(126),
            TransportProtocol::Crtp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(125),
            TransportProtocol::Fire
        ));
        assert!(matches!(
            TransportProtocol::from_u8(124),
            TransportProtocol::IsisOverIpv4
        ));
        assert!(matches!(
            TransportProtocol::from_u8(123),
            TransportProtocol::Ptp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(122),
            TransportProtocol::Sm
        ));
        assert!(matches!(
            TransportProtocol::from_u8(121),
            TransportProtocol::Smp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(120),
            TransportProtocol::Uti
        ));
        assert!(matches!(
            TransportProtocol::from_u8(119),
            TransportProtocol::Srp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(118),
            TransportProtocol::Stp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(117),
            TransportProtocol::Iatp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(116),
            TransportProtocol::Ddx
        ));
        assert!(matches!(
            TransportProtocol::from_u8(115),
            TransportProtocol::L2tp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(114),
            TransportProtocol::Any0HopProtocol
        ));
        assert!(matches!(
            TransportProtocol::from_u8(113),
            TransportProtocol::Pgm
        ));
        assert!(matches!(
            TransportProtocol::from_u8(112),
            TransportProtocol::Vrrp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(111),
            TransportProtocol::IpxInIp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(110),
            TransportProtocol::CompaqPeer
        ));
        assert!(matches!(
            TransportProtocol::from_u8(109),
            TransportProtocol::Snp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(108),
            TransportProtocol::Ipcomp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(107),
            TransportProtocol::AN
        ));
        assert!(matches!(
            TransportProtocol::from_u8(106),
            TransportProtocol::Qnx
        ));
        assert!(matches!(
            TransportProtocol::from_u8(105),
            TransportProtocol::Scps
        ));
        assert!(matches!(
            TransportProtocol::from_u8(104),
            TransportProtocol::Aris
        ));
        assert!(matches!(
            TransportProtocol::from_u8(2),
            TransportProtocol::Igmp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(3),
            TransportProtocol::Ggp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(4),
            TransportProtocol::Ipv4
        ));
        assert!(matches!(
            TransportProtocol::from_u8(5),
            TransportProtocol::St
        ));
        assert!(matches!(
            TransportProtocol::from_u8(7),
            TransportProtocol::Cbt
        ));
        assert!(matches!(
            TransportProtocol::from_u8(8),
            TransportProtocol::Egp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(9),
            TransportProtocol::Igp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(10),
            TransportProtocol::BbnRccMon
        ));
        assert!(matches!(
            TransportProtocol::from_u8(11),
            TransportProtocol::NvpIi
        ));
        assert!(matches!(
            TransportProtocol::from_u8(12),
            TransportProtocol::Pup
        ));
        assert!(matches!(
            TransportProtocol::from_u8(13),
            TransportProtocol::Argus
        ));
        assert!(matches!(
            TransportProtocol::from_u8(14),
            TransportProtocol::Emcon
        ));
        assert!(matches!(
            TransportProtocol::from_u8(15),
            TransportProtocol::Xnet
        ));
        assert!(matches!(
            TransportProtocol::from_u8(16),
            TransportProtocol::Chaos
        ));
        assert!(matches!(
            TransportProtocol::from_u8(18),
            TransportProtocol::Mux
        ));
        assert!(matches!(
            TransportProtocol::from_u8(19),
            TransportProtocol::DcnMeas
        ));
        assert!(matches!(
            TransportProtocol::from_u8(20),
            TransportProtocol::Hmp
        ));
        assert!(matches!(
            TransportProtocol::from_u8(21),
            TransportProtocol::Prm
        ));
    }

    // test the to_transport method
    #[test]
    fn test_to_transport() {
        let protocol = TransportProtocol::Tcp;
        let transport = protocol.to_transport();
        assert_eq!(transport.protocol, TransportProtocol::Tcp);
        assert_eq!(transport.source_port, None);
        assert_eq!(transport.destination_port, None);
        assert_eq!(transport.payload, None);
    }
}
