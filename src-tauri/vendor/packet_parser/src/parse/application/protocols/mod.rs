// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use bitcoin::BitcoinPacket;
use copt::CotpHeader;
use dhcp::DhcpPacket;
use dns::DnsPacket;
use http::HttpRequest;
use modbus_tcp::ModbusTcpPacket;
use mqtt::MqttPacket;
use ntp::NtpPacket;
use s7comm::S7CommPacket;
use tls::TlsPacket;

use crate::parse::application::protocols::{
    giop::GiopPacket, quic::QuicPacket, srvloc::SrvlocPacket,
};

pub mod ams;
pub mod bitcoin;
pub mod copt;
pub mod dhcp;
pub mod dns;
pub mod giop;
pub mod http;
pub mod modbus_tcp;
pub mod mqtt;
pub mod ntp;
pub mod quic;
pub mod s7comm;
pub mod srvloc;
pub mod tls;

/// The `ApplicationProtocol` enum represents the possible layer 7 information that can be parsed.
#[derive(Debug)]
pub enum ApplicationProtocol<'a> {
    Ntp(NtpPacket),
    Tls(TlsPacket<'a>),
    Http(HttpRequest),
    Mqtt(MqttPacket),
    Dhcp(DhcpPacket),
    Bitcoin(BitcoinPacket),
    Dns(DnsPacket),
    S7Comm(S7CommPacket<'a>),
    Cotp(CotpHeader),
    Quic(QuicPacket),
    Giop(GiopPacket),
    Srvloc(SrvlocPacket),
    Ams(ams::AmsPacket<'a>),
    ModbusTcp(ModbusTcpPacket<'a>),
    Raw(&'a [u8]),
    None,
}
