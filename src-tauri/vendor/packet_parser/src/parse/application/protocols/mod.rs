// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use bitcoin::BitcoinPacket;
use copt::CotpHeader;
use dhcp::DhcpPacket;
use dhcpv6::Dhcpv6Packet;
use dns::DnsPacket;
use ethernet_ip::EtherNetIpPacket;
use http::HttpRequest;
use modbus_tcp::ModbusTcpPacket;
use mqtt::MqttPacket;
use ntp::NtpPacket;
use opcua::OpcuaPacket;
use postgresql::PostgreSqlPacket;
use s7comm::S7CommPacket;
use snmp::SnmpPacket;
use tls::TlsPacket;

use crate::parse::application::protocols::{
    giop::GiopPacket, quic::QuicPacket, srvloc::SrvlocPacket,
};

pub mod ams;
pub mod bitcoin;
pub mod copt;
pub mod dhcp;
pub mod dhcpv6;
pub mod dns;
pub mod ethernet_ip;
pub mod ftp;
pub mod giop;
pub mod http;
pub mod modbus_tcp;
pub mod mqtt;
pub mod nntp;
pub mod ntp;
pub mod opcua;
pub mod postgresql;
pub mod quic;
pub mod s7comm;
pub mod smtp;
pub mod snmp;
pub mod srvloc;
pub mod tls;

/// Borne une pré-allocation dimensionnée par un champ du paquet.
///
/// Les compteurs d'éléments (`ancount` DNS, `item_count` EtherNet/IP…) sont
/// sous contrôle de l'émetteur : les utiliser tels quels dans
/// `Vec::with_capacity` permet à 17 octets de payload de réserver 4 Mo. Comme
/// le profil release déclare `panic = "abort"`, un échec d'allocation tue le
/// processus au lieu de remonter une erreur.
///
/// La borne ne change pas le comportement observable : les boucles de parsing
/// échouent de toute façon sur des données tronquées. Seule la mémoire
/// réservée d'avance est plafonnée à ce que les octets restants peuvent
/// réellement contenir.
///
/// `min_item_len` doit être la taille minimale d'un élément, en octets, et
/// être non nul.
pub(crate) fn bounded_capacity(count: usize, remaining: usize, min_item_len: usize) -> usize {
    debug_assert!(min_item_len > 0, "min_item_len doit être non nul");
    count.min(remaining / min_item_len.max(1))
}

#[cfg(test)]
mod bounded_capacity_tests {
    use super::bounded_capacity;

    #[test]
    fn keeps_the_count_when_the_buffer_can_hold_it() {
        // 32 réponses DNS de 16 octets : la pré-allocation reste utile.
        assert_eq!(bounded_capacity(32, 512, 11), 32);
    }

    #[test]
    fn clamps_a_hostile_count_to_the_available_bytes() {
        // `ancount = 0xFFFF` avec 5 octets restants : 4 Mo réclamés, 0 accordé.
        assert_eq!(bounded_capacity(65_535, 5, 11), 0);
        // Le cas réel de l'audit : 17 octets de payload, en-tête consommé.
        assert_eq!(bounded_capacity(65_535, 17, 11), 1);
    }

    #[test]
    fn handles_an_exhausted_buffer() {
        assert_eq!(bounded_capacity(65_535, 0, 11), 0);
    }

    #[test]
    fn never_exceeds_the_declared_count() {
        // Un gros buffer ne doit pas gonfler la réservation au-delà du besoin.
        assert_eq!(bounded_capacity(3, 65_535, 4), 3);
    }
}

/// The `ApplicationProtocol` enum represents the possible layer 7 information that can be parsed.
#[derive(Debug)]
pub enum ApplicationProtocol<'a> {
    Ntp(NtpPacket),
    Tls(TlsPacket<'a>),
    Http(HttpRequest<'a>),
    Mqtt(MqttPacket<'a>),
    Dhcp(DhcpPacket<'a>),
    Dhcpv6(Dhcpv6Packet<'a>),
    Bitcoin(BitcoinPacket<'a>),
    Dns(DnsPacket),
    EtherNetIp(EtherNetIpPacket<'a>),
    S7Comm(S7CommPacket<'a>),
    Snmp(SnmpPacket<'a>),
    Cotp(CotpHeader<'a>),
    Quic(QuicPacket<'a>),
    Giop(GiopPacket<'a>),
    Srvloc(SrvlocPacket<'a>),
    Ams(ams::AmsPacket<'a>),
    ModbusTcp(ModbusTcpPacket<'a>),
    Opcua(OpcuaPacket<'a>),
    PostgreSql(PostgreSqlPacket<'a>),
    Raw(&'a [u8]),
    None,
}
