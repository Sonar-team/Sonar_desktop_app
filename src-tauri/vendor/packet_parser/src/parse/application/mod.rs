// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

pub mod protocols;
use protocols::{
    bitcoin::BitcoinPacket, dns::DnsPacket, ethernet_ip::EtherNetIpPacket, s7comm::S7CommPacket,
    snmp::SnmpPacket, tls::TlsPacket,
};
use serde::Serialize;

use crate::{
    errors::application::ApplicationError,
    parse::application::protocols::{
        giop::GiopPacket, modbus_tcp::ModbusTcpPacket, ntp::NtpPacket, opcua::OpcuaPacket,
        postgresql::is_likely_postgresql_payload, quic::QuicPacket, srvloc::SrvlocPacket,
    },
};

/// The `Application` struct contains information about the layer 7 protocol and its parsed data.
#[derive(Debug, Clone, Serialize, Eq)]
pub struct Application {
    pub application_protocol: &'static str,
}

impl TryFrom<&[u8]> for Application {
    type Error = ApplicationError;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
        if packet.is_empty() {
            return Err(ApplicationError::EmptyPacket);
        }

        if NtpPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "NTP",
            });
        }

        if BitcoinPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "Bitcoin",
            });
        }
        if OpcuaPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "OPC UA",
            });
        }
        if EtherNetIpPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "EtherNet/IP",
            });
        }
        if is_likely_postgresql_payload(packet) {
            return Ok(Application {
                application_protocol: "PostgreSQL",
            });
        }
        if DnsPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "DNS",
            });
        }
        if SnmpPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "SNMP",
            });
        }
        if TlsPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "TLS",
            });
        }
        if S7CommPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "S7Comm",
            });
        }
        if GiopPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "GIOP",
            });
        }
        if SrvlocPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "SRVLOCK",
            });
        }
        if ModbusTcpPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "ModbusTCP",
            });
        }
        // if Dhcpv6Packet::try_from(packet).is_ok() {
        //     return Ok(Application {
        //         application_protocol: "DHCPv6",
        //     });
        // }
        // if AmsPacket::try_from(packet).is_ok() {
        //     return Ok(Application {
        //         application_protocol: "AMS",
        //     });
        // }

        // if CotpHeader::from_bytes(packet).is_ok() {
        //     return Ok(Application {
        //         application_protocol: "COTP",
        //     });
        // }
        if QuicPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "QUIQ",
            });
        }
        // If no parser matches, return a "None" protocol
        Ok(Application {
            application_protocol: "Unknown",
        })
    }
}

impl PartialEq for Application {
    fn eq(&self, other: &Self) -> bool {
        self.application_protocol == other.application_protocol
    }
}

use std::hash::{Hash, Hasher};

impl Hash for Application {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.application_protocol.hash(state);
    }
}

#[cfg(test)]
mod tests {

    use crate::parse::application::Application;
    use std::convert::TryFrom;

    #[test]
    fn test_ntp_packet_parsing() {
        let ntp_payload = hex::decode("d9000afa000000000001029000000000000000000000000000000000000000000000000000000000c50204ecec42ee92").expect("Invalid hex string");

        match Application::try_from(ntp_payload.as_slice()) {
            Ok(parsed) => {
                println!("Parsed application protocol: {:?}", parsed);
                assert_eq!(parsed.application_protocol, "NTP");
            }
            Err(e) => {
                panic!("Failed to parse DNS packet: {:?}", e);
            }
        }
    }

    #[test]
    fn test_postgresql_packet_detection_from_payload() {
        let query = b"select 1\0";
        let mut payload = Vec::new();
        payload.push(b'Q');
        payload.extend_from_slice(&(4 + query.len() as u32).to_be_bytes());
        payload.extend_from_slice(query);

        let parsed = Application::try_from(payload.as_slice()).unwrap();

        assert_eq!(parsed.application_protocol, "PostgreSQL");
    }

    #[test]
    fn test_postgresql_weak_shape_is_not_detected_from_payload() {
        let payload = [b'S', 0x00, 0x00, 0x00, 0x04];

        let parsed = Application::try_from(payload.as_slice()).unwrap();

        assert_ne!(parsed.application_protocol, "PostgreSQL");
    }

    #[test]
    fn test_detects_giop() {
        let mut packet = b"GIOP".to_vec();
        packet.extend_from_slice(&[1, 2, 0, 1]); // version 1.2, big-endian, Reply
        packet.extend_from_slice(&0u32.to_be_bytes()); // body vide

        let parsed = Application::try_from(packet.as_slice()).unwrap();
        assert_eq!(parsed.application_protocol, "GIOP");
    }

    #[test]
    fn test_detects_srvloc() {
        // SLP v2 header minimal, lang tag "en"
        let mut packet = vec![2u8, 8];
        packet.extend_from_slice(&[0, 0, 16]); // packet length u24
        packet.extend_from_slice(&[0x20, 0x00]); // flags
        packet.extend_from_slice(&[0, 0, 0]); // next ext offset
        packet.extend_from_slice(&[0x42, 0x42]); // xid
        packet.extend_from_slice(&[0, 2]); // lang tag len
        packet.extend_from_slice(b"en");

        let parsed = Application::try_from(packet.as_slice()).unwrap();
        assert_eq!(parsed.application_protocol, "SRVLOCK");
    }

    #[test]
    fn test_detects_quic() {
        let mut packet = vec![0xC0u8]; // Initial, PN len 1
        packet.extend_from_slice(&1u32.to_be_bytes());
        packet.push(0); // dcid len
        packet.push(0); // scid len
        packet.push(0); // token len
        packet.push(1); // length = PN seul
        packet.push(0); // PN

        let parsed = Application::try_from(packet.as_slice()).unwrap();
        assert_eq!(parsed.application_protocol, "QUIQ");
    }

    #[test]
    fn test_detects_snmp() {
        // SNMPv2c GetRequest minimal : seq(version, community, pdu)
        let packet: &[u8] = &[
            0x30, 0x18, // SEQUENCE
            0x02, 0x01, 0x01, // version 2c
            0x04, 0x06, b'p', b'u', b'b', b'l', b'i', b'c', // community
            0xA0, 0x0B, // GetRequest
            0x02, 0x01, 0x01, // request id
            0x02, 0x01, 0x00, // error status
            0x02, 0x01, 0x00, // error index
            0x30, 0x00, // varbind list vide
        ];

        let parsed = Application::try_from(packet).unwrap();
        assert_eq!(parsed.application_protocol, "SNMP");
    }

    #[test]
    fn test_detects_postgresql() {
        let mut packet = vec![b'Q'];
        let query = b"select 1\0";
        packet.extend_from_slice(&((4 + query.len()) as u32).to_be_bytes());
        packet.extend_from_slice(query);

        let parsed = Application::try_from(packet.as_slice()).unwrap();
        assert_eq!(parsed.application_protocol, "PostgreSQL");
    }

    #[test]
    fn test_unknown_payload_falls_through() {
        let packet = [0xFFu8, 0xFE, 0xFD, 0xFC, 0xFB];
        let parsed = Application::try_from(&packet[..]).unwrap();
        assert_eq!(parsed.application_protocol, "Unknown");
    }

    #[test]
    fn test_empty_packet_is_an_error() {
        assert!(matches!(
            Application::try_from(&[][..]),
            Err(crate::errors::application::ApplicationError::EmptyPacket)
        ));
    }
}
