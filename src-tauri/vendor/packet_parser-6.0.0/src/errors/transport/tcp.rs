// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TcpError {
    #[error("Packet too short to be a valid TCP header")]
    PacketTooShort,

    #[error("Invalid data offset: {0}")]
    InvalidDataOffset(u8),

    #[error("Invalid TCP header length")]
    InvalidHeaderLength,
}
