// src/parse/data_link/vlan_tag.rs (par ex.)

use serde::Serialize;

use super::ethertype::Ethertype; // adapte le chemin si besoin

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct VlanTag {
    /// VLAN ID sur 12 bits (0–4095)
    pub id: u16,
    /// Priority Code Point (0–7)
    pub pcp: u8,
    /// Drop Eligible Indicator
    pub dei: bool,
    /// EtherType interne (couche L3 réelle)
    #[serde(skip_serializing)]
    pub inner_ethertype: Ethertype,
}

impl VlanTag {
    /// Nom lisible de l'EtherType interne (IPv4, IPv6, etc.)
    pub fn inner_ethertype_name(&self) -> String {
        self.inner_ethertype.name()
    }
}

impl TryFrom<&[u8]> for VlanTag {
    type Error = crate::errors::data_link::DataLinkError; // adapte si tu as un VlanError

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // On attend au moins TCI (2 octets) + EtherType interne (2 octets)
        if bytes.len() < 4 {
            return Err(Self::Error::DataLinkTooShort(bytes.len() as u8));
        }

        let tci = u16::from_be_bytes([bytes[0], bytes[1]]);
        let pcp = ((tci & 0b1110_0000_0000_0000) >> 13) as u8;
        let dei = ((tci & 0b0001_0000_0000_0000) >> 12) != 0;
        let id = tci & 0x0FFF;

        let inner_ethertype_raw = u16::from_be_bytes([bytes[2], bytes[3]]);
        let inner_ethertype = Ethertype::from(inner_ethertype_raw);

        Ok(Self {
            id,
            pcp,
            dei,
            inner_ethertype,
        })
    }
}
