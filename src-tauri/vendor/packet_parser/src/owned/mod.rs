use std::{hash::Hasher, net::IpAddr};

use serde::Serialize;
use std::fmt::Display;

use crate::parse::data_link::vlan_tag::VlanTag;
use crate::{Application, IpType, PacketFlow};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PacketFlowOwned {
    #[serde(flatten)]
    pub data_link: DataLinkOwned,
    #[serde(flatten)]
    pub internet: Option<InternetOwned>,
    #[serde(flatten)]
    pub transport: Option<TransportOwned>,
    #[serde(flatten)]
    pub application: Option<Application>,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n    Destination MAC: {},\n    Source MAC: {},\n    Ethertype: {}\n    VLAN: {}\n",
            self.destination_mac,
            self.source_mac,
            self.ethertype,
            match &self.vlan {
                Some(vlan) => vlan.to_string(),
                None => "None".to_string(),
            },
        )
    }
}

impl Display for InternetOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let source_ip = match &self.source_ip {
            Some(ip) => ip.to_string(),
            None => "None".to_string(),
        };

        let destination_ip = match &self.destination_ip {
            Some(ip) => ip.to_string(),
            None => "None".to_string(),
        };

        write!(
            f,
            "\n    Source IP: {},\n    Destination IP: {},\n    Protocol: {}\n",
            source_ip, destination_ip, self.protocol
        )
    }
}

impl Display for TransportOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let source_port = match self.source_port {
            Some(port) => port.to_string(),
            None => "None".to_string(),
        };

        let destination_port = match self.destination_port {
            Some(port) => port.to_string(),
            None => "None".to_string(),
        };

        write!(
            f,
            "\n    Source Port: {},\n    Destination Port: {},\n    Protocol: {}\n",
            source_port, destination_port, self.protocol
        )
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

#[derive(Debug, Clone, Serialize, PartialEq)]
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
            internet: match flow.internet {
                Some(internet) => Some(InternetOwned {
                    source_ip: internet.source,
                    ip_source_type: internet.source_type,
                    destination_ip: internet.destination,
                    ip_destination_type: internet.destination_type,
                    protocol: internet.protocol_name,
                }),
                None => None,
            },
            transport: match flow.transport {
                Some(transport) => Some(TransportOwned {
                    source_port: transport.source_port,
                    destination_port: transport.destination_port,
                    protocol: transport.protocol.to_string(),
                }),
                None => None,
            },
            application: match flow.application {
                Some(application) => Some(Application {
                    application_protocol: application.application_protocol,
                }),
                None => None,
            },
        }
    }
}

use std::hash::Hash;
impl Hash for PacketFlowOwned {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_link.hash(state);
        self.internet.hash(state);
        self.transport.hash(state);
        self.application.hash(state);
    }
}

use std::fmt::{Formatter, Result};

impl Display for PacketFlowOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Packet Flow:")?;
        writeln!(f, "  Data Link: {}", self.data_link)?;

        if let Some(ref internet) = self.internet {
            writeln!(f, "  Internet: {internet}")?;
        }

        if let Some(ref transport) = self.transport {
            writeln!(f, "  Transport: {transport}")?;
        }

        if let Some(ref application) = self.application {
            writeln!(f, "  Application: {application}")?;
        }

        Ok(())
    }
}
