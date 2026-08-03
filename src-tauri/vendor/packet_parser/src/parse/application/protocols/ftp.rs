// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Module for parsing FTP control-channel messages (RFC 959).

use std::convert::TryFrom;

use crate::{
    checks::application::ftp::{
        looks_like_reply, parse_payload_as_utf8, parse_reply_code, require_known_command,
        split_first_line, validate_command_syntax,
    },
    errors::application::ftp::FtpParseError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// A parsed FTP control-channel message: either a client command or a server
/// reply.
///
/// ```mermaid
/// ---
/// title: FTP control line
/// ---
/// packet-beta
/// 0-31: "Command verb or reply code"
/// 32-95: "Optional arguments or reply text"
/// 96-111: "CRLF"
/// ```
///
/// All borrowed fields are zero-copy views into the original packet payload.
#[derive(Debug, PartialEq, Eq)]
pub enum FtpMessage<'a> {
    /// A client command, e.g. `USER csanders`.
    Command {
        verb: &'a str,
        args: Option<&'a str>,
    },
    /// A server reply, e.g. `230 Logged in` or a multi-line `211-...`/`211 End.`
    /// reply. `lines` holds the text of every line after its code/separator,
    /// in order; a single-line reply has exactly one entry.
    Reply { code: u16, lines: Vec<&'a str> },
}

impl<'a> TryFrom<&'a [u8]> for FtpMessage<'a> {
    type Error = FtpParseError;

    fn try_from(payload: &'a [u8]) -> Result<Self, Self::Error> {
        parse_ftp_message(payload)
    }
}

/// Parses an FTP control-channel message from a given payload without
/// copying packet bytes.
pub fn parse_ftp_message(payload: &[u8]) -> Result<FtpMessage<'_>, FtpParseError> {
    if payload.is_empty() {
        return Err(FtpParseError::EmptyPayload);
    }

    let payload_str = parse_payload_as_utf8(payload)?;
    let (first_line, rest) = split_first_line(payload_str)?;

    if looks_like_reply(first_line) {
        let (code, separator, text) = parse_reply_code(first_line)?;
        return parse_reply(code, separator, text, rest);
    }

    let mut parts = first_line.splitn(2, ' ');
    let verb = require_known_command(parts.next().unwrap_or(""))?;
    let args = parts.next().map(str::trim).filter(|s| !s.is_empty());
    validate_command_syntax(verb, args, first_line)?;

    Ok(FtpMessage::Command { verb, args })
}

