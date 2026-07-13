// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum S7CommParseError {
    #[error("S7Comm packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("Invalid TPKT version: expected 0x03, got 0x{version:02x}")]
    InvalidTpktVersion { version: u8 },

    #[error("Invalid COTP header length: expected at least {expected} bytes, got {actual}")]
    InvalidCotpHeaderLength { expected: usize, actual: usize },

    #[error("S7 header too short: expected at least {expected} bytes, got {actual}")]
    S7HeaderTooShort { expected: usize, actual: usize },

    #[error("Invalid S7 parameter length: expected end {expected}, packet length {actual}")]
    InvalidParameterLength { expected: usize, actual: usize },

    #[error("Invalid S7 data length: expected end {expected}, packet length {actual}")]
    InvalidDataLength { expected: usize, actual: usize },

    #[error("Empty S7 parameter data")]
    EmptyParameterData,

    #[error("Invalid S7 parameter item header")]
    InvalidParameterItemHeader,

    #[error("Invalid S7 parameter item length")]
    InvalidParameterItemLength,

    #[error("S7ANY parameter too short")]
    S7AnyParameterTooShort,
}
