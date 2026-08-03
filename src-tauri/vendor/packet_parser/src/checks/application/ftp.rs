// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::ftp::FtpParseError;

/// Standard FTP commands (RFC 959 and common extensions) accepted by the
/// parser. Restricting to known verbs keeps random text payloads from being
/// classified as FTP.
const FTP_COMMANDS: [&str; 46] = [
    "USER", "PASS", "ACCT", "CWD", "CDUP", "SMNT", "QUIT", "REIN", "PORT", "PASV", "TYPE", "STRU",
    "MODE", "RETR", "STOR", "STOU", "APPE", "ALLO", "REST", "RNFR", "RNTO", "ABOR", "DELE", "RMD",
    "MKD", "PWD", "LIST", "NLST", "SITE", "SYST", "STAT", "HELP", "NOOP", "FEAT", "CLNT", "OPTS",
    "EPSV", "EPRT", "SIZE", "MDTM", "MLST", "MLSD", "AUTH", "PBSZ", "PROT", "LPRT",
];

/// Checks that the FTP payload is not empty.
pub fn validate_ftp_payload_not_empty(payload: &[u8]) -> Result<(), FtpParseError> {
    if payload.is_empty() {
        return Err(FtpParseError::EmptyPayload);
    }
    Ok(())
}

/// Validates that the payload is UTF-8 and returns it as a borrowed `&str`.
pub fn parse_payload_as_utf8(payload: &[u8]) -> Result<&str, FtpParseError> {
    std::str::from_utf8(payload).map_err(|_| FtpParseError::InvalidUtf8)
}

/// Splits off the first CRLF-terminated line, returning the line (without the
/// CRLF) and the remainder of the payload.
pub fn split_first_line(payload: &str) -> Result<(&str, &str), FtpParseError> {
    match payload.find("\r\n") {
        Some(index) => Ok((&payload[..index], &payload[index + 2..])),
        None => Err(FtpParseError::IncompleteLine),
    }
}

/// Requires the command verb to be a known FTP command, returning it borrowed.
pub fn require_known_command(verb: &str) -> Result<&str, FtpParseError> {
    if FTP_COMMANDS
        .iter()
        .any(|command| command.eq_ignore_ascii_case(verb))
    {
        Ok(verb)
    } else {
        Err(FtpParseError::UnknownCommand(verb.to_string()))
    }
}

/// Performs command-specific minimum arity and syntax checks.
///
/// This deliberately leaves path names and extension parameters opaque: they
/// remain zero-copy and their interpretation belongs to the FTP command that
/// consumes them.
pub fn validate_command_syntax(
    verb: &str,
    args: Option<&str>,
    line: &str,
) -> Result<(), FtpParseError> {
    let has_args = args.is_some_and(|value| !value.is_empty());

    if [
        "CDUP", "QUIT", "REIN", "PASV", "ABOR", "PWD", "SYST", "NOOP", "FEAT",
    ]
    .iter()
    .any(|command| command.eq_ignore_ascii_case(verb))
        && has_args
    {
        return Err(invalid_command(line));
    }

    if [
        "USER", "PASS", "ACCT", "CWD", "SMNT", "PORT", "TYPE", "STRU", "MODE", "RETR", "STOR",
        "APPE", "ALLO", "REST", "RNFR", "RNTO", "DELE", "RMD", "MKD", "SITE", "CLNT", "OPTS",
        "EPRT", "SIZE", "MDTM", "AUTH", "PBSZ", "PROT", "LPRT",
    ]
    .iter()
    .any(|command| command.eq_ignore_ascii_case(verb))
        && !has_args
    {
        return Err(invalid_command(line));
    }

    let Some(args) = args else {
        return Ok(());
    };

    if verb.eq_ignore_ascii_case("PORT") && !valid_port(args) {
        return Err(invalid_command(line));
    }
    if verb.eq_ignore_ascii_case("EPRT") && !valid_eprt(args) {
        return Err(invalid_command(line));
    }
    if verb.eq_ignore_ascii_case("LPRT") && !valid_lprt(args) {
        return Err(invalid_command(line));
    }
    if verb.eq_ignore_ascii_case("PBSZ") && args.parse::<u64>().is_err() {
        return Err(invalid_command(line));
    }
    if verb.eq_ignore_ascii_case("PROT")
        && !["C", "S", "E", "P"]
            .iter()
            .any(|level| level.eq_ignore_ascii_case(args))
    {
        return Err(invalid_command(line));
    }

    Ok(())
}

fn invalid_command(line: &str) -> FtpParseError {
    FtpParseError::InvalidCommandSyntax(line.to_string())
}

fn valid_port(args: &str) -> bool {
    let mut count = 0usize;
    for part in args.split(',') {
        if part.is_empty() || part.parse::<u8>().is_err() {
            return false;
        }
        count += 1;
    }
    count == 6
}

fn valid_eprt(args: &str) -> bool {
    let bytes = args.as_bytes();
    let Some(&delimiter) = bytes.first() else {
        return false;
    };
    if !(33..=126).contains(&delimiter) {
        return false;
    }

    let delimiter = char::from(delimiter);
    let fields: Vec<_> = args[1..].split(delimiter).collect();
    fields.len() == 4
        && fields[3].is_empty()
        && fields[0].parse::<u8>().is_ok_and(|family| family != 0)
        && !fields[1].is_empty()
        && fields[2].parse::<u16>().is_ok_and(|port| port != 0)
}

