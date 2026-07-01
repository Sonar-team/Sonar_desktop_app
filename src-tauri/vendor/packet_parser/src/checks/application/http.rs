// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::http::HttpParseError;

pub fn parse_payload_as_utf8(payload: &[u8]) -> Result<&str, HttpParseError> {
    std::str::from_utf8(payload).map_err(|_| HttpParseError::InvalidUtf8)
}

pub fn require_request_line(line: Option<&str>) -> Result<&str, HttpParseError> {
    line.ok_or(HttpParseError::MissingRequestLine)
}

pub fn require_method(part: Option<&str>) -> Result<&str, HttpParseError> {
    part.ok_or(HttpParseError::MissingMethod)
}

pub fn require_uri(part: Option<&str>) -> Result<&str, HttpParseError> {
    part.ok_or(HttpParseError::MissingUri)
}

pub fn require_version(part: Option<&str>) -> Result<&str, HttpParseError> {
    part.ok_or(HttpParseError::MissingVersion)
}

pub fn require_header_part(part: Option<&str>) -> Result<&str, HttpParseError> {
    part.ok_or(HttpParseError::InvalidHeader)
}
