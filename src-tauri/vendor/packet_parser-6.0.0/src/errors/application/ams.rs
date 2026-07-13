// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AmsParseError {
    #[error("AMS header too short: expected at least {expected} bytes, got {actual}")]
    HeaderTooShort { expected: usize, actual: usize },

    #[error("AMS payload length ({cb_data}) does not match actual data length ({actual})")]
    InvalidCbDataLength { cb_data: u32, actual: usize },

    #[error("Unknown AMS command id: 0x{0:04x}")]
    UnknownCommand(u16),

    #[error("Invalid AMS state flags: reserved bits set (0x{0:04x})")]
    InvalidStateFlags(u16),
}
