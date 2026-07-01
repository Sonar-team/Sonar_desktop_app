// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GiopParseError {
    #[error("Invalid GIOP packet length")]
    InvalidSize,

    #[error("Invalid GIOP magic (expected 'GIOP')")]
    InvalidMagic,

    #[error("Unsupported GIOP version {0}.{1}")]
    UnsupportedVersion(u8, u8),

    #[error("Unknown GIOP message type {0}")]
    UnknownMessageType(u8),

    #[error("Truncated GIOP body (expected {expected} bytes, got {actual})")]
    TruncatedBody { expected: usize, actual: usize },

    #[error("Invalid UTF-8 in string field")]
    InvalidUtf8,

    #[error("Unexpected end of buffer")]
    UnexpectedEof,

    #[error("Other GIOP parsing error: {0}")]
    Other(&'static str),
}
