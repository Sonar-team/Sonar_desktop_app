use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DnsType(pub u16);

impl DnsType {
    pub fn new(value: u16) -> Self {
        Self(value)
    }
}

#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
pub mod DnsTypes {
    use super::DnsType;

    pub const A: DnsType = DnsType(1);
    pub const NS: DnsType = DnsType(2);
    pub const MD: DnsType = DnsType(3);
    pub const MF: DnsType = DnsType(4);
    pub const CNAME: DnsType = DnsType(5);
    pub const SOA: DnsType = DnsType(6);
    pub const MB: DnsType = DnsType(7);
    pub const MG: DnsType = DnsType(8);
    pub const MR: DnsType = DnsType(9);
    pub const NULL: DnsType = DnsType(10);
    pub const WKS: DnsType = DnsType(11);
    pub const PTR: DnsType = DnsType(12);
    pub const HINFO: DnsType = DnsType(13);
    pub const MINFO: DnsType = DnsType(14);
    pub const MX: DnsType = DnsType(15);
    pub const TXT: DnsType = DnsType(16);
    pub const RP: DnsType = DnsType(17);
    pub const AFSDB: DnsType = DnsType(18);
    pub const X25: DnsType = DnsType(19);
    pub const ISDN: DnsType = DnsType(20);
    pub const RT: DnsType = DnsType(21);
    pub const NSAP: DnsType = DnsType(22);
    pub const NSAP_PTR: DnsType = DnsType(23);
    pub const SIG: DnsType = DnsType(24);
    pub const KEY: DnsType = DnsType(25);
    pub const PX: DnsType = DnsType(26);
    pub const GPOS: DnsType = DnsType(27);
    pub const AAAA: DnsType = DnsType(28);
    pub const LOC: DnsType = DnsType(29);
    pub const NXT: DnsType = DnsType(30);
    pub const EID: DnsType = DnsType(31);
    pub const NIMLOC: DnsType = DnsType(32);
    pub const SRV: DnsType = DnsType(33);
    pub const ATMA: DnsType = DnsType(34);
    pub const NAPTR: DnsType = DnsType(35);
    pub const KX: DnsType = DnsType(36);
    pub const CERT: DnsType = DnsType(37);
    pub const A6: DnsType = DnsType(38);
    pub const DNAME: DnsType = DnsType(39);
    pub const SINK: DnsType = DnsType(40);
    pub const OPT: DnsType = DnsType(41);
    pub const APL: DnsType = DnsType(42);
    pub const DS: DnsType = DnsType(43);
    pub const SSHFP: DnsType = DnsType(44);
    pub const IPSECKEY: DnsType = DnsType(45);
    pub const RRSIG: DnsType = DnsType(46);
    pub const NSEC: DnsType = DnsType(47);
    pub const DNSKEY: DnsType = DnsType(48);
    pub const DHCID: DnsType = DnsType(49);
    pub const NSEC3: DnsType = DnsType(50);
    pub const NSEC3PARAM: DnsType = DnsType(51);
    pub const TLSA: DnsType = DnsType(52);
    pub const SMIMEA: DnsType = DnsType(53);
    pub const HIP: DnsType = DnsType(55);
    pub const NINFO: DnsType = DnsType(56);
    pub const RKEY: DnsType = DnsType(57);
    pub const TALINK: DnsType = DnsType(58);
    pub const CDS: DnsType = DnsType(59);
    pub const CDNSKEY: DnsType = DnsType(60);
    pub const OPENPGPKEY: DnsType = DnsType(61);
    pub const CSYNC: DnsType = DnsType(62);
    pub const ZONEMD: DnsType = DnsType(63);
    pub const SVCB: DnsType = DnsType(64);
    pub const HTTPS: DnsType = DnsType(65);
    pub const SPF: DnsType = DnsType(99);
    pub const UINFO: DnsType = DnsType(100);
    pub const UID: DnsType = DnsType(101);
    pub const GID: DnsType = DnsType(102);
    pub const UNSPEC: DnsType = DnsType(103);
    pub const NID: DnsType = DnsType(104);
    pub const L32: DnsType = DnsType(105);
    pub const L64: DnsType = DnsType(106);
    pub const LP: DnsType = DnsType(107);
    pub const EUI48: DnsType = DnsType(108);
    pub const EUI64: DnsType = DnsType(109);
    pub const TKEY: DnsType = DnsType(249);
    pub const TSIG: DnsType = DnsType(250);
    pub const IXFR: DnsType = DnsType(251);
    pub const AXFR: DnsType = DnsType(252);
    pub const MAILB: DnsType = DnsType(253);
    pub const MAILA: DnsType = DnsType(254);
    pub const ANY: DnsType = DnsType(255);
    pub const URI: DnsType = DnsType(256);
    pub const CAA: DnsType = DnsType(257);
    pub const AVC: DnsType = DnsType(258);
    pub const DOA: DnsType = DnsType(259);
    pub const AMTRELAY: DnsType = DnsType(260);
    pub const TA: DnsType = DnsType(32768);
    pub const DLV: DnsType = DnsType(32769);
}

