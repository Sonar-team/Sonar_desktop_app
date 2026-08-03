// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::nntp::NntpParseError;

/// Standard NNTP commands (RFC 3977 and common extensions) accepted by the
/// parser. Restricting to known verbs keeps random text payloads from being
/// classified as NNTP.
const NNTP_COMMANDS: [&str; 26] = [
    "CAPABILITIES",
    "ARTICLE",
    "BODY",
    "HEAD",
    "STAT",
    "GROUP",
    "LISTGROUP",
    "HELP",
    "IHAVE",
    "LAST",
    "LIST",
    "MODE",
    "NEWGROUPS",
    "NEWNEWS",
    "NEXT",
    "POST",
    "QUIT",
    "SLAVE",
    "XGTITLE",
    "XHDR",
    "XOVER",
    "OVER",
    "HDR",
    "XPAT",
    "AUTHINFO",
    "DATE",
];

/// Validates that the payload contains at least one byte before any parsing
/// takes place.
pub fn validate_non_empty(payload: &[u8]) -> Result<(), NntpParseError> {
    if payload.is_empty() {
        return Err(NntpParseError::EmptyPayload);
    }
    Ok(())
}

/// Validates that the payload is UTF-8 and returns it as a borrowed `&str`.
pub fn parse_payload_as_utf8(payload: &[u8]) -> Result<&str, NntpParseError> {
    std::str::from_utf8(payload).map_err(|_| NntpParseError::InvalidUtf8)
}

/// Splits off the first CRLF-terminated line at the byte level. The remainder
/// is intentionally not decoded because it may be an NNTP data block rather
/// than another UTF-8 protocol line.
pub fn split_first_line(payload: &[u8]) -> Result<(&[u8], &[u8]), NntpParseError> {
    let Some(index) = payload.windows(2).position(|window| window == b"\r\n") else {
        return Err(NntpParseError::IncompleteLine);
    };
    Ok((&payload[..index], &payload[index + 2..]))
}

/// Splits a command line into its verb and optional arguments. RFC 3977 allows
/// spaces and tabs as separators; both returned values still borrow the input.
pub fn split_command(first_line: &str) -> (&str, Option<&str>) {
    let Some(index) = first_line
        .as_bytes()
        .iter()
        .position(|byte| matches!(byte, b' ' | b'\t'))
    else {
        return (first_line, None);
    };

    let args = first_line[index..].trim_matches(|character| matches!(character, ' ' | '\t'));
    (&first_line[..index], (!args.is_empty()).then_some(args))
}

/// Requires the command verb to be a known NNTP command, returning it borrowed.
pub fn require_known_command(verb: &str) -> Result<&str, NntpParseError> {
    if NNTP_COMMANDS
        .iter()
        .any(|command| command.eq_ignore_ascii_case(verb))
    {
        Ok(verb)
    } else {
        Err(NntpParseError::UnknownCommand(verb.to_string()))
    }
}

/// Performs minimum command-specific arity checks from RFC 3977. Arguments
/// remain opaque borrowed text after their token count has been checked.
pub fn validate_command_syntax(
    verb: &str,
    args: Option<&str>,
    line: &str,
) -> Result<(), NntpParseError> {
    let count = args.map_or(0, |value| value.split_ascii_whitespace().count());
    let valid = if [
        "CAPABILITIES",
        "DATE",
        "HELP",
        "LAST",
        "NEXT",
        "POST",
        "QUIT",
        "SLAVE",
    ]
    .iter()
    .any(|command| command.eq_ignore_ascii_case(verb))
    {
        count == 0
    } else if ["GROUP", "IHAVE", "MODE"]
        .iter()
        .any(|command| command.eq_ignore_ascii_case(verb))
    {
        count == 1
    } else if [
        "ARTICLE", "BODY", "HEAD", "STAT", "OVER", "XOVER", "XGTITLE",
    ]
    .iter()
    .any(|command| command.eq_ignore_ascii_case(verb))
    {
        count <= 1
    } else if verb.eq_ignore_ascii_case("LIST") || verb.eq_ignore_ascii_case("LISTGROUP") {
        count <= 2
    } else if verb.eq_ignore_ascii_case("HDR") || verb.eq_ignore_ascii_case("XHDR") {
        (1..=2).contains(&count)
    } else if verb.eq_ignore_ascii_case("XPAT") {
        count >= 3
    } else if verb.eq_ignore_ascii_case("NEWGROUPS") {
        (2..=4).contains(&count)
    } else if verb.eq_ignore_ascii_case("NEWNEWS") {
        (3..=5).contains(&count)
    } else if verb.eq_ignore_ascii_case("AUTHINFO") {
        count >= 2
    } else {
        true
    };

    if valid {
        Ok(())
    } else {
        Err(NntpParseError::InvalidCommandSyntax(line.to_string()))
    }
}

/// Returns whether a line starts like an NNTP status response. This is broader
/// than [`is_response_line`] so malformed numeric responses surface as response
/// errors rather than unknown commands.
pub fn looks_like_response(line: &str) -> bool {
    line.as_bytes().first().is_some_and(u8::is_ascii_digit)
}

