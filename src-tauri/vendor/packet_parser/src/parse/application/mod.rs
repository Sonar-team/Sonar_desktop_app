// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

pub mod protocols;
use protocols::{bitcoin::BitcoinPacket, dns::DnsPacket, s7comm::S7CommPacket, tls::TlsPacket};
use serde::Serialize;

use crate::{
    errors::application::ApplicationError,
    parse::application::protocols::{
        giop::GiopPacket, modbus_tcp::ModbusTcpPacket, ntp::NtpPacket, srvloc::SrvlocPacket,
    },
};

/// The `Application` struct contains information about the layer 7 protocol and its parsed data.
#[derive(Debug, Clone, Serialize, Eq)]
pub struct Application {
    pub application_protocol: String,
}

impl TryFrom<&[u8]> for Application {
    type Error = ApplicationError;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
        if packet.is_empty() {
            return Err(ApplicationError::EmptyPacket);
        }

        if NtpPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "NTP".to_string(),
            });
        }

        if BitcoinPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "Bitcoin".to_string(),
            });
        }
        if DnsPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "DNS".to_string(),
            });
        }
        if TlsPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "TLS".to_string(),
            });
        }
        if S7CommPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "S7Comm".to_string(),
            });
        }
        if GiopPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "GIOP".to_string(),
            });
        }
        if SrvlocPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "SRVLOCK".to_string(),
            });
        }
        if ModbusTcpPacket::try_from(packet).is_ok() {
            return Ok(Application {
                application_protocol: "ModbusTCP".to_string(),
            });
        }
        // if AmsPacket::try_from(packet).is_ok() {
        //     return Ok(Application {
        //         application_protocol: "AMS".to_string(),
        //     });
        // }

        // if CotpHeader::from_bytes(packet).is_ok() {
        //     return Ok(Application {
        //         application_protocol: "COTP".to_string(),
        //     });
        // }
        // if QuicPacket::try_from(packet).is_ok() {
        //     return Ok(Application {
        //         application_protocol: "QUIQ".to_string(),
        //     });
        // }
        // If no parser matches, return a "None" protocol
        Ok(Application {
            application_protocol: "Unknown".to_string(),
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
}
