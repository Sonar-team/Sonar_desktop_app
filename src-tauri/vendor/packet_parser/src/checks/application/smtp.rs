// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::smtp::SmtpParseError;

/// Standard SMTP commands (RFC 5321 and common extensions) accepted by the
/// parser. Restricting to known verbs keeps random text payloads from being
/// classified as SMTP.
const SMTP_COMMANDS: [&str; 13] = [
    "HELO", "EHLO", "MAIL", "RCPT", "DATA", "RSET", "VRFY", "EXPN", "HELP", "NOOP", "QUIT", "AUTH",
    "STARTTLS",
];

/// Validates that the payload is UTF-8 and returns it as a borrowed `&str`.
pub fn parse_payload_as_utf8(payload: &[u8]) -> Result<&str, SmtpParseError> {
    std::str::from_utf8(payload).map_err(|_| SmtpParseError::InvalidUtf8)
}

/// Splits off the first CRLF-terminated line, returning the line (without the
/// CRLF) and the remainder of the payload.
pub fn split_first_line(payload: &str) -> Result<(&str, &str), SmtpParseError> {
    match payload.find("\r\n") {
        Some(index) => Ok((&payload[..index], &payload[index + 2..])),
        None => Err(SmtpParseError::IncompleteLine),
    }
}

/// Requires the command verb to be a known SMTP command, returning it borrowed.
pub fn require_known_command(verb: &str) -> Result<&str, SmtpParseError> {
    if SMTP_COMMANDS
        .iter()
        .any(|command| command.eq_ignore_ascii_case(verb))
    {
        Ok(verb)
    } else {
        Err(SmtpParseError::UnknownCommand(verb.to_string()))
    }
}

/// Performs the minimum command-specific syntax checks needed to distinguish
/// complete SMTP commands from truncated or malformed lines.
pub fn validate_command_syntax(
    verb: &str,
    args: Option<&str>,
    line: &str,
) -> Result<(), SmtpParseError> {
    let invalid = || SmtpParseError::InvalidCommandSyntax(line.to_string());

    if ["DATA", "RSET", "QUIT", "STARTTLS"]
        .iter()
        .any(|command| command.eq_ignore_ascii_case(verb))
    {
        return if args.is_none() {
            Ok(())
        } else {
            Err(invalid())
        };
    }

    if verb.eq_ignore_ascii_case("HELO") || verb.eq_ignore_ascii_case("EHLO") {
        return if args.is_some_and(single_atom) {
            Ok(())
        } else {
            Err(invalid())
        };
    }

    if verb.eq_ignore_ascii_case("MAIL") {
        return if args.is_some_and(|value| valid_path_command(value, "FROM:")) {
            Ok(())
        } else {
            Err(invalid())
        };
    }

    if verb.eq_ignore_ascii_case("RCPT") {
        return if args.is_some_and(|value| valid_path_command(value, "TO:")) {
            Ok(())
        } else {
            Err(invalid())
        };
    }

    if ["VRFY", "EXPN", "AUTH"]
        .iter()
        .any(|command| command.eq_ignore_ascii_case(verb))
        && args.is_none()
    {
        return Err(invalid());
    }

    Ok(())
}

fn single_atom(value: &str) -> bool {
    value.split_ascii_whitespace().count() == 1
}

fn valid_path_command(value: &str, prefix: &str) -> bool {
    let Some(prefix_bytes) = value.as_bytes().get(..prefix.len()) else {
        return false;
    };
    if !prefix_bytes.eq_ignore_ascii_case(prefix.as_bytes()) {
        return false;
    }

    // The matched prefix is ASCII, therefore this is a UTF-8 character
    // boundary even when SMTPUTF8 is in use in the path itself.
    let path_and_parameters = value[prefix.len()..].trim_start_matches(' ');
    let Some(path_without_opening_bracket) = path_and_parameters.strip_prefix('<') else {
        return false;
    };
    let Some(closing_bracket) = path_without_opening_bracket.find('>') else {
        return false;
    };
    let parameters = &path_without_opening_bracket[closing_bracket + 1..];
    parameters.is_empty() || parameters.starts_with(' ')
}

/// Returns whether a line starts like an SMTP reply. This lets callers retain
/// a precise `InvalidReplyCode` error for malformed numeric status lines.
pub fn looks_like_reply(line: &str) -> bool {
    line.as_bytes().first().is_some_and(u8::is_ascii_digit)
}

