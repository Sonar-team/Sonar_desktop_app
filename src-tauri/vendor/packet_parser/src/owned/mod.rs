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
