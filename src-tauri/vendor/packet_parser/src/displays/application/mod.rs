// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::parse::application::{Application, protocols::ApplicationProtocol};
use std::fmt;
pub mod bitcoin;
pub mod dhcp;
pub mod http;
impl fmt::Display for Application {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.application_protocol)
    }
}

impl<'a> fmt::Display for ApplicationProtocol<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationProtocol::ModbusTcp(_) => write!(f, "ModbusTCP"),
            ApplicationProtocol::Ntp(_) => write!(f, "NTP"),
            ApplicationProtocol::Tls(_) => write!(f, "TLS"),
            ApplicationProtocol::Http(_) => write!(f, "HTTP"),
            ApplicationProtocol::Mqtt(_) => write!(f, "MQTT"),
            ApplicationProtocol::Dhcp(_) => write!(f, "DHCP"),
            ApplicationProtocol::Dhcpv6(_) => write!(f, "DHCPv6"),
            ApplicationProtocol::Bitcoin(_) => write!(f, "Bitcoin"),
            ApplicationProtocol::Dns(_) => write!(f, "DNS"),
            ApplicationProtocol::EtherNetIp(_) => write!(f, "EtherNet/IP"),
            ApplicationProtocol::S7Comm(_) => write!(f, "S7Comm"),
            ApplicationProtocol::Snmp(_) => write!(f, "SNMP"),
            ApplicationProtocol::Cotp(_) => write!(f, "COTP"),
            ApplicationProtocol::Quic(_) => write!(f, "QUIC"),
            ApplicationProtocol::Giop(_) => write!(f, "GIOP"),
            ApplicationProtocol::Srvloc(_) => write!(f, "SRVLOC"),
            ApplicationProtocol::Ams(_) => write!(f, "AMS"),
            ApplicationProtocol::Opcua(_) => write!(f, "OPC UA"),
            ApplicationProtocol::PostgreSql(_) => write!(f, "PostgreSQL"),
            ApplicationProtocol::Raw(data) => {
                let preview_len = 16.min(data.len());
                let hex_preview: String = data[..preview_len]
                    .iter()
                    .map(|&b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");

                if data.len() > preview_len {
                    write!(f, "Raw [{} bytes]: {}...", data.len(), hex_preview)
                } else {
                    write!(f, "Raw [{} bytes]: {}", data.len(), hex_preview)
                }
            }
            ApplicationProtocol::None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_display() {
        let app = Application {
            application_protocol: "NTP",
        };
        assert_eq!(app.to_string(), "NTP ");
    }

    #[test]
    fn test_application_protocol_display_simple_variants() {
        use crate::parse::application::protocols::{
            ams::AmsPacket, mqtt::MqttPacket, ntp::NtpPacket,
        };

        // NTP : fixture minimale valide (48 octets)
        let ntp_bytes: &[u8] = &[
            0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let ntp = NtpPacket::try_from(ntp_bytes).unwrap();
        assert_eq!(ApplicationProtocol::Ntp(ntp).to_string(), "NTP");

        // MQTT : PINGREQ
        let mqtt = MqttPacket::try_from(&[0xC0u8, 0x00][..]).unwrap();
        assert_eq!(ApplicationProtocol::Mqtt(mqtt).to_string(), "MQTT");

        // AMS : header valide sans data
        let mut ams_bytes = vec![0u8; 32];
        ams_bytes[16] = 0x01; // cmd_id = 1
        let ams = AmsPacket::try_from(ams_bytes.as_slice()).unwrap();
        assert_eq!(ApplicationProtocol::Ams(ams).to_string(), "AMS");

        assert_eq!(ApplicationProtocol::None.to_string(), "None");
    }

    #[test]
    fn test_application_protocol_display_raw_short() {
        let data = [0xDE, 0xAD];
        let rendered = ApplicationProtocol::Raw(&data).to_string();
        assert_eq!(rendered, "Raw [2 bytes]: DE AD");
    }

    #[test]
    fn test_application_protocol_display_raw_truncated() {
        let data = [0xAA; 20];
        let rendered = ApplicationProtocol::Raw(&data).to_string();
        assert!(rendered.starts_with("Raw [20 bytes]:"));
        assert!(rendered.ends_with("..."));
    }
}