impl fmt::Display for DnsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                DnsTypes::A => "A",                   // 1
                DnsTypes::NS => "NS",                 // 2
                DnsTypes::MD => "MD",                 // 3
                DnsTypes::MF => "MF",                 // 4
                DnsTypes::CNAME => "CNAME",           // 5
                DnsTypes::SOA => "SOA",               // 6
                DnsTypes::MB => "MB",                 // 7
                DnsTypes::MG => "MG",                 // 8
                DnsTypes::MR => "MR",                 // 9
                DnsTypes::NULL => "NULL",             // 10
                DnsTypes::WKS => "WKS",               // 11
                DnsTypes::PTR => "PTR",               // 12
                DnsTypes::HINFO => "HINFO",           // 13
                DnsTypes::MINFO => "MINFO",           // 14
                DnsTypes::MX => "MX",                 // 15
                DnsTypes::TXT => "TXT",               // 16
                DnsTypes::RP => "RP",                 // 17
                DnsTypes::AFSDB => "AFSDB",           // 18
                DnsTypes::X25 => "X25",               // 19
                DnsTypes::ISDN => "ISDN",             // 20
                DnsTypes::RT => "RT",                 // 21
                DnsTypes::NSAP => "NSAP",             // 22
                DnsTypes::NSAP_PTR => "NSAP_PTR",     // 23
                DnsTypes::SIG => "SIG",               // 24
                DnsTypes::KEY => "KEY",               // 25
                DnsTypes::PX => "PX",                 // 26
                DnsTypes::GPOS => "GPOS",             // 27
                DnsTypes::AAAA => "AAAA",             // 28
                DnsTypes::LOC => "LOC",               // 29
                DnsTypes::NXT => "NXT",               // 30
                DnsTypes::EID => "EID",               // 31
                DnsTypes::NIMLOC => "NIMLOC",         // 32
                DnsTypes::SRV => "SRV",               // 33
                DnsTypes::ATMA => "ATMA",             // 34
                DnsTypes::NAPTR => "NAPTR",           // 35
                DnsTypes::KX => "KX",                 // 36
                DnsTypes::CERT => "CERT",             // 37
                DnsTypes::A6 => "A6",                 // 38
                DnsTypes::DNAME => "DNAME",           // 39
                DnsTypes::SINK => "SINK",             // 40
                DnsTypes::OPT => "OPT",               // 41
                DnsTypes::APL => "APL",               // 42
                DnsTypes::DS => "DS",                 // 43
                DnsTypes::SSHFP => "SSHFP",           // 44
                DnsTypes::IPSECKEY => "IPSECKEY",     // 45
                DnsTypes::RRSIG => "RRSIG",           // 46
                DnsTypes::NSEC => "NSEC",             // 47
                DnsTypes::DNSKEY => "DNSKEY",         // 48
                DnsTypes::DHCID => "DHCID",           // 49
                DnsTypes::NSEC3 => "NSEC3",           // 50
                DnsTypes::NSEC3PARAM => "NSEC3PARAM", // 51
                DnsTypes::TLSA => "TLSA",             // 52
                DnsTypes::SMIMEA => "SMIMEA",         // 53
                DnsTypes::HIP => "HIP",               // 55
                DnsTypes::NINFO => "NINFO",           // 56
                DnsTypes::RKEY => "RKEY",             // 57
                DnsTypes::TALINK => "TALINK",         // 58
                DnsTypes::CDS => "CDS",               // 59
                DnsTypes::CDNSKEY => "CDNSKEY",       // 60
                DnsTypes::OPENPGPKEY => "OPENPGPKEY", // 61
                DnsTypes::CSYNC => "CSYNC",           // 62
                DnsTypes::ZONEMD => "ZONEMD",         // 63
                DnsTypes::SVCB => "SVCB",             // 64
                DnsTypes::HTTPS => "HTTPS",           // 65
                DnsTypes::SPF => "SPF",               // 99
                DnsTypes::UINFO => "UINFO",           // 100
                DnsTypes::UID => "UID",               // 101
                DnsTypes::GID => "GID",               // 102
                DnsTypes::UNSPEC => "UNSPEC",         // 103
                DnsTypes::NID => "NID",               // 104
                DnsTypes::L32 => "L32",               // 105
                DnsTypes::L64 => "L64",               // 106
                DnsTypes::LP => "LP",                 // 107
                DnsTypes::EUI48 => "EUI48",           // 108
                DnsTypes::EUI64 => "EUI64",           // 109
                DnsTypes::TKEY => "TKEY",             // 249
                DnsTypes::TSIG => "TSIG",             // 250
                DnsTypes::IXFR => "IXFR",             // 251
                DnsTypes::AXFR => "AXFR",             // 252
                DnsTypes::MAILB => "MAILB",           // 253
                DnsTypes::MAILA => "MAILA",           // 254
                DnsTypes::ANY => "ANY",               // 255
                DnsTypes::URI => "URI",               // 256
                DnsTypes::CAA => "CAA",               // 257
                DnsTypes::AVC => "AVC",               // 258
                DnsTypes::DOA => "DOA",               // 259
                DnsTypes::AMTRELAY => "AMTRELAY",     // 260
                DnsTypes::TA => "TA",                 // 32768
                DnsTypes::DLV => "DLV",               // 32769
                _ => "unknown",
            }
        )
    }
}