/// Builds a `Reply` from its already-parsed first line. A single space
/// separator means a one-line reply; a dash starts a multi-line reply, whose
/// continuation lines are read from `rest` until a terminator line
/// (`{code} ...`) is found.
fn parse_reply<'a>(
    code: u16,
    separator: u8,
    text: &'a str,
    rest: &'a str,
) -> Result<FtpMessage<'a>, FtpParseError> {
    if separator == b' ' {
        return Ok(FtpMessage::Reply {
            code,
            lines: vec![text],
        });
    }

    let mut lines = vec![text];
    let mut remainder = rest;
    while !remainder.is_empty() {
        let (line, next) = split_first_line(remainder)?;
        remainder = next;
        match parse_reply_code(line) {
            // Terminator: same code, space separator.
            Ok((line_code, b' ', line_text)) if line_code == code => {
                lines.push(line_text);
                return Ok(FtpMessage::Reply { code, lines });
            }
            // Continuation line repeating the code with a dash (common but
            // not mandatory per RFC 959): strip the prefix like the others.
            Ok((line_code, b'-', line_text)) if line_code == code => {
                lines.push(line_text);
            }
            // Free-form continuation line: kept as-is.
            _ => lines.push(line),
        }
    }

    Err(FtpParseError::MissingReplyTerminator)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trames reelles : pcaps_exemple/ftp.pcap (session FTP complete vers
    // 192.168.0.193:21).

    /// Trame 4 : bienvenue du serveur.
    const WELCOME_REPLY: &[u8] = b"220 Chris Sanders FTP Server\r\n";
    /// Trame 5 : commande USER.
    const USER_COMMAND: &[u8] = b"USER csanders\r\n";
    /// Trame 6 : mot de passe requis.
    const PASSWORD_REQUIRED_REPLY: &[u8] = b"331 Password required for csanders.\r\n";
    /// Trame 12 : reponse multi-lignes a FEAT.
    const FEAT_REPLY: &[u8] = b"211-Extensions supported:\r\n CLNT\r\n MDTM\r\n PASV\r\n REST STREAM\r\n SIZE\r\n211 End.\r\n";
    /// Trame 18 : reponse a PWD, chemin entre guillemets.
    const PWD_REPLY: &[u8] = b"257 \"/\" is current directory.\r\n";

    #[test]
    fn test_parse_welcome_reply() {
        let message = FtpMessage::try_from(WELCOME_REPLY).expect("valid reply");
        assert_eq!(
            message,
            FtpMessage::Reply {
                code: 220,
                lines: vec!["Chris Sanders FTP Server"],
            }
        );
    }

    #[test]
    fn test_parse_user_command() {
        let message = FtpMessage::try_from(USER_COMMAND).expect("valid command");
        assert_eq!(
            message,
            FtpMessage::Command {
                verb: "USER",
                args: Some("csanders"),
            }
        );
    }

    #[test]
    fn test_parse_password_required_reply() {
        let message = FtpMessage::try_from(PASSWORD_REQUIRED_REPLY).expect("valid reply");
        assert_eq!(
            message,
            FtpMessage::Reply {
                code: 331,
                lines: vec!["Password required for csanders."],
            }
        );
    }

    #[test]
    fn test_parse_multiline_feat_reply() {
        let message = FtpMessage::try_from(FEAT_REPLY).expect("valid multi-line reply");
        assert_eq!(
            message,
            FtpMessage::Reply {
                code: 211,
                lines: vec![
                    "Extensions supported:",
                    " CLNT",
                    " MDTM",
                    " PASV",
                    " REST STREAM",
                    " SIZE",
                    "End.",
                ],
            }
        );
    }

    #[test]
    fn test_parse_pwd_reply_with_quoted_path() {
        let message = FtpMessage::try_from(PWD_REPLY).expect("valid reply");
        assert_eq!(
            message,
            FtpMessage::Reply {
                code: 257,
                lines: vec!["\"/\" is current directory."],
            }
        );
    }

    #[test]
    fn test_parse_command_without_args() {
        let message = FtpMessage::try_from(&b"SYST\r\n"[..]).expect("valid command");
        assert_eq!(
            message,
            FtpMessage::Command {
                verb: "SYST",
                args: None,
            }
        );
    }

    #[test]
    fn test_commands_are_case_insensitive_and_zero_copy() {
        // Trame 1 (originale 329) de
        // pcaps_exemple/protocols/ftp/ftp_ipv6_lowercase.pcapng.
        let payload = b"opts utf8 on\r\n";
        let message = FtpMessage::try_from(&payload[..]).expect("valid lower-case command");
        assert_eq!(
            message,
            FtpMessage::Command {
                verb: "opts",
                args: Some("utf8 on"),
            }
        );
        let range = payload.as_ptr_range();
        match message {
            FtpMessage::Command { verb, args } => {
                assert!(range.contains(&verb.as_ptr()));
                assert!(range.contains(&args.expect("args").as_ptr()));
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_modern_extension_commands_are_recognized() {
        for payload in [
            &b"MLST\r\n"[..],
            &b"MLSD pub\r\n"[..],
            &b"AUTH TLS\r\n"[..],
            &b"PBSZ 0\r\n"[..],
            &b"PROT P\r\n"[..],
            &b"LPRT 4,4,127,0,0,1,2,8,73\r\n"[..],
        ] {
            assert!(
                FtpMessage::try_from(payload).is_ok(),
                "payload: {payload:?}"
            );
        }
    }

    #[test]
    fn test_parse_is_zero_copy() {
        let message = FtpMessage::try_from(USER_COMMAND).expect("valid command");
        let range = USER_COMMAND.as_ptr_range();
        match message {
            FtpMessage::Command { verb, args } => {
                assert!(range.contains(&verb.as_ptr()));
                assert!(range.contains(&args.expect("args").as_ptr()));
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_empty_payload_is_an_error() {
        assert_eq!(
            FtpMessage::try_from(&[][..]),
            Err(FtpParseError::EmptyPayload)
        );
    }

    #[test]
    fn test_unknown_command_is_rejected() {
        let payload = b"BONJOUR tout le monde\r\n";
        assert_eq!(
            FtpMessage::try_from(&payload[..]),
            Err(FtpParseError::UnknownCommand("BONJOUR".to_string()))
        );
    }

    #[test]
    fn test_multiline_reply_missing_terminator_is_an_error() {
        let payload = b"211-Extensions supported:\r\n CLNT\r\n";
        assert_eq!(
            FtpMessage::try_from(&payload[..]),
            Err(FtpParseError::MissingReplyTerminator)
        );
    }

    #[test]
    fn test_line_without_crlf_is_incomplete() {
        assert_eq!(
            FtpMessage::try_from(&b"USER alice"[..]),
            Err(FtpParseError::IncompleteLine)
        );
        assert_eq!(
            FtpMessage::try_from(&b"211-Features\r\n MLST"[..]),
            Err(FtpParseError::IncompleteLine)
        );
    }

    #[test]
    fn test_invalid_reply_range_and_command_arity_are_rejected() {
        assert_eq!(
            FtpMessage::try_from(&b"999 impossible\r\n"[..]),
            Err(FtpParseError::InvalidReplyCode(
                "999 impossible".to_string()
            ))
        );
        assert_eq!(
            FtpMessage::try_from(&b"NOOP unexpected\r\n"[..]),
            Err(FtpParseError::InvalidCommandSyntax(
                "NOOP unexpected".to_string()
            ))
        );
        assert_eq!(
            FtpMessage::try_from(&b"RETR\r\n"[..]),
            Err(FtpParseError::InvalidCommandSyntax("RETR".to_string()))
        );
    }

    #[test]
    fn test_non_utf8_payload_is_an_error() {
        let payload = [0xFFu8, 0xFE, 0xFD];
        assert_eq!(
            FtpMessage::try_from(&payload[..]),
            Err(FtpParseError::InvalidUtf8)
        );
    }
}