/// Parses a 3-digit SMTP reply code from the start of `line`, returning the
/// code, its optional separator (`' '` or `'-'`), and the text after it. A
/// bare reply code is valid and has no separator and empty text.
pub fn parse_reply_code(line: &str) -> Result<(u16, Option<u8>, &str), SmtpParseError> {
    let bytes = line.as_bytes();
    if bytes.len() < 3 || !bytes[..3].iter().all(u8::is_ascii_digit) {
        return Err(SmtpParseError::InvalidReplyCode(line.to_string()));
    }

    let code: u16 = line[..3].parse().expect("validated ASCII digits");
    if !(200..=599).contains(&code) {
        return Err(SmtpParseError::InvalidReplyCode(line.to_string()));
    }

    match bytes.get(3).copied() {
        None => Ok((code, None, "")),
        Some(separator @ (b' ' | b'-')) => Ok((code, Some(separator), &line[4..])),
        Some(_) => Err(SmtpParseError::InvalidReplyCode(line.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_payload_as_utf8_invalid() {
        assert_eq!(
            parse_payload_as_utf8(&[0xFF, 0xFE]),
            Err(SmtpParseError::InvalidUtf8)
        );
    }

    #[test]
    fn test_split_first_line_with_separator() {
        assert_eq!(
            split_first_line("EHLO gp\r\nMAIL FROM:<a>"),
            Ok(("EHLO gp", "MAIL FROM:<a>"))
        );
    }

    #[test]
    fn test_split_first_line_without_separator() {
        assert_eq!(
            split_first_line("EHLO gp"),
            Err(SmtpParseError::IncompleteLine)
        );
    }

    #[test]
    fn test_require_known_command() {
        assert_eq!(require_known_command("EHLO"), Ok("EHLO"));
        assert_eq!(require_known_command("ehlo"), Ok("ehlo"));
        assert_eq!(
            require_known_command("BONJOUR"),
            Err(SmtpParseError::UnknownCommand("BONJOUR".to_string()))
        );
    }

    #[test]
    fn test_parse_reply_code_single_line() {
        assert_eq!(parse_reply_code("250 OK"), Ok((250, Some(b' '), "OK")));
        assert_eq!(parse_reply_code("250"), Ok((250, None, "")));
    }

    #[test]
    fn test_parse_reply_code_continuation() {
        assert_eq!(
            parse_reply_code("250-SIZE 52428800"),
            Ok((250, Some(b'-'), "SIZE 52428800"))
        );
    }

    #[test]
    fn test_parse_reply_code_invalid() {
        assert!(parse_reply_code("abc OK").is_err());
        assert!(parse_reply_code("25").is_err());
        assert!(parse_reply_code("250OK").is_err());
        assert!(parse_reply_code("199 preliminary").is_err());
        assert!(parse_reply_code("999 impossible").is_err());
    }

    #[test]
    fn test_validate_command_syntax() {
        assert_eq!(validate_command_syntax("DATA", None, "DATA"), Ok(()));
        assert!(validate_command_syntax("DATA", Some("extra"), "DATA extra").is_err());
        assert_eq!(
            validate_command_syntax("ehlo", Some("example.test"), "ehlo example.test"),
            Ok(())
        );
        assert!(validate_command_syntax("EHLO", None, "EHLO").is_err());
        assert!(validate_command_syntax("EHLO", Some("a b"), "EHLO a b").is_err());
        assert_eq!(
            validate_command_syntax("MAIL", Some("from:<> SIZE=0"), "MAIL from:<> SIZE=0"),
            Ok(())
        );
        assert_eq!(
            validate_command_syntax(
                "RCPT",
                Some("TO: <user@example.test>"),
                "RCPT TO: <user@example.test>"
            ),
            Ok(())
        );
        assert!(validate_command_syntax("MAIL", Some("user@example.test"), "bad").is_err());
        assert!(validate_command_syntax("AUTH", None, "AUTH").is_err());
    }

    #[test]
    fn test_bdat_is_not_claimed_without_binary_body_model() {
        assert_eq!(
            require_known_command("BDAT"),
            Err(SmtpParseError::UnknownCommand("BDAT".to_string()))
        );
    }
}
