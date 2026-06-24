// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HttpParseError {
    #[error("Invalid UTF-8 in HTTP request")]
    InvalidUtf8,

    #[error("Missing HTTP request line")]
    MissingRequestLine,

    #[error("Missing HTTP method")]
    MissingMethod,

    #[error("Missing HTTP URI")]
    MissingUri,

    #[error("Missing HTTP version")]
    MissingVersion,

    #[error("Invalid HTTP header")]
    InvalidHeader,
}
