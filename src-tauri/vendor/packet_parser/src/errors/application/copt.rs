// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CotpParseError {
    #[error("COTP packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("COTP length exceeds packet: declared {declared} bytes, got {actual}")]
    LengthExceedsPacket { declared: usize, actual: usize },

    #[error("COTP connection header too short: expected at least {expected} bytes, got {actual}")]
    ConnectionHeaderTooShort { expected: usize, actual: usize },

    #[error("COTP parameter header truncated at offset {offset}")]
    ParameterHeaderTruncated { offset: usize },

    #[error(
        "COTP parameter length exceeds packet at offset {offset}: declared {declared} bytes, available {available}"
    )]
    ParameterLengthExceedsPacket {
        offset: usize,
        declared: usize,
        available: usize,
    },
}
