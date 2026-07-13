// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

use crate::parse::application::protocols::snmp::SnmpVersion;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SnmpError {
    #[error("SNMP packet too short: min {min} bytes, got {actual}")]
    PacketTooShort { min: usize, actual: usize },

    #[error("SNMP packet truncated in {field}: needed at least {needed} bytes, got {actual}")]
    Truncated {
        field: &'static str,
        needed: usize,
        actual: usize,
    },

    #[error("Unsupported BER indefinite length in {field}")]
    UnsupportedIndefiniteLength { field: &'static str },

    #[error("BER length uses too many bytes in {field}: {actual}")]
    UnsupportedLengthSize { field: &'static str, actual: usize },

    #[error("BER length overflow in {field}")]
    LengthOverflow { field: &'static str },

    #[error("Invalid SNMP ASN.1 tag for {field}: expected 0x{expected:02X}, got 0x{actual:02X}")]
    InvalidTag {
        field: &'static str,
        expected: u8,
        actual: u8,
    },

    #[error("Invalid SNMP top-level length: consumed {consumed}, packet length {packet_len}")]
    TrailingData { consumed: usize, packet_len: usize },

    #[error("Invalid SNMP integer length in {field}: {actual}")]
    InvalidIntegerLength { field: &'static str, actual: usize },

    #[error("Invalid SNMP unsigned integer length in {field}: {actual}")]
    InvalidUnsignedLength { field: &'static str, actual: usize },

    #[error("Unsigned integer overflow in {field}")]
    UnsignedOverflow { field: &'static str },

    #[error("Unsupported SNMP version {version}")]
    UnsupportedVersion { version: i64 },

    #[error("Unsupported SNMP PDU tag 0x{tag:02X} for version {version:?}")]
    UnsupportedPduType { tag: u8, version: SnmpVersion },

    #[error("Invalid SNMP PDU structure: {0}")]
    InvalidPduStructure(&'static str),

    #[error("Invalid SNMP IP address length: {actual}")]
    InvalidIpAddressLength { actual: usize },
}
