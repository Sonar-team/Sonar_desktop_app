// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CotpParseError {
    #[error("COTP packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("COTP length exceeds packet: declared {declared} bytes, got {actual}")]
    LengthExceedsPacket { declared: usize, actual: usize },

    #[error("invalid COTP length indicator 0x{length:02x}: expected 1..=254")]
    InvalidLengthIndicator { length: u8 },

    #[error("COTP {context} offset arithmetic overflow")]
    LengthOverflow { context: &'static str },

    #[error("COTP connection header too short: expected at least {expected} bytes, got {actual}")]
    ConnectionHeaderTooShort { expected: usize, actual: usize },

    #[error("COTP {pdu} fixed header too short: expected at least {expected} bytes, got {actual}")]
    FixedHeaderTooShort {
        pdu: &'static str,
        expected: usize,
        actual: usize,
    },

    #[error("invalid COTP {pdu} header length indicator {length}")]
    InvalidTpduHeaderLength { pdu: &'static str, length: u8 },

    #[error("invalid COTP protocol class {class}: expected 0..=4")]
    InvalidProtocolClass { class: u8 },

    #[error("invalid COTP class/options byte 0x{value:02x}: reserved bits 2 and 3 must be zero")]
    ReservedClassOptionBits { value: u8 },

    #[error("COTP option {option} is not valid for class {class}")]
    InvalidClassOption { class: u8, option: &'static str },

    #[error("invalid COTP initial credit {credit} for class {class}: expected zero")]
    InvalidInitialCredit { class: u8, credit: u8 },

    #[error("invalid COTP CR destination reference 0x{destination_reference:04x}: expected zero")]
    InvalidCrDestinationReference { destination_reference: u16 },

    #[error("invalid COTP {pdu} {field} reference 0x{value:04x}: expected non-zero")]
    InvalidConnectionReference {
        pdu: &'static str,
        field: &'static str,
        value: u16,
    },

    #[error("invalid COTP DR disconnect reason 0x{reason:02x}")]
    InvalidDisconnectReason { reason: u8 },

    #[error("invalid COTP ER reject cause 0x{cause:02x}")]
    InvalidRejectCause { cause: u8 },

    #[error("invalid COTP TPDU-size code 0x{code:02x}: expected 0x07..=0x0d")]
    InvalidTpduSizeCode { code: u8 },

    #[error("invalid COTP parameter 0x{code:02x} value length: expected {expected}, got {actual}")]
    InvalidParameterValueLength {
        code: u8,
        expected: usize,
        actual: usize,
    },

    #[error("invalid COTP parameter 0x{code:02x} value length: expected {expected}, got {actual}")]
    InvalidParameterLength {
        code: u8,
        expected: &'static str,
        actual: usize,
    },

    #[error("unexpected COTP parameter 0x{code:02x} in {pdu}")]
    UnexpectedParameterCode { pdu: &'static str, code: u8 },

    #[error("COTP {pdu} requires the EOT bit")]
    MissingEot { pdu: &'static str },

    #[error("invalid COTP {pdu} sequence number: reserved high bit must be zero")]
    ReservedSequenceNumberBit { pdu: &'static str },

    #[error("invalid COTP extended-format {pdu} code 0x{code:02x}: low credit nibble must be zero")]
    InvalidExtendedCreditCode { pdu: &'static str, code: u8 },

    #[error("COTP {pdu} user data is not permitted for class {class}")]
    UserDataNotAllowed { pdu: &'static str, class: u8 },

    #[error("COTP {pdu} user data too long: maximum {maximum} bytes, got {actual}")]
    UserDataTooLong {
        pdu: &'static str,
        maximum: usize,
        actual: usize,
    },

    #[error("COTP {pdu} user data too short: expected at least {minimum} byte(s), got {actual}")]
    UserDataTooShort {
        pdu: &'static str,
        minimum: usize,
        actual: usize,
    },

    #[error("COTP {pdu} too long: maximum {maximum} bytes, got {actual}")]
    PduTooLong {
        pdu: &'static str,
        maximum: usize,
        actual: usize,
    },

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
