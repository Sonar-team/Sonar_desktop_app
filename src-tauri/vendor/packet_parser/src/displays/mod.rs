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
    use crate::parse::data_link::ethertype::Ethertype;
    use crate::parse::data_link::mac_addres::MacAddress;

    #[test]
    fn test_packet_flow_display_only_data_link() {
        let payload = [0x01, 0x02, 0x03, 0x04];

        let packet = PacketFlow {
            data_link: crate::parse::data_link::DataLink {
                destination_mac: MacAddress([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
                source_mac: MacAddress([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]),
                ethertype: Ethertype(0x0800),
                vlan: None,
                payload: &payload,
            },
            internet: None,
            transport: None,
            application: None,
            inner: None,
            corrupted: None,
        };

        let expected = concat!(
            "ParsedPacket :\n",
            "  Data Link Layer: \n",
            "    Destination MAC: aa:bb:cc:dd:ee:ff,\n",
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
                destination_mac: MacAddress([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
                source_mac: MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
                ethertype: Ethertype(0x0806),
                vlan: None,
                payload: &payload,
            },
            internet: None,
            transport: None,
            application: None,
            inner: None,
            corrupted: None,
        };

        let rendered = packet.to_string();

        assert!(rendered.contains("ParsedPacket :"));
        assert!(rendered.contains("Data Link Layer:"));
        assert!(!rendered.contains("Internet Layer:"));
        assert!(!rendered.contains("Transport Layer:"));
        assert!(!rendered.contains("Application Layer:"));
    }
}
