// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SrvlocPacketParseError {
    #[error("SRVLOC packet too short")]
    InvalidPacketLength,

    #[error("SRVLOC packet truncated: expected at least {expected_at_least} bytes, got {actual}")]
    Truncated {
        expected_at_least: usize,
        actual: usize,
    },

    #[error("Unsupported SRVLOC version {0}")]
    UnsupportedVersion(u8),

    #[error("Invalid UTF-8 in SRVLOC field '{0}'")]
    InvalidUtf8(&'static str),
}
