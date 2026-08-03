// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NntpParseError {
    #[error("Invalid UTF-8 in NNTP payload")]
    InvalidUtf8,

    #[error("Empty NNTP payload")]
    EmptyPayload,

    #[error("Incomplete NNTP line: missing CRLF terminator")]
    IncompleteLine,

    #[error("Unknown NNTP command: {0}")]
    UnknownCommand(String),

    #[error("Invalid NNTP command syntax: {0}")]
    InvalidCommandSyntax(String),

    #[error("Invalid NNTP response code: {0}")]
    InvalidResponseCode(String),
}
