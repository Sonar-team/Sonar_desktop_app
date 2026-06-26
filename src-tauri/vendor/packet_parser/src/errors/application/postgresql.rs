// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PostgreSqlError {
    #[error("PostgreSQL packet is empty")]
    EmptyPacket,

    #[error("PostgreSQL buffer too small: needed {needed} bytes, got {actual}")]
    BufferTooSmall { needed: usize, actual: usize },

    #[error("Invalid PostgreSQL message type: 0x{0:02X}")]
    InvalidMessageType(u8),

    #[error("Invalid PostgreSQL message length: {got}")]
    InvalidMessageLength { got: u32 },

    #[error("PostgreSQL message length mismatch: expected {expected} bytes, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },

    #[error("Invalid UTF-8 in PostgreSQL string field")]
    InvalidUtf8,

    #[error("Missing NUL terminator in PostgreSQL field {field}")]
    MissingNullTerminator { field: &'static str },

    #[error("Invalid PostgreSQL field length for {field}: {got}")]
    InvalidFieldLength { field: &'static str, got: i32 },

    #[error("Trailing bytes in PostgreSQL {message_type} message: {remaining}")]
    TrailingBytes {
        message_type: &'static str,
        remaining: usize,
    },

    #[error("Unsupported PostgreSQL startup code: {0}")]
    UnsupportedStartupCode(u32),
}
