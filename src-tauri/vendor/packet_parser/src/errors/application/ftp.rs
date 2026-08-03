// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FtpParseError {
    #[error("Invalid UTF-8 in FTP payload")]
    InvalidUtf8,

    #[error("Empty FTP payload")]
    EmptyPayload,

    #[error("Incomplete FTP line: missing CRLF terminator")]
    IncompleteLine,

    #[error("Unknown FTP command: {0}")]
    UnknownCommand(String),

    #[error("Invalid FTP command syntax: {0}")]
    InvalidCommandSyntax(String),

    #[error("Invalid FTP reply code: {0}")]
    InvalidReplyCode(String),

    #[error("Multi-line FTP reply is missing its terminator line")]
    MissingReplyTerminator,
}
