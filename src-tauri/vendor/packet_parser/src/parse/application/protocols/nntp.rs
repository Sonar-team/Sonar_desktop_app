// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Module for parsing NNTP messages (RFC 3977).

use std::convert::TryFrom;

use crate::{
    checks::application::nntp::{
        looks_like_response, parse_payload_as_utf8, parse_response_code, require_known_command,
        split_command, split_first_line, validate_command_syntax, validate_non_empty,
    },
    errors::application::nntp::NntpParseError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// A parsed NNTP message: either a client command or a server response.
///
/// ```mermaid
/// ---
/// title: NNTP command or response line
/// ---
/// packet-beta
/// 0-31: "Command verb or response code"
/// 32-95: "Optional arguments or response text"
/// 96-111: "CRLF"
/// ```
///
/// All borrowed fields are zero-copy views into the original packet payload.
///
/// Unlike FTP/SMTP, an NNTP response is always a single status line (RFC
/// 3977 §3.1): any following data block (e.g. an article body) is a separate
/// concern, terminated by a lone `.` line, and is not modeled here.
#[derive(Debug, PartialEq, Eq)]
pub enum NntpMessage<'a> {
    /// A client command, e.g. `MODE READER`.
    Command {
        verb: &'a str,
        args: Option<&'a str>,
    },
    /// A server response, e.g. `224 data follows`.
    Response { code: u16, text: &'a str },
}

impl<'a> TryFrom<&'a [u8]> for NntpMessage<'a> {
    type Error = NntpParseError;

    fn try_from(payload: &'a [u8]) -> Result<Self, Self::Error> {
        parse_nntp_message(payload)
    }
}

