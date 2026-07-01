// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EtherNetIpError {
    #[error("EtherNet/IP packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("Unknown EtherNet/IP command 0x{command:04X}")]
    UnknownCommand { command: u16 },

    #[error("EtherNet/IP length mismatch: expected {expected} bytes, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },

    #[error("Invalid EtherNet/IP options field: expected 0, got 0x{options:08X}")]
    InvalidOptions { options: u32 },

    #[error("EtherNet/IP command {command} requires empty command data, got {actual} bytes")]
    UnexpectedCommandData {
        command: &'static str,
        actual: usize,
    },

    #[error("Invalid RegisterSession payload length: expected {expected} bytes, got {actual}")]
    InvalidRegisterSessionLength { expected: usize, actual: usize },

    #[error("Unsupported EtherNet/IP encapsulation protocol version {version}")]
    UnsupportedProtocolVersion { version: u16 },

    #[error("Invalid RegisterSession options flags: expected 0, got 0x{flags:04X}")]
    InvalidRegisterSessionOptions { flags: u16 },

    #[error("EtherNet/IP CPF truncated in {field}: needed at least {needed} bytes, got {actual}")]
    Truncated {
        field: &'static str,
        needed: usize,
        actual: usize,
    },

    #[error("Invalid EtherNet/IP interface handle: expected 0, got 0x{interface_handle:08X}")]
    InvalidInterfaceHandle { interface_handle: u32 },

    #[error("EtherNet/IP CPF has trailing bytes: consumed {consumed}, actual {actual}")]
    TrailingCpfData { consumed: usize, actual: usize },
}
