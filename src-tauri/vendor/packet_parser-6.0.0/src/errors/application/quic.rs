// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

/// Error types for QUIC packet parsing (RFC 9000 / RFC 9001, Long Header).
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum QuicError {
    #[error("Packet truncated: needed {needed} more bytes, only {remaining} remaining")]
    Truncated { needed: usize, remaining: usize },

    #[error("Truncated varint: needed {needed} continuation bytes, only {remaining} remaining")]
    TruncatedVarint { needed: usize, remaining: usize },

    #[error("Not a Long Header packet: header form bit must be 1")]
    NotLongHeader,

    #[error("Fixed bit is 0: must be 1 per RFC 9000 §17.2")]
    FixedBitNotSet,

    #[error("Unsupported QUIC version: {0:#010x}")]
    UnsupportedVersion(u32),

    #[error("Length field {length_field} is smaller than packet number length {pn_length}")]
    LengthFieldTooSmall { length_field: u64, pn_length: u8 },

    #[error("Payload too short: expected {expected} bytes, only {available} available")]
    PayloadTooShort { expected: usize, available: usize },
}
