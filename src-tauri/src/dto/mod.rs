use std::net::IpAddr;

use pcap::{
    Address as PcapAddress, ConnectionStatus as PcapConnectionStatus, Device,
    DeviceFlags as PcapDeviceFlags, IfFlags as PcapIfFlags,
};
use serde::Serialize;

/// ====== DTO sérialisables pour l'IPC Tauri ======

#[derive(Debug, Serialize)]
pub struct NetDevice {
    pub name: String,
    pub desc: Option<String>,
    pub addresses: Vec<Address>,
    pub flags: DeviceFlags,
}

#[derive(Debug, Serialize)]
pub struct Address {
    pub addr: IpAddr,
    pub netmask: Option<IpAddr>,
    pub broadcast_addr: Option<IpAddr>,
    pub dst_addr: Option<IpAddr>,
}

#[derive(Debug, Serialize)]
pub struct DeviceFlags {
    pub if_flags: IfFlags,
    pub connection_status: ConnectionStatus,
}

/// `IfFlags` de pcap est un bitflag interne.
/// On sérialise la valeur brute pour l’UI.
/// (Tu pourras exposer des booléens dérivés si besoin.)
#[derive(Debug, Serialize)]
pub struct IfFlags {
    pub bits: u32,
}

#[derive(Debug, Serialize)]
pub enum ConnectionStatus {
    Unknown,
    Connected,
    Disconnected,
    NotApplicable,
}

/// ====== Conversions depuis pcap::* vers nos DTO ======

impl From<PcapAddress> for Address {
    fn from(a: PcapAddress) -> Self {
        Address {
            addr: a.addr,
            netmask: a.netmask,
            broadcast_addr: a.broadcast_addr,
            dst_addr: a.dst_addr,
        }
    }
}

impl From<PcapIfFlags> for IfFlags {
    fn from(f: PcapIfFlags) -> Self {
        IfFlags { bits: f.bits() }
    }
}

impl From<PcapConnectionStatus> for ConnectionStatus {
    fn from(s: PcapConnectionStatus) -> Self {
        match s {
            PcapConnectionStatus::Unknown => ConnectionStatus::Unknown,
            PcapConnectionStatus::Connected => ConnectionStatus::Connected,
            PcapConnectionStatus::Disconnected => ConnectionStatus::Disconnected,
            PcapConnectionStatus::NotApplicable => ConnectionStatus::NotApplicable,
        }
    }
}

impl From<PcapDeviceFlags> for DeviceFlags {
    fn from(df: PcapDeviceFlags) -> Self {
        DeviceFlags {
            if_flags: df.if_flags.into(),
            connection_status: df.connection_status.into(),
        }
    }
}

impl From<Device> for NetDevice {
    fn from(d: Device) -> Self {
        NetDevice {
            name: d.name,
            desc: d.desc,
            addresses: d.addresses.into_iter().map(Address::from).collect(),
            flags: d.flags.into(),
        }
    }
}

// TODO : RUST DOC
// #[derive(Debug, Error)]
// pub enum PcapError {
//     #[error("Impossible de lister les interfaces réseau")]
//     DeviceListError(#[from] pcap::Error),
// }
