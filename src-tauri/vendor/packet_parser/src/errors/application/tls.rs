// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TlsError {
    #[error("TLS record too short")]
    TooShort,

    #[error("Invalid TLS content type: {0}")]
    InvalidContentType(u8),

    #[error("Invalid TLS version: {major}.{minor}")]
    InvalidVersion { major: u8, minor: u8 },

    #[error("TLS record length is inconsistent: declared {declared}, available {available}")]
    InconsistentLength { declared: u16, available: usize },
}
