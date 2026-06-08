use serde::Serialize;
use std::{
    fmt::{Display, Formatter, Result},
    net::IpAddr,
};

use crate::parse::data_link::vlan_tag::VlanTag;
use crate::{IpType, PacketFlow};

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct PacketFlowOwned {
    #[serde(flatten)]
    pub data_link: DataLinkOwned,
    #[serde(flatten)]
    pub internet: Option<InternetOwned>,
    #[serde(flatten)]
    pub transport: Option<TransportOwned>,
    #[serde(flatten)]
    pub application: Option<ApplicationOwned>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct DataLinkOwned {
    pub destination_mac: String,
    /// The source MAC address as a string.
    pub source_mac: String,
    /// The Ethertype of the packet, indicating the protocol in the payload.
    pub ethertype: String,
    pub vlan: Option<VlanTag>,
}

impl Display for DataLinkOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "\n    Destination MAC: {},\n    Source MAC: {},\n    Ethertype: {},\n    VLAN: ",
            self.destination_mac, self.source_mac, self.ethertype,
        )?;

        match &self.vlan {
            Some(vlan) => write!(f, "{vlan}")?,
            None => write!(f, "None")?,
        }

        writeln!(f)
    }
}

impl Display for InternetOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Pas d’allocation: on écrit directement.
        write!(f, "\n    Source IP: ")?;
        match &self.source_ip {
            Some(ip) => write!(f, "{ip}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Destination IP: ")?;
        match &self.destination_ip {
            Some(ip) => write!(f, "{ip}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Protocol: {}\n", self.protocol)
    }
}

impl Display for TransportOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\n    Source Port: ")?;
        match self.source_port {
            Some(p) => write!(f, "{p}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Destination Port: ")?;
        match self.destination_port {
            Some(p) => write!(f, "{p}")?,
            None => write!(f, "None")?,
        }

        write!(f, ",\n    Protocol: {}\n", self.protocol)
    }
}

impl Display for ApplicationOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\n    Protocol: {}\n", self.protocol)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct InternetOwned {
    pub source_ip: Option<IpAddr>,
    pub ip_source_type: Option<IpType>,
    pub destination_ip: Option<IpAddr>,
    pub ip_destination_type: Option<IpType>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct TransportOwned {
    pub source_port: Option<u16>,
    pub destination_port: Option<u16>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Hash, Eq)]
pub struct ApplicationOwned {
    pub protocol: String,
}

impl<'a> From<PacketFlow<'a>> for PacketFlowOwned {
    fn from(flow: PacketFlow<'a>) -> Self {
        Self {
            data_link: DataLinkOwned {
                destination_mac: flow.data_link.destination_mac,
                source_mac: flow.data_link.source_mac,
                ethertype: flow.data_link.ethertype,
                vlan: flow.data_link.vlan,
            },
            internet: flow.internet.map(|internet| InternetOwned {
                source_ip: internet.source,
                ip_source_type: internet.source_type,
                destination_ip: internet.destination,
                ip_destination_type: internet.destination_type,
                protocol: internet.protocol_name,
            }),
            transport: flow.transport.map(|transport| TransportOwned {
                source_port: transport.source_port,
                destination_port: transport.destination_port,
                protocol: transport.protocol.to_string(),
            }),
            application: flow.application.map(|application| ApplicationOwned {
                protocol: application.application_protocol.to_string(),
            }),
        }
    }
}

