// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::data_link::{DataLinkError, mac_addres::MacParseError},
    parse::data_link::mac_addres::MAC_LEN,
};

const DATALINK_HEADER_LEN: usize = 14;
const DATALINK_VLAN_HEADER_LEN: usize = 18;
const VLAN_TAG_LEN: usize = 4;

pub fn validate_data_link_length(packets: &[u8]) -> Result<(), DataLinkError> {
    if packets.len() < DATALINK_HEADER_LEN {
        return Err(DataLinkError::DataLinkTooShort(packets.len() as u8));
    }
    Ok(())
}

pub fn validate_data_link_vlan_length(packets: &[u8]) -> Result<(), DataLinkError> {
    if packets.len() < DATALINK_VLAN_HEADER_LEN {
        return Err(DataLinkError::DataLinkTooShort(packets.len() as u8));
    }
    Ok(())
}

pub fn validate_vlan_tag_length(bytes: &[u8]) -> Result<(), DataLinkError> {
    if bytes.len() < VLAN_TAG_LEN {
        return Err(DataLinkError::DataLinkTooShort(bytes.len() as u8));
    }
    Ok(())
}

pub fn validate_mac_length(packets: &[u8]) -> Result<(), MacParseError> {
    if packets.len() != MAC_LEN {
        return Err(MacParseError::InvalidLength {
            actual: packets.len(),
        });
    }
    Ok(())
}