/// Parses an NNTP message from a given payload without copying packet bytes.
pub fn parse_nntp_message(payload: &[u8]) -> Result<NntpMessage<'_>, NntpParseError> {
    validate_non_empty(payload)?;

    let (first_line_bytes, _rest) = split_first_line(payload)?;
    let first_line = parse_payload_as_utf8(first_line_bytes)?;

    // Any numeric-looking first line is a response. This preserves a precise
    // response-code error for malformed codes instead of misclassifying them
    // as unknown commands.
    if looks_like_response(first_line) {
        let (code, text) = parse_response_code(first_line)?;
        return Ok(NntpMessage::Response { code, text });
    }

    let (verb, args) = split_command(first_line);
    let verb = require_known_command(verb)?;
    validate_command_syntax(verb, args, first_line)?;

    Ok(NntpMessage::Command { verb, args })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trames reelles : pcaps_exemple/protocols/nntp/nntp_control.pcapng.
    // Les numeros originaux du sample Wireshark sont documentes dans SOURCE.md.

    /// Trame 1 (originale 16) : commande MODE READER.
    const MODE_READER_COMMAND: &[u8] = b"MODE READER\r\n";
    /// Trame 2 (originale 19) : commande XOVER sans argument.
    const XOVER_COMMAND: &[u8] = b"XOVER\r\n";
    /// Trame 3 (originale 26) : commande LIST NEWSGROUPS.
    const LIST_NEWSGROUPS_COMMAND: &[u8] = b"LIST NEWSGROUPS\r\n";
    /// Trame 4 (originale 37) : commande XOVER avec une plage d'articles.
    const XOVER_RANGE_COMMAND: &[u8] = b"XOVER 2241-3260\r\n";
    /// Trame 5 (originale 38) : reponse a une commande XOVER/LIST.
    const DATA_FOLLOWS_RESPONSE: &[u8] = b"224 data follows\r\n";
    /// Trame 6 (originale 116) : commande ARTICLE avec un numero.
    const ARTICLE_COMMAND: &[u8] = b"ARTICLE 2482\r\n";

    #[test]
    fn test_parse_mode_reader_command() {
        let message = NntpMessage::try_from(MODE_READER_COMMAND).expect("valid command");
        assert_eq!(
            message,
            NntpMessage::Command {
                verb: "MODE",
                args: Some("READER"),
            }
        );
    }

    #[test]
    fn test_parse_xover_command_without_args() {
        let message = NntpMessage::try_from(XOVER_COMMAND).expect("valid command");
        assert_eq!(
            message,
            NntpMessage::Command {
                verb: "XOVER",
                args: None,
            }
        );
    }

    #[test]
    fn test_parse_list_newsgroups_command() {
        let message = NntpMessage::try_from(LIST_NEWSGROUPS_COMMAND).expect("valid command");
        assert_eq!(
            message,
            NntpMessage::Command {
                verb: "LIST",
                args: Some("NEWSGROUPS"),
            }
        );
    }

    #[test]
    fn test_parse_xover_range_command() {
        let message = NntpMessage::try_from(XOVER_RANGE_COMMAND).expect("valid command");
        assert_eq!(
            message,
            NntpMessage::Command {
                verb: "XOVER",
                args: Some("2241-3260"),
            }
        );
    }

    #[test]
    fn test_parse_data_follows_response() {
        let message = NntpMessage::try_from(DATA_FOLLOWS_RESPONSE).expect("valid response");
        assert_eq!(
            message,
            NntpMessage::Response {
                code: 224,
                text: "data follows",
            }
        );
    }

    #[test]
    fn test_parse_article_command() {
        let message = NntpMessage::try_from(ARTICLE_COMMAND).expect("valid command");
        assert_eq!(
            message,
            NntpMessage::Command {
                verb: "ARTICLE",
                args: Some("2482"),
            }
        );
    }

    #[test]
    fn test_command_verbs_are_case_insensitive_and_zero_copy() {
        let payload = b"mode reader\r\n";
        let message = NntpMessage::try_from(&payload[..]).expect("valid lower-case command");
        assert_eq!(
            message,
            NntpMessage::Command {
                verb: "mode",
                args: Some("reader"),
            }
        );
        let range = payload.as_ptr_range();
        match message {
            NntpMessage::Command { verb, args } => {
                assert!(range.contains(&verb.as_ptr()));
                assert!(range.contains(&args.expect("args").as_ptr()));
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_capabilities_and_hdr_commands_are_recognized() {
        assert_eq!(
            NntpMessage::try_from(&b"CAPABILITIES\r\n"[..]),
            Ok(NntpMessage::Command {
                verb: "CAPABILITIES",
                args: None,
            })
        );
        assert_eq!(
            NntpMessage::try_from(&b"hdr Subject 12-20\r\n"[..]),
            Ok(NntpMessage::Command {
                verb: "hdr",
                args: Some("Subject 12-20"),
            })
        );
    }

    #[test]
    fn test_tabs_are_valid_command_separators() {
        assert_eq!(
            NntpMessage::try_from(&b"mode\treader\r\n"[..]),
            Ok(NntpMessage::Command {
                verb: "mode",
                args: Some("reader"),
            })
        );
    }

    #[test]
    fn test_bare_response_code_is_accepted() {
        assert_eq!(
            NntpMessage::try_from(&b"224\r\n"[..]),
            Ok(NntpMessage::Response {
                code: 224,
                text: "",
            })
        );
    }

    #[test]
    fn test_non_utf8_data_block_is_not_decoded() {
        let payload = b"224 data follows\r\n\xff\xfe\r\n.\r\n";
        assert_eq!(
            NntpMessage::try_from(&payload[..]),
            Ok(NntpMessage::Response {
                code: 224,
                text: "data follows",
            })
        );
    }

    #[test]
    fn test_parse_is_zero_copy() {
        let message = NntpMessage::try_from(MODE_READER_COMMAND).expect("valid command");
        let range = MODE_READER_COMMAND.as_ptr_range();
        match message {
            NntpMessage::Command { verb, args } => {
                assert!(range.contains(&verb.as_ptr()));
                assert!(range.contains(&args.expect("args").as_ptr()));
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_empty_payload_is_an_error() {
        assert_eq!(
            NntpMessage::try_from(&[][..]),
            Err(NntpParseError::EmptyPayload)
        );
    }

    #[test]
    fn test_unknown_command_is_rejected() {
        let payload = b"BONJOUR tout le monde\r\n";
        assert_eq!(
            NntpMessage::try_from(&payload[..]),
            Err(NntpParseError::UnknownCommand("BONJOUR".to_string()))
        );
    }

    #[test]
    fn test_line_without_crlf_is_incomplete() {
        assert_eq!(
            NntpMessage::try_from(&b"MODE READER"[..]),
            Err(NntpParseError::IncompleteLine)
        );
    }

    #[test]
    fn test_invalid_response_range_and_command_arity_are_rejected() {
        assert_eq!(
            NntpMessage::try_from(&b"999 impossible\r\n"[..]),
            Err(NntpParseError::InvalidResponseCode(
                "999 impossible".to_string()
            ))
        );
        assert_eq!(
            NntpMessage::try_from(&b"224-data follows\r\n"[..]),
            Err(NntpParseError::InvalidResponseCode(
                "224-data follows".to_string()
            ))
        );
        assert_eq!(
            NntpMessage::try_from(&b"CAPABILITIES extra\r\n"[..]),
            Err(NntpParseError::InvalidCommandSyntax(
                "CAPABILITIES extra".to_string()
            ))
        );
        assert_eq!(
            NntpMessage::try_from(&b"HDR\r\n"[..]),
            Err(NntpParseError::InvalidCommandSyntax("HDR".to_string()))
        );
    }

    #[test]
    fn test_non_utf8_payload_is_an_error() {
        let payload = [0xFFu8, 0xFE, 0xFD, b'\r', b'\n'];
        assert_eq!(
            NntpMessage::try_from(&payload[..]),
            Err(NntpParseError::InvalidUtf8)
        );
    }
}