impl Display for PacketFlowOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Packet Flow:")?;
        writeln!(f, "  Data Link: {}", self.data_link)?;

        if let Some(internet) = &self.internet {
            writeln!(f, "  Internet: {internet}")?;
        }
        if let Some(transport) = &self.transport {
            writeln!(f, "  Transport: {transport}")?;
        }
        if let Some(application) = &self.application {
            writeln!(f, "  Application: {application}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::net::{IpAddr, Ipv4Addr};

    fn sample_data_link_without_vlan() -> DataLinkOwned {
        DataLinkOwned {
            destination_mac: "AA:BB:CC:DD:EE:FF".to_string(),
            source_mac: "11:22:33:44:55:66".to_string(),
            ethertype: "IPv4".to_string(),
            vlan: None,
        }
    }

    fn sample_internet() -> InternetOwned {
        InternetOwned {
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            ip_source_type: None,
            destination_ip: Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            ip_destination_type: None,
            protocol: "TCP".to_string(),
        }
    }

    fn sample_transport() -> TransportOwned {
        TransportOwned {
            source_port: Some(12345),
            destination_port: Some(80),
            protocol: "TCP".to_string(),
        }
    }

    fn sample_application() -> ApplicationOwned {
        ApplicationOwned {
            protocol: "HTTP".to_string(),
        }
    }

    fn sample_packet_flow_owned() -> PacketFlowOwned {
        PacketFlowOwned {
            data_link: sample_data_link_without_vlan(),
            internet: Some(sample_internet()),
            transport: Some(sample_transport()),
            application: Some(sample_application()),
        }
    }

    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_data_link_owned_display_without_vlan() {
        let data_link = sample_data_link_without_vlan();

        let expected = "\n    Destination MAC: AA:BB:CC:DD:EE:FF,\n    Source MAC: 11:22:33:44:55:66,\n    Ethertype: IPv4,\n    VLAN: None\n";
        assert_eq!(data_link.to_string(), expected);
    }

    #[test]
    fn test_internet_owned_display_with_ips() {
        let internet = sample_internet();

        let expected =
            "\n    Source IP: 192.168.1.10,\n    Destination IP: 8.8.8.8,\n    Protocol: TCP\n";
        assert_eq!(internet.to_string(), expected);
    }

    #[test]
    fn test_internet_owned_display_with_none_ips() {
        let internet = InternetOwned {
            source_ip: None,
            ip_source_type: None,
            destination_ip: None,
            ip_destination_type: None,
            protocol: "UDP".to_string(),
        };

        let expected = "\n    Source IP: None,\n    Destination IP: None,\n    Protocol: UDP\n";
        assert_eq!(internet.to_string(), expected);
    }

    #[test]
    fn test_transport_owned_display_with_ports() {
        let transport = sample_transport();

        let expected = "\n    Source Port: 12345,\n    Destination Port: 80,\n    Protocol: TCP\n";
        assert_eq!(transport.to_string(), expected);
    }

    #[test]
    fn test_transport_owned_display_with_none_ports() {
        let transport = TransportOwned {
            source_port: None,
            destination_port: None,
            protocol: "ICMP".to_string(),
        };

        let expected =
            "\n    Source Port: None,\n    Destination Port: None,\n    Protocol: ICMP\n";
        assert_eq!(transport.to_string(), expected);
    }

    #[test]
    fn test_application_owned_display() {
        let application = sample_application();

        let expected = "\n    Protocol: HTTP\n";
        assert_eq!(application.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_owned_display_full() {
        let flow = sample_packet_flow_owned();

        let expected = concat!(
            "Packet Flow:\n",
            "  Data Link: \n",
            "    Destination MAC: AA:BB:CC:DD:EE:FF,\n",
            "    Source MAC: 11:22:33:44:55:66,\n",
            "    Ethertype: IPv4,\n",
            "    VLAN: None\n",
            "\n",
            "  Internet: \n",
            "    Source IP: 192.168.1.10,\n",
            "    Destination IP: 8.8.8.8,\n",
            "    Protocol: TCP\n",
            "\n",
            "  Transport: \n",
            "    Source Port: 12345,\n",
            "    Destination Port: 80,\n",
            "    Protocol: TCP\n",
            "\n",
            "  Application: \n",
            "    Protocol: HTTP\n",
            "\n"
        );

        assert_eq!(flow.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_owned_display_only_data_link() {
        let flow = PacketFlowOwned {
            data_link: sample_data_link_without_vlan(),
            internet: None,
            transport: None,
            application: None,
        };

        let expected = concat!(
            "Packet Flow:\n",
            "  Data Link: \n",
            "    Destination MAC: AA:BB:CC:DD:EE:FF,\n",
            "    Source MAC: 11:22:33:44:55:66,\n",
            "    Ethertype: IPv4,\n",
            "    VLAN: None\n",
            "\n"
        );

        assert_eq!(flow.to_string(), expected);
    }

    #[test]
    fn test_packet_flow_owned_clone_and_eq() {
        let flow = sample_packet_flow_owned();
        let cloned = flow.clone();

        assert_eq!(flow, cloned);
    }

    #[test]
    fn test_packet_flow_owned_hash_stable_for_equal_values() {
        let flow1 = sample_packet_flow_owned();
        let flow2 = sample_packet_flow_owned();

        assert_eq!(flow1, flow2);
        assert_eq!(hash_of(&flow1), hash_of(&flow2));
    }

    #[test]
    fn test_data_link_owned_hash_stable_for_equal_values() {
        let dl1 = sample_data_link_without_vlan();
        let dl2 = sample_data_link_without_vlan();

        assert_eq!(dl1, dl2);
        assert_eq!(hash_of(&dl1), hash_of(&dl2));
    }

    #[test]
    fn test_packet_flow_owned_serialize() {
        let flow = sample_packet_flow_owned();
        let json = serde_json::to_string(&flow).unwrap();

        assert!(json.contains("\"destination_mac\":\"AA:BB:CC:DD:EE:FF\""));
        assert!(json.contains("\"source_mac\":\"11:22:33:44:55:66\""));
        assert!(json.contains("\"ethertype\":\"IPv4\""));
        assert!(json.contains("\"source_ip\":\"192.168.1.10\""));
        assert!(json.contains("\"destination_ip\":\"8.8.8.8\""));
        assert!(json.contains("\"source_port\":12345"));
        assert!(json.contains("\"destination_port\":80"));
        assert!(json.contains("\"protocol\":\"HTTP\"") || json.contains("\"protocol\":\"TCP\""));
    }

    #[test]
    fn test_internet_owned_serialize_none_fields() {
        let internet = InternetOwned {
            source_ip: None,
            ip_source_type: None,
            destination_ip: None,
            ip_destination_type: None,
            protocol: "UDP".to_string(),
        };

        let json = serde_json::to_string(&internet).unwrap();

        assert!(json.contains("\"source_ip\":null"));
        assert!(json.contains("\"destination_ip\":null"));
        assert!(json.contains("\"protocol\":\"UDP\""));
    }
}
