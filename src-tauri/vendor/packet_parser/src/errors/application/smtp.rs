// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SmtpParseError {
    #[error("Invalid UTF-8 in SMTP payload")]
    InvalidUtf8,

    #[error("Empty SMTP payload")]
    EmptyPayload,

    #[error("Incomplete SMTP line: missing CRLF terminator")]
    IncompleteLine,

    #[error("Unknown SMTP command: {0}")]
    UnknownCommand(String),

    #[error("Invalid SMTP command syntax: {0}")]
    InvalidCommandSyntax(String),

    #[error("Invalid SMTP reply code: {0}")]
    InvalidReplyCode(String),

    #[error("Multi-line SMTP reply is missing its terminator line")]
    MissingReplyTerminator,

    #[error("Invalid SMTP multi-line reply: {0}")]
    InvalidMultilineReply(String),
}
