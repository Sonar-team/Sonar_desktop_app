// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::http::HttpParseError;

/// Separator between the head (request line + headers) and the body.
const HEAD_BODY_SEPARATOR: &str = "\r\n\r\n";

/// Validates that the payload is UTF-8 and returns it as a borrowed `&str`.
///
/// The validation is done once with `std::str::from_utf8`: no allocation,
/// no lossy replacement of invalid bytes.
pub fn parse_payload_as_utf8(payload: &[u8]) -> Result<&str, HttpParseError> {
    std::str::from_utf8(payload).map_err(|_| HttpParseError::InvalidUtf8)
}

/// Splits the payload into a borrowed head (request line + headers) and a
/// borrowed body, around the first `CRLF CRLF` separator.
///
/// If the separator is absent, the whole payload is the head and the body
/// is empty.
pub fn split_head_body(payload: &str) -> (&str, &str) {
    match payload.find(HEAD_BODY_SEPARATOR) {
        Some(index) => (
            &payload[..index],
            &payload[index + HEAD_BODY_SEPARATOR.len()..],
        ),
        None => (payload, ""),
    }
}

/// Standard HTTP request methods (RFC 9110) accepted by the parser.
const HTTP_METHODS: [&str; 9] = [
    "GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH",
];

/// Requires the HTTP method token to be present and to be a standard HTTP
/// method, then returns it borrowed. Rejecting unknown methods keeps random
/// text payloads from being classified as HTTP.
pub fn require_method(part: Option<&str>) -> Result<&str, HttpParseError> {
    let method = part.ok_or(HttpParseError::MissingMethod)?;
    if !HTTP_METHODS.contains(&method) {
        return Err(HttpParseError::InvalidMethod(method.to_string()));
    }
    Ok(method)
}

/// Requires the URI token to be present and returns it borrowed.
pub fn require_uri(part: Option<&str>) -> Result<&str, HttpParseError> {
    part.ok_or(HttpParseError::MissingUri)
}

/// Requires the HTTP version token to be present and shaped like `HTTP/x.y`,
/// then returns it borrowed.
pub fn require_version(part: Option<&str>) -> Result<&str, HttpParseError> {
    let version = part.ok_or(HttpParseError::MissingVersion)?;
    if !version.starts_with("HTTP/") {
        return Err(HttpParseError::InvalidVersion(version.to_string()));
    }
    Ok(version)
}

/// Requires a header name or value part to be present and returns it borrowed.
pub fn require_header_part(part: Option<&str>) -> Result<&str, HttpParseError> {
    part.ok_or(HttpParseError::InvalidHeader)
}

/// Checks that a header line contains a `:` separator and returns the
/// borrowed `(name, value)` pair, both trimmed of surrounding whitespace.
/// A line without a `:` yields `InvalidHeader`.
pub fn extract_header_line(line: &str) -> Result<(&str, &str), HttpParseError> {
    let mut header_parts = line.splitn(2, ':');
    let name = require_header_part(header_parts.next())?.trim();
    let value = require_header_part(header_parts.next())?.trim();
    Ok((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_payload_as_utf8_valid() {
        let payload = b"GET / HTTP/1.1";
        let result = parse_payload_as_utf8(payload).expect("valid UTF-8");
        assert_eq!(result, "GET / HTTP/1.1");
        // Zero-copy: the returned str borrows the input bytes.
        assert_eq!(result.as_ptr(), payload.as_ptr());
    }

    #[test]
    fn test_parse_payload_as_utf8_invalid() {
        let payload = [0x47, 0x45, 0x54, 0xFF, 0xFE];
        assert_eq!(
            parse_payload_as_utf8(&payload),
            Err(HttpParseError::InvalidUtf8)
        );
    }

    #[test]
    fn test_split_head_body_with_separator() {
        let payload = "GET / HTTP/1.1\r\nHost: a\r\n\r\nbody";
        let (head, body) = split_head_body(payload);
        assert_eq!(head, "GET / HTTP/1.1\r\nHost: a");
        assert_eq!(body, "body");
    }

    #[test]
    fn test_split_head_body_without_separator() {
        let payload = "GET / HTTP/1.1\r\nHost: a";
        let (head, body) = split_head_body(payload);
        assert_eq!(head, payload);
        assert_eq!(body, "");
    }

    #[test]
    fn test_split_head_body_empty_body() {
        let payload = "GET / HTTP/1.1\r\n\r\n";
        let (head, body) = split_head_body(payload);
        assert_eq!(head, "GET / HTTP/1.1");
        assert_eq!(body, "");
    }

    #[test]
    fn test_require_method() {
        assert_eq!(require_method(Some("GET")), Ok("GET"));
        assert_eq!(require_method(None), Err(HttpParseError::MissingMethod));
    }

    #[test]
    fn test_require_uri() {
        assert_eq!(require_uri(Some("/index.html")), Ok("/index.html"));
        assert_eq!(require_uri(None), Err(HttpParseError::MissingUri));
    }

    #[test]
    fn test_require_version() {
        assert_eq!(require_version(Some("HTTP/1.1")), Ok("HTTP/1.1"));
        assert_eq!(require_version(None), Err(HttpParseError::MissingVersion));
    }

    #[test]
    fn test_require_header_part() {
        assert_eq!(require_header_part(Some("Host")), Ok("Host"));
        assert_eq!(
            require_header_part(None),
            Err(HttpParseError::InvalidHeader)
        );
    }

    #[test]
    fn test_extract_header_line_valid() {
        assert_eq!(
            extract_header_line("Host: www.example.com"),
            Ok(("Host", "www.example.com"))
        );
        // No space after the colon, and surrounding whitespace is trimmed.
        assert_eq!(extract_header_line("Accept:*/*"), Ok(("Accept", "*/*")));
        assert_eq!(extract_header_line("  Host :  a  "), Ok(("Host", "a")));
    }

    #[test]
    fn test_extract_header_line_keeps_extra_colons_in_value() {
        assert_eq!(
            extract_header_line("Referer: http://example.com/"),
            Ok(("Referer", "http://example.com/"))
        );
    }

    #[test]
    fn test_extract_header_line_missing_colon() {
        assert_eq!(
            extract_header_line("NotAHeader"),
            Err(HttpParseError::InvalidHeader)
        );
    }
}
