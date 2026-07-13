// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Dhcpv6PacketParseError {
    #[error("Invalid DHCPv6 packet length")]
    PacketLength,

    #[error("Invalid DHCPv6 transaction ID")]
    TransactionId,

    #[error("Invalid DHCPv6 message type: {message_type}")]
    MessageType { message_type: u8 },
}
