//! DTO sérialisables pour l'IPC Tauri : miroirs des types `pcap::Device` et
//! associés, exposés au frontend par la commande `get_net_interfaces`.

use std::net::IpAddr;

use pcap::{
    Address as PcapAddress, ConnectionStatus as PcapConnectionStatus, Device,
    DeviceFlags as PcapDeviceFlags, IfFlags as PcapIfFlags,
};
use serde::Serialize;

/// Interface réseau telle que présentée au frontend.
#[derive(Debug, Serialize)]
pub struct NetDevice {
    pub name: String,
    pub desc: Option<String>,
    pub addresses: Vec<Address>,
    pub flags: DeviceFlags,
}

/// Adresse portée par une interface (IP, masque, broadcast, destination).
#[derive(Debug, Serialize)]
pub struct Address {
    pub addr: IpAddr,
    pub netmask: Option<IpAddr>,
    pub broadcast_addr: Option<IpAddr>,
    pub dst_addr: Option<IpAddr>,
}

/// Drapeaux d'une interface : bits bruts + état de connexion.
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

/// État de connexion d'une interface, tel que remonté par pcap.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Garde contre une variante `pcap::ConnectionStatus` ajoutée en amont
    /// sans son pendant dans le DTO : chaque variante doit se mapper vers
    /// elle-même, pas retomber silencieusement sur `Unknown`.
    #[test]
    fn connection_status_maps_each_variant_to_its_own_dto_variant() {
        assert!(matches!(
            ConnectionStatus::from(PcapConnectionStatus::Unknown),
            ConnectionStatus::Unknown
        ));
        assert!(matches!(
            ConnectionStatus::from(PcapConnectionStatus::Connected),
            ConnectionStatus::Connected
        ));
        assert!(matches!(
            ConnectionStatus::from(PcapConnectionStatus::Disconnected),
            ConnectionStatus::Disconnected
        ));
        assert!(matches!(
            ConnectionStatus::from(PcapConnectionStatus::NotApplicable),
            ConnectionStatus::NotApplicable
        ));
    }

    #[test]
    fn if_flags_carries_the_raw_bits_through() {
        let flags: IfFlags = PcapIfFlags::from_bits_truncate(0b101).into();
        assert_eq!(flags.bits, 0b101);
    }
}
