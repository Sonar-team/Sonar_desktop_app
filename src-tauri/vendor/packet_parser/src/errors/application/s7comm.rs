// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::Serialize;
use thiserror::Error;

/// Errors returned while validating a TPKT/COTP/S7Comm protocol data unit.
///
/// This enum is non-exhaustive so stricter wire validation can gain dedicated
/// errors without forcing downstream exhaustive matches to change.
#[non_exhaustive]
#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize)]
pub enum S7CommParseError {
    #[error("S7Comm packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("Invalid TPKT version: expected 0x03, got 0x{version:02x}")]
    InvalidTpktVersion { version: u8 },

    #[error("Invalid TPKT reserved byte: expected 0x00, got 0x{reserved:02x}")]
    InvalidTpktReserved { reserved: u8 },

    #[error("Invalid TPKT length: expected at least {minimum} bytes, declared {declared}")]
    InvalidTpktLength { declared: usize, minimum: usize },

    #[error("Truncated TPKT: declared {declared} bytes, only {actual} available")]
    TruncatedTpkt { declared: usize, actual: usize },

    #[error("Invalid COTP length indicator: expected 2 for a DT TPDU, got {length}")]
    InvalidCotpLengthIndicator { length: u8 },

    #[error("Invalid COTP PDU type: expected DT (0xf0), got 0x{pdu_type:02x}")]
    InvalidCotpPduType { pdu_type: u8 },

    #[error("Incomplete COTP DT TPDU: EOT (last data unit) is not set")]
    CotpNotLastDataUnit,

    #[error("Invalid COTP class-0 TPDU number: expected 0, got {tpdu_number}")]
    InvalidCotpTpduNumber { tpdu_number: u8 },

    #[error("Invalid COTP header length: expected at least {expected} bytes, got {actual}")]
    InvalidCotpHeaderLength { expected: usize, actual: usize },

    #[error("S7 header too short: expected at least {expected} bytes, got {actual}")]
    S7HeaderTooShort { expected: usize, actual: usize },

    #[error("Invalid S7 protocol identifier: expected 0x32, got 0x{protocol_id:02x}")]
    InvalidS7ProtocolId { protocol_id: u8 },

    #[error("Invalid S7 ROSCTR: expected 0x01, 0x02, 0x03, or 0x07, got 0x{rosctr:02x}")]
    InvalidS7Rosctr { rosctr: u8 },

    #[error("Invalid S7 reserved field: expected 0x0000, got 0x{reserved:04x}")]
    InvalidS7Reserved { reserved: u16 },

    #[error("Invalid S7 parameter length: expected end {expected}, packet length {actual}")]
    InvalidParameterLength { expected: usize, actual: usize },

    #[error("Invalid S7 data length: expected end {expected}, packet length {actual}")]
    InvalidDataLength { expected: usize, actual: usize },

    #[error(
        "Inconsistent S7 section lengths: sections end at {sections_end}, TPKT ends at {tpkt_end}"
    )]
    InconsistentSectionLengths {
        sections_end: usize,
        tpkt_end: usize,
    },

    #[error("Empty S7 parameter data")]
    EmptyParameterData,

    #[error("Invalid S7 parameter item header")]
    InvalidParameterItemHeader,

    #[error("Invalid S7 parameter item length")]
    InvalidParameterItemLength,

    #[error("Invalid S7 variable specification type: expected 0x12, got 0x{spec_type:02x}")]
    InvalidParameterSpecificationType { spec_type: u8 },

    #[error("S7 variable specification is missing its syntax identifier")]
    MissingParameterSyntaxId,

    #[error("Missing S7 Read/Write Var item count")]
    MissingParameterItemCount,

    #[error("Missing fill byte after an odd-sized S7 parameter item")]
    MissingParameterItemPadding,

    #[error("Unexpected bytes after S7 parameter items: {remaining} byte(s) remain")]
    UnexpectedParameterBytes { remaining: usize },

    #[error("Invalid S7ANY specification length: expected 10, got {length}")]
    InvalidS7AnyLength { length: u8 },

    #[error("S7Comm {context} offset arithmetic overflow")]
    LengthOverflow { context: &'static str },
}