fn valid_lprt(args: &str) -> bool {
    let Some(values) = args
        .split(',')
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
    else {
        return false;
    };
    if values.len() < 5 {
        return false;
    }

    let address_len = usize::from(values[1]);
    let port_len_index = 2 + address_len;
    let Some(&port_len) = values.get(port_len_index) else {
        return false;
    };

    values[0] != 0
        && address_len != 0
        && port_len != 0
        && values.len() == port_len_index + 1 + usize::from(port_len)
}

/// Returns whether a line starts like an FTP reply. This lets callers retain a
/// precise `InvalidReplyCode` error for malformed numeric status lines.
pub fn looks_like_reply(line: &str) -> bool {
    line.as_bytes().first().is_some_and(u8::is_ascii_digit)
}

/// Parses a 3-digit FTP reply code from the start of `line`, returning the
/// code, the separator that follows it (`' '` or `'-'`), and the rest of the
/// line after the separator.
pub fn parse_reply_code(line: &str) -> Result<(u16, u8, &str), FtpParseError> {
    let bytes = line.as_bytes();
    if bytes.len() < 4 || !bytes[..3].iter().all(u8::is_ascii_digit) {
        return Err(FtpParseError::InvalidReplyCode(line.to_string()));
    }
    let separator = bytes[3];
    if separator != b' ' && separator != b'-' {
        return Err(FtpParseError::InvalidReplyCode(line.to_string()));
    }
    // Digits are ASCII, so byte slicing at these offsets is safe on the `&str`.
    let code: u16 = line[..3].parse().expect("validated ASCII digits");
    if !(100..=599).contains(&code) {
        return Err(FtpParseError::InvalidReplyCode(line.to_string()));
    }
    Ok((code, separator, &line[4..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_payload_as_utf8_invalid() {
        assert_eq!(
            parse_payload_as_utf8(&[0xFF, 0xFE]),
            Err(FtpParseError::InvalidUtf8)
        );
    }

    #[test]
    fn test_split_first_line_with_separator() {
        assert_eq!(
            split_first_line("USER a\r\nPASS b"),
            Ok(("USER a", "PASS b"))
        );
    }

    #[test]
    fn test_split_first_line_without_separator() {
        assert_eq!(
            split_first_line("USER a"),
            Err(FtpParseError::IncompleteLine)
        );
    }

    #[test]
    fn test_require_known_command() {
        assert_eq!(require_known_command("USER"), Ok("USER"));
        assert_eq!(require_known_command("user"), Ok("user"));
        assert_eq!(require_known_command("MLSD"), Ok("MLSD"));
        assert_eq!(require_known_command("lprt"), Ok("lprt"));
        assert_eq!(
            require_known_command("BONJOUR"),
            Err(FtpParseError::UnknownCommand("BONJOUR".to_string()))
        );
    }

    #[test]
    fn test_parse_reply_code_single_line() {
        assert_eq!(
            parse_reply_code("230 Logged in"),
            Ok((230, b' ', "Logged in"))
        );
    }

    #[test]
    fn test_parse_reply_code_continuation() {
        assert_eq!(
            parse_reply_code("211-Extensions supported:"),
            Ok((211, b'-', "Extensions supported:"))
        );
    }

    #[test]
    fn test_parse_reply_code_invalid() {
        assert!(parse_reply_code("abc Logged in").is_err());
        assert!(parse_reply_code("23").is_err());
        assert!(parse_reply_code("230Logged in").is_err());
        assert!(parse_reply_code("999 impossible").is_err());
    }

    #[test]
    fn test_validate_command_syntax() {
        assert_eq!(validate_command_syntax("NOOP", None, "NOOP"), Ok(()));
        assert!(validate_command_syntax("NOOP", Some("extra"), "NOOP extra").is_err());
        assert!(validate_command_syntax("USER", None, "USER").is_err());
        assert_eq!(
            validate_command_syntax("PORT", Some("127,0,0,1,8,1"), "PORT 127,0,0,1,8,1"),
            Ok(())
        );
        assert!(validate_command_syntax("PORT", Some("127,0,0,999,8,1"), "bad").is_err());
        assert_eq!(
            validate_command_syntax("EPRT", Some("|2|::1|2121|"), "EPRT |2|::1|2121|"),
            Ok(())
        );
        assert_eq!(
            validate_command_syntax(
                "LPRT",
                Some("6,16,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,8,73"),
                "LPRT valid"
            ),
            Ok(())
        );
        assert_eq!(validate_command_syntax("PBSZ", Some("0"), "PBSZ 0"), Ok(()));
        assert!(validate_command_syntax("PBSZ", Some("many"), "PBSZ many").is_err());
        assert_eq!(validate_command_syntax("PROT", Some("p"), "PROT p"), Ok(()));
        assert!(validate_command_syntax("PROT", Some("X"), "PROT X").is_err());
    }
}
