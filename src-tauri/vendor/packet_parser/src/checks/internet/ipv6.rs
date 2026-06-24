// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::internet::ipv6::Ipv6Error;

pub const IPV6_HEADER_LEN: usize = 40;

pub fn validate_ipv6_header_length(data: &[u8]) -> Result<(), Ipv6Error> {
    if data.len() < IPV6_HEADER_LEN {
        return Err(Ipv6Error::InvalidLength {
            expected: IPV6_HEADER_LEN,
            actual: data.len(),
        });
    }
    Ok(())
}

pub fn validate_ipv6_version(version: u8) -> Result<(), Ipv6Error> {
    if version != 6 {
        return Err(Ipv6Error::InvalidVersion(version));
    }
    Ok(())
}

pub fn validate_ipv6_payload_length(
    data_len: usize,
    payload_length: u16,
) -> Result<usize, Ipv6Error> {
    let total_expected_len = IPV6_HEADER_LEN + payload_length as usize;
    if data_len < total_expected_len {
        return Err(Ipv6Error::InvalidPayloadLength {
            expected: payload_length,
            actual: data_len.saturating_sub(IPV6_HEADER_LEN),
        });
    }
    Ok(total_expected_len)
}
