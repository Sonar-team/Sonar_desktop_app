// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt::{self, Display, Formatter};

use crate::parse::PacketFlow;

pub(crate) mod application;
pub(crate) mod data_link;
pub(crate) mod internet;
pub(crate) mod transport;

impl Display for PacketFlow<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ParsedPacket :")?;
        writeln!(f, "  Data Link Layer: {}", self.data_link)?;

        if let Some(internet) = &self.internet {
            writeln!(f, "  Internet Layer: {internet}")?;
        }

        if let Some(trans) = &self.transport {
            writeln!(f, "  Transport Layer: {trans}")?;
        }
        if let Some(app) = &self.application {
            writeln!(f, "  Application Layer: {app}")?;
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::PacketFlow;

    #[test]
    fn test_packet_flow_display_only_data_link() {
        let payload = [0x01, 0x02, 0x03, 0x04];

        let packet = PacketFlow {
            data_link: crate::parse::data_link::DataLink {
                destination_mac: "AA:BB:CC:DD:EE:FF".to_string(),
                source_mac: "11:22:33:44:55:66".to_string(),
                ethertype: "IPv4".to_string(),
                vlan: None,
                payload: &payload,
            },
            internet: None,
            transport: None,
            application: None,
        };

        let expected = concat!(
            "ParsedPacket :\n",
            "  Data Link Layer: \n",
            "    Destination MAC: AA:BB:CC:DD:EE:FF,\n",
            "    Source MAC: 11:22:33:44:55:66,\n",
            "    Ethertype: IPv4,\n",
            "    VLAN: None,\n",
            "    Payload Length: 4\n",
            "\n"
        );

        assert_eq!(packet.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_display_omits_none_layers() {
        let payload = [0xAA];

        let packet = PacketFlow {
            data_link: crate::parse::data_link::DataLink {
                destination_mac: "FF:FF:FF:FF:FF:FF".to_string(),
                source_mac: "00:00:00:00:00:00".to_string(),
                ethertype: "ARP".to_string(),
                vlan: None,
                payload: &payload,
            },
            internet: None,
            transport: None,
            application: None,
        };

        let rendered = packet.to_string();

        assert!(rendered.contains("ParsedPacket :"));
        assert!(rendered.contains("Data Link Layer:"));
        assert!(!rendered.contains("Internet Layer:"));
        assert!(!rendered.contains("Transport Layer:"));
        assert!(!rendered.contains("Application Layer:"));
    }
}
