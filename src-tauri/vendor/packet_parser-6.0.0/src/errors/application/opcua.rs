// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OpcuaParseError {
    #[error("OPC UA packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("unknown OPC UA message type: {0:?}")]
    UnknownMessageType([u8; 3]),

    #[error("unknown OPC UA chunk type: 0x{0:02x}")]
    UnknownChunkType(u8),

    #[error("invalid OPC UA message size: {size}")]
    InvalidMessageSize { size: u32 },

    #[error("truncated OPC UA chunk: expected {expected} bytes, got {actual}")]
    TruncatedChunk { expected: usize, actual: usize },

    #[error("OPC UA body too short: expected at least {expected} bytes, got {actual}")]
    BodyTooShort { expected: usize, actual: usize },

    #[error("invalid OPC UA string length: {length}")]
    InvalidStringLength { length: i32 },

    #[error("truncated OPC UA string: expected {expected} bytes, got {actual}")]
    TruncatedString { expected: usize, actual: usize },

    #[error("invalid UTF-8 in OPC UA string")]
    InvalidUtf8,
}
