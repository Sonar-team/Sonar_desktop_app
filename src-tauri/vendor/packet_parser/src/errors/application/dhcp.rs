// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DhcpParseError {
    #[error("DHCP packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("Invalid DHCP operation code: {op}")]
    InvalidOperation { op: u8 },

    #[error("Unsupported DHCP hardware type: {htype}")]
    UnsupportedHardwareType { htype: u8 },

    #[error("Invalid DHCP hardware address length: {hlen}")]
    InvalidHardwareAddressLength { hlen: u8 },
}