/// Checks whether `line` has the shape of an NNTP status line: three valid
/// status digits, optionally followed by a space and response text.
///
/// This is exactly the guard enforced by [`parse_response_code`], exposed so
/// the parser can dispatch response vs. command without using an error as
/// control flow.
pub fn is_response_line(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.len() < 3 || !bytes[..3].iter().all(u8::is_ascii_digit) {
        return false;
    }
    if bytes.len() > 3 && bytes[3] != b' ' {
        return false;
    }

    let code = bytes[..3]
        .iter()
        .fold(0u16, |acc, &digit| acc * 10 + u16::from(digit - b'0'));
    (100..=599).contains(&code)
}

/// Parses a 3-digit NNTP response code from the start of `line`, returning
/// the code and the text that follows its optional single-space separator.
///
/// Unlike FTP/SMTP, NNTP status lines are never split with a `-` continuation
/// marker (RFC 3977 §3.1): a response is exactly one line.
pub fn parse_response_code(line: &str) -> Result<(u16, &str), NntpParseError> {
    if !is_response_line(line) {
        return Err(NntpParseError::InvalidResponseCode(line.to_string()));
    }
    // `is_response_line` proved the first three bytes are ASCII digits, so the
    // code fits in a u16 (at most 999) and byte slicing at offsets 3 and 4 is
    // safe on the `&str`.
    let code = line.as_bytes()[..3]
        .iter()
        .fold(0u16, |acc, &digit| acc * 10 + u16::from(digit - b'0'));
    let text = if line.len() == 3 { "" } else { &line[4..] };
    Ok((code, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert_eq!(validate_non_empty(b"MODE READER\r\n"), Ok(()));
        assert_eq!(validate_non_empty(&[]), Err(NntpParseError::EmptyPayload));
    }

    #[test]
    fn test_parse_payload_as_utf8_invalid() {
        assert_eq!(
            parse_payload_as_utf8(&[0xFF, 0xFE]),
            Err(NntpParseError::InvalidUtf8)
        );
    }

    #[test]
    fn test_split_first_line_with_separator() {
        assert_eq!(
            split_first_line(b"MODE READER\r\nXOVER\r\n"),
            Ok((&b"MODE READER"[..], &b"XOVER\r\n"[..]))
        );
    }

    #[test]
    fn test_split_first_line_without_separator() {
        assert_eq!(
            split_first_line(b"MODE READER"),
            Err(NntpParseError::IncompleteLine)
        );
    }

    #[test]
    fn test_split_command() {
        assert_eq!(split_command("MODE READER"), ("MODE", Some("READER")));
        assert_eq!(split_command("MODE\tREADER"), ("MODE", Some("READER")));
        assert_eq!(
            split_command("HDR\tSubject\t12-20"),
            ("HDR", Some("Subject\t12-20"))
        );
        assert_eq!(split_command("XOVER"), ("XOVER", None));
        // A trailing space yields empty arguments, normalized to `None`.
        assert_eq!(split_command("XOVER "), ("XOVER", None));
        assert_eq!(split_command(""), ("", None));
    }

    #[test]
    fn test_is_response_line() {
        assert!(is_response_line("224 data follows"));
        assert!(is_response_line("224"));
        assert!(!is_response_line("224-data follows"));
        assert!(!is_response_line("22"));
        assert!(!is_response_line("abcd"));
        assert!(!is_response_line("MODE READER"));
        assert!(!is_response_line("999 impossible"));
    }

    #[test]
    fn test_require_known_command() {
        assert_eq!(require_known_command("XOVER"), Ok("XOVER"));
        assert_eq!(require_known_command("xover"), Ok("xover"));
        assert_eq!(require_known_command("CAPABILITIES"), Ok("CAPABILITIES"));
        assert_eq!(require_known_command("hdr"), Ok("hdr"));
        assert_eq!(
            require_known_command("BONJOUR"),
            Err(NntpParseError::UnknownCommand("BONJOUR".to_string()))
        );
    }

    #[test]
    fn test_parse_response_code() {
        assert_eq!(
            parse_response_code("224 data follows"),
            Ok((224, "data follows"))
        );
        assert_eq!(parse_response_code("224"), Ok((224, "")));
    }

    #[test]
    fn test_parse_response_code_invalid() {
        assert!(parse_response_code("abc data follows").is_err());
        assert!(parse_response_code("22").is_err());
        assert!(parse_response_code("224-data follows").is_err());
        assert!(parse_response_code("999 impossible").is_err());
    }

    #[test]
    fn test_validate_command_syntax() {
        assert_eq!(
            validate_command_syntax("CAPABILITIES", None, "CAPABILITIES"),
            Ok(())
        );
        assert!(validate_command_syntax("CAPABILITIES", Some("extra"), "bad").is_err());
        assert_eq!(
            validate_command_syntax("HDR", Some("Subject"), "HDR Subject"),
            Ok(())
        );
        assert!(validate_command_syntax("HDR", None, "HDR").is_err());
        assert_eq!(validate_command_syntax("ARTICLE", None, "ARTICLE"), Ok(()));
        assert!(validate_command_syntax("ARTICLE", Some("1 2"), "ARTICLE 1 2").is_err());
        assert_eq!(
            validate_command_syntax("AUTHINFO", Some("USER alice"), "AUTHINFO USER alice"),
            Ok(())
        );
        assert!(validate_command_syntax("AUTHINFO", Some("USER"), "AUTHINFO USER").is_err());
    }
}
