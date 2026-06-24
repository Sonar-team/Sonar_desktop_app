// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModbusTcpError {
    #[error("Modbus/TCP buffer too small: needed {needed} bytes, got {actual}")]
    BufferTooSmall { needed: usize, actual: usize },

    #[error("Invalid Modbus/TCP protocol identifier: {got}")]
    InvalidProtocolIdentifier { got: u16 },

    #[error("Invalid Modbus/TCP length field: {got}")]
    InvalidLengthField { got: u16 },

    #[error("Modbus/TCP length mismatch: expected {expected} bytes, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },

    #[error("Modbus/TCP PDU too small: needed {needed} bytes, got {actual}")]
    PduTooSmall { needed: usize, actual: usize },
}
