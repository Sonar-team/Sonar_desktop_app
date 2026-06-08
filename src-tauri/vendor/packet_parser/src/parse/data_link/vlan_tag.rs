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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_vlan_tag_try_from_valid_ipv4() {
        // TCI :
        // PCP = 5  -> 101
        // DEI = 1  -> 1
        // VID = 100 -> 0x064
        //
        // tci = (5 << 13) | (1 << 12) | 100
        let tci: u16 = (5 << 13) | (1 << 12) | 100;
        let tci_bytes = tci.to_be_bytes();

        // EtherType interne IPv4 = 0x0800
        let bytes = [tci_bytes[0], tci_bytes[1], 0x08, 0x00];

        let vlan = VlanTag::try_from(bytes.as_slice()).unwrap();

        assert_eq!(vlan.id, 100);
        assert_eq!(vlan.pcp, 5);
        assert!(vlan.dei);
        assert_eq!(vlan.inner_ethertype, Ethertype::from(0x0800));
    }

    #[test]
    fn test_vlan_tag_try_from_valid_ipv6() {
        // PCP = 0, DEI = 0, VID = 4095
        let tci: u16 = 0x0FFF;
        let tci_bytes = tci.to_be_bytes();

        // EtherType interne IPv6 = 0x86DD
        let bytes = [tci_bytes[0], tci_bytes[1], 0x86, 0xDD];

        let vlan = VlanTag::try_from(bytes.as_slice()).unwrap();

        assert_eq!(vlan.id, 4095);
        assert_eq!(vlan.pcp, 0);
        assert!(!vlan.dei);
        assert_eq!(vlan.inner_ethertype, Ethertype::from(0x86DD));
    }

    #[test]
    fn test_vlan_tag_try_from_valid_zero_vid() {
        // PCP = 3, DEI = 0, VID = 0
        let tci: u16 = 3 << 13;
        let tci_bytes = tci.to_be_bytes();

        let bytes = [tci_bytes[0], tci_bytes[1], 0x08, 0x00];

        let vlan = VlanTag::try_from(bytes.as_slice()).unwrap();

        assert_eq!(vlan.id, 0);
        assert_eq!(vlan.pcp, 3);
        assert!(!vlan.dei);
        assert_eq!(vlan.inner_ethertype, Ethertype::from(0x0800));
    }

    #[test]
    fn test_vlan_tag_try_from_too_short_empty() {
        let err = VlanTag::try_from(&[][..]).unwrap_err();
        assert_eq!(
            err,
            crate::errors::data_link::DataLinkError::DataLinkTooShort(0)
        );
    }

    #[test]
    fn test_vlan_tag_try_from_too_short_three_bytes() {
        let err = VlanTag::try_from(&[0x00, 0x01, 0x08][..]).unwrap_err();
        assert_eq!(
            err,
            crate::errors::data_link::DataLinkError::DataLinkTooShort(3)
        );
    }

    #[test]
    fn test_inner_ethertype_name() {
        let vlan = VlanTag {
            id: 10,
            pcp: 1,
            dei: false,
            inner_ethertype: Ethertype::from(0x0800),
        };

        assert_eq!(vlan.inner_ethertype_name(), vlan.inner_ethertype.name());
    }

    #[test]
    fn test_serialize_skips_inner_ethertype() {
        let vlan = VlanTag {
            id: 42,
            pcp: 6,
            dei: true,
            inner_ethertype: Ethertype::from(0x0800),
        };

        let json = serde_json::to_string(&vlan).unwrap();

        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"pcp\":6"));
        assert!(json.contains("\"dei\":true"));
        assert!(!json.contains("inner_ethertype"));
    }

    #[test]
    fn test_clone_and_eq() {
        let vlan1 = VlanTag {
            id: 123,
            pcp: 4,
            dei: true,
            inner_ethertype: Ethertype::from(0x86DD),
        };

        let vlan2 = vlan1.clone();

        assert_eq!(vlan1, vlan2);
    }
}
