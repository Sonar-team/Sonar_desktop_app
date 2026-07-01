// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::convert::TryFrom;
use std::str;

use crate::{
    checks::application::postgresql::{
        POSTGRESQL_TYPED_HEADER_LEN, POSTGRESQL_UNTYPED_HEADER_LEN, validate_no_trailing_bytes,
        validate_packet_not_empty, validate_remaining, validate_typed_header_available,
        validate_typed_message_available, validate_untyped_header_available,
        validate_untyped_message_available,
    },
    errors::application::postgresql::PostgreSqlError,
};

const POSTGRESQL_PROTOCOL_VERSION_3: u32 = 196_608;
const POSTGRESQL_SSL_REQUEST_CODE: u32 = 80_877_103;
const POSTGRESQL_CANCEL_REQUEST_CODE: u32 = 80_877_102;
const POSTGRESQL_GSSENC_REQUEST_CODE: u32 = 80_877_104;

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// PostgreSQL frontend/backend protocol packet
///
/// ```mermaid
/// ---
/// title: PostgreSqlPacket
/// ---
/// packet-beta
/// 0-7: "Message Type u8"
/// 8-39: "Length u32"
/// 40-103: "Payload variable"
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct PostgreSqlPacket<'a> {
    pub messages: Vec<PostgreSqlMessage<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PostgreSqlMessage<'a> {
    pub message_type: PostgreSqlMessageType,
    /// PostgreSQL length field. For typed messages this excludes the type byte.
    pub length: u32,
    pub payload: &'a [u8],
    pub body: PostgreSqlMessageBody<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PostgreSqlMessageType {
    Authentication,
    BackendKeyData,
    Bind,
    BindComplete,
    CancelRequest,
    CloseOrCommandComplete,
    CloseComplete,
    CopyBothResponse,
    CopyData,
    CopyDone,
    CopyFail,
    CopyInResponse,
    DataRowOrDescribe,
    EmptyQueryResponse,
    ErrorResponseOrExecute,
    FlushOrCopyOutResponse,
    FunctionCall,
    FunctionCallResponse,
    GssEncRequest,
    NoData,
    NoticeResponse,
    NotificationResponse,
    ParameterDescription,
    ParameterStatusOrSync,
    Parse,
    ParseComplete,
    PortalSuspended,
    Query,
    ReadyForQuery,
    RowDescription,
    SslRequest,
    StartupMessage,
    Terminate,
}

impl PostgreSqlMessageType {
    fn name(self) -> &'static str {
        match self {
            PostgreSqlMessageType::Authentication => "Authentication",
            PostgreSqlMessageType::BackendKeyData => "BackendKeyData",
            PostgreSqlMessageType::Bind => "Bind",
            PostgreSqlMessageType::BindComplete => "BindComplete",
            PostgreSqlMessageType::CancelRequest => "CancelRequest",
            PostgreSqlMessageType::CloseOrCommandComplete => "CloseOrCommandComplete",
            PostgreSqlMessageType::CloseComplete => "CloseComplete",
            PostgreSqlMessageType::CopyBothResponse => "CopyBothResponse",
            PostgreSqlMessageType::CopyData => "CopyData",
            PostgreSqlMessageType::CopyDone => "CopyDone",
            PostgreSqlMessageType::CopyFail => "CopyFail",
            PostgreSqlMessageType::CopyInResponse => "CopyInResponse",
            PostgreSqlMessageType::DataRowOrDescribe => "DataRowOrDescribe",
            PostgreSqlMessageType::EmptyQueryResponse => "EmptyQueryResponse",
            PostgreSqlMessageType::ErrorResponseOrExecute => "ErrorResponseOrExecute",
            PostgreSqlMessageType::FlushOrCopyOutResponse => "FlushOrCopyOutResponse",
            PostgreSqlMessageType::FunctionCall => "FunctionCall",
            PostgreSqlMessageType::FunctionCallResponse => "FunctionCallResponse",
            PostgreSqlMessageType::GssEncRequest => "GssEncRequest",
            PostgreSqlMessageType::NoData => "NoData",
            PostgreSqlMessageType::NoticeResponse => "NoticeResponse",
            PostgreSqlMessageType::NotificationResponse => "NotificationResponse",
            PostgreSqlMessageType::ParameterDescription => "ParameterDescription",
            PostgreSqlMessageType::ParameterStatusOrSync => "ParameterStatusOrSync",
            PostgreSqlMessageType::Parse => "Parse",
            PostgreSqlMessageType::ParseComplete => "ParseComplete",
            PostgreSqlMessageType::PortalSuspended => "PortalSuspended",
            PostgreSqlMessageType::Query => "Query",
            PostgreSqlMessageType::ReadyForQuery => "ReadyForQuery",
            PostgreSqlMessageType::RowDescription => "RowDescription",
            PostgreSqlMessageType::SslRequest => "SslRequest",
            PostgreSqlMessageType::StartupMessage => "StartupMessage",
            PostgreSqlMessageType::Terminate => "Terminate",
        }
    }
}

impl TryFrom<u8> for PostgreSqlMessageType {
    type Error = PostgreSqlError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'R' => PostgreSqlMessageType::Authentication,
            b'K' => PostgreSqlMessageType::BackendKeyData,
            b'B' => PostgreSqlMessageType::Bind,
            b'2' => PostgreSqlMessageType::BindComplete,
            b'C' => PostgreSqlMessageType::CloseOrCommandComplete,
            b'3' => PostgreSqlMessageType::CloseComplete,
            b'W' => PostgreSqlMessageType::CopyBothResponse,
            b'd' => PostgreSqlMessageType::CopyData,
            b'c' => PostgreSqlMessageType::CopyDone,
            b'f' => PostgreSqlMessageType::CopyFail,
            b'G' => PostgreSqlMessageType::CopyInResponse,
            b'D' => PostgreSqlMessageType::DataRowOrDescribe,
            b'I' => PostgreSqlMessageType::EmptyQueryResponse,
            b'E' => PostgreSqlMessageType::ErrorResponseOrExecute,
            b'H' => PostgreSqlMessageType::FlushOrCopyOutResponse,
            b'F' => PostgreSqlMessageType::FunctionCall,
            b'V' => PostgreSqlMessageType::FunctionCallResponse,
            b'n' => PostgreSqlMessageType::NoData,
            b'N' => PostgreSqlMessageType::NoticeResponse,
            b'A' => PostgreSqlMessageType::NotificationResponse,
            b't' => PostgreSqlMessageType::ParameterDescription,
            b'S' => PostgreSqlMessageType::ParameterStatusOrSync,
            b'P' => PostgreSqlMessageType::Parse,
            b'1' => PostgreSqlMessageType::ParseComplete,
            b's' => PostgreSqlMessageType::PortalSuspended,
            b'Q' => PostgreSqlMessageType::Query,
            b'Z' => PostgreSqlMessageType::ReadyForQuery,
            b'T' => PostgreSqlMessageType::RowDescription,
            b'X' => PostgreSqlMessageType::Terminate,
            _ => return Err(PostgreSqlError::InvalidMessageType(value)),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PostgreSqlMessageBody<'a> {
    Parse(PostgreSqlParse<'a>),
    Bind(PostgreSqlBind<'a>),
    Execute(PostgreSqlExecute<'a>),
    Query { query: &'a str },
    Startup(PostgreSqlStartup<'a>),
    CancelRequest { process_id: u32, secret_key: u32 },
    Empty,
    Raw(&'a [u8]),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PostgreSqlParse<'a> {
    pub statement: &'a str,
    pub query: &'a str,
    pub parameter_type_oids: Vec<u32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PostgreSqlBind<'a> {
    pub portal: &'a str,
    pub statement: &'a str,
    pub parameter_formats: Vec<u16>,
    pub parameter_values: Vec<Option<&'a [u8]>>,
    pub result_formats: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PostgreSqlExecute<'a> {
    pub portal: &'a str,
    pub max_rows: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PostgreSqlStartup<'a> {
    pub protocol_version: u32,
    pub parameters: Vec<(&'a str, &'a str)>,
}

impl<'a> TryFrom<&'a [u8]> for PostgreSqlPacket<'a> {
    type Error = PostgreSqlError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        validate_packet_not_empty(value)?;

        match parse_typed_messages(value) {
            Ok(packet) => Ok(packet),
            Err(typed_error) => parse_untyped_message(value).or(Err(typed_error)),
        }
    }
}

fn parse_typed_messages(payload: &[u8]) -> Result<PostgreSqlPacket<'_>, PostgreSqlError> {
    let mut messages = Vec::new();
    let mut offset = 0usize;

    while offset < payload.len() {
        let remaining = &payload[offset..];
        validate_typed_header_available(remaining)?;

        let message_type = PostgreSqlMessageType::try_from(remaining[0])?;
        let length = u32::from_be_bytes([remaining[1], remaining[2], remaining[3], remaining[4]]);
        let consumed = validate_typed_message_available(remaining, length)?;

        let body = &remaining[POSTGRESQL_TYPED_HEADER_LEN..consumed];
        let parsed_body = parse_typed_body(message_type, body)?;

        messages.push(PostgreSqlMessage {
            message_type,
            length,
            payload: body,
            body: parsed_body,
        });

        offset += consumed;
    }

    Ok(PostgreSqlPacket { messages })
}

fn parse_untyped_message(payload: &[u8]) -> Result<PostgreSqlPacket<'_>, PostgreSqlError> {
    validate_untyped_header_available(payload)?;

    let length = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let consumed = validate_untyped_message_available(payload, length)?;
    let code = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let body = &payload[POSTGRESQL_UNTYPED_HEADER_LEN..consumed];

    let (message_type, parsed_body) = match code {
        POSTGRESQL_PROTOCOL_VERSION_3 => (
            PostgreSqlMessageType::StartupMessage,
            PostgreSqlMessageBody::Startup(parse_startup(code, body)?),
        ),
        POSTGRESQL_SSL_REQUEST_CODE => (
            PostgreSqlMessageType::SslRequest,
            PostgreSqlMessageBody::Empty,
        ),
        POSTGRESQL_CANCEL_REQUEST_CODE => {
            let mut cur = Cursor::new(body);
            let process_id = cur.read_u32("process_id")?;
            let secret_key = cur.read_u32("secret_key")?;
            validate_no_trailing_bytes(cur.remaining(), "CancelRequest")?;
            (
                PostgreSqlMessageType::CancelRequest,
                PostgreSqlMessageBody::CancelRequest {
                    process_id,
                    secret_key,
                },
            )
        }
        POSTGRESQL_GSSENC_REQUEST_CODE => (
            PostgreSqlMessageType::GssEncRequest,
            PostgreSqlMessageBody::Empty,
        ),
        _ => return Err(PostgreSqlError::UnsupportedStartupCode(code)),
    };

    Ok(PostgreSqlPacket {
        messages: vec![PostgreSqlMessage {
            message_type,
            length,
            payload: body,
            body: parsed_body,
        }],
    })
}

fn parse_typed_body<'a>(
    message_type: PostgreSqlMessageType,
    body: &'a [u8],
) -> Result<PostgreSqlMessageBody<'a>, PostgreSqlError> {
    match message_type {
        PostgreSqlMessageType::Parse => parse_parse(body).map(PostgreSqlMessageBody::Parse),
        PostgreSqlMessageType::Bind => parse_bind(body).map(PostgreSqlMessageBody::Bind),
        PostgreSqlMessageType::ErrorResponseOrExecute => match parse_execute(body) {
            Ok(execute) => Ok(PostgreSqlMessageBody::Execute(execute)),
            Err(_) => Ok(PostgreSqlMessageBody::Raw(body)),
        },
        PostgreSqlMessageType::EmptyQueryResponse => {
            validate_no_trailing_bytes(body.len(), message_type.name())?;
            Ok(PostgreSqlMessageBody::Empty)
        }
        PostgreSqlMessageType::ParameterStatusOrSync
        | PostgreSqlMessageType::Terminate
        | PostgreSqlMessageType::FlushOrCopyOutResponse => {
            if body.is_empty() {
                Ok(PostgreSqlMessageBody::Empty)
            } else {
                Ok(PostgreSqlMessageBody::Raw(body))
            }
        }
        PostgreSqlMessageType::Query => parse_query(body),
        _ => Ok(PostgreSqlMessageBody::Raw(body)),
    }
}

fn parse_parse(body: &[u8]) -> Result<PostgreSqlParse<'_>, PostgreSqlError> {
    let mut cur = Cursor::new(body);
    let statement = cur.read_cstring("statement")?;
    let query = cur.read_cstring("query")?;
    let parameter_count = cur.read_u16("parameter_count")? as usize;

    validate_remaining(cur.remaining(), parameter_count * 4, "parameter_type_oids")?;

    let mut parameter_type_oids = Vec::with_capacity(parameter_count);
    for _ in 0..parameter_count {
        parameter_type_oids.push(cur.read_u32("parameter_type_oid")?);
    }

    validate_no_trailing_bytes(cur.remaining(), "Parse")?;

    Ok(PostgreSqlParse {
        statement,
        query,
        parameter_type_oids,
    })
}

fn parse_bind(body: &[u8]) -> Result<PostgreSqlBind<'_>, PostgreSqlError> {
    let mut cur = Cursor::new(body);
    let portal = cur.read_cstring("portal")?;
    let statement = cur.read_cstring("statement")?;

    let format_count = cur.read_u16("parameter_format_count")? as usize;
    validate_remaining(cur.remaining(), format_count * 2, "parameter_formats")?;

    let mut parameter_formats = Vec::with_capacity(format_count);
    for _ in 0..format_count {
        parameter_formats.push(cur.read_u16("parameter_format")?);
    }

    let value_count = cur.read_u16("parameter_value_count")? as usize;
    let mut parameter_values = Vec::with_capacity(value_count);
    for _ in 0..value_count {
        let len = cur.read_i32("parameter_value_length")?;
        if len == -1 {
            parameter_values.push(None);
            continue;
        }

        if len < 0 {
            return Err(PostgreSqlError::InvalidFieldLength {
                field: "parameter_value_length",
                got: len,
            });
        }

        let value = cur.read_bytes(len as usize, "parameter_value")?;
        parameter_values.push(Some(value));
    }

    let result_format_count = cur.read_u16("result_format_count")? as usize;
    validate_remaining(cur.remaining(), result_format_count * 2, "result_formats")?;

    let mut result_formats = Vec::with_capacity(result_format_count);
    for _ in 0..result_format_count {
        result_formats.push(cur.read_u16("result_format")?);
    }

    validate_no_trailing_bytes(cur.remaining(), "Bind")?;

    Ok(PostgreSqlBind {
        portal,
        statement,
        parameter_formats,
        parameter_values,
        result_formats,
    })
}

fn parse_execute(body: &[u8]) -> Result<PostgreSqlExecute<'_>, PostgreSqlError> {
    let mut cur = Cursor::new(body);
    let portal = cur.read_cstring("portal")?;
    let max_rows = cur.read_u32("max_rows")?;

    validate_no_trailing_bytes(cur.remaining(), "Execute")?;

    Ok(PostgreSqlExecute { portal, max_rows })
}

fn parse_query(body: &[u8]) -> Result<PostgreSqlMessageBody<'_>, PostgreSqlError> {
    let mut cur = Cursor::new(body);
    let query = cur.read_cstring("query")?;

    validate_no_trailing_bytes(cur.remaining(), "Query")?;

    Ok(PostgreSqlMessageBody::Query { query })
}

fn parse_startup<'a>(
    protocol_version: u32,
    body: &'a [u8],
) -> Result<PostgreSqlStartup<'a>, PostgreSqlError> {
    let mut cur = Cursor::new(body);
    let mut parameters = Vec::new();

    loop {
        if cur.remaining() == 0 {
            break;
        }

        if cur.peek() == Some(0) {
            cur.skip(1)?;
            validate_no_trailing_bytes(cur.remaining(), "StartupMessage")?;
            break;
        }

        let key = cur.read_cstring("startup_parameter_key")?;
        let value = cur.read_cstring("startup_parameter_value")?;
        parameters.push((key, value));
    }

    Ok(PostgreSqlStartup {
        protocol_version,
        parameters,
    })
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip(&mut self, count: usize) -> Result<(), PostgreSqlError> {
        validate_remaining(self.remaining(), count, "skip")?;
        self.pos += count;
        Ok(())
    }

    fn read_u16(&mut self, field: &'static str) -> Result<u16, PostgreSqlError> {
        validate_remaining(self.remaining(), 2, field)?;
        let bytes = [self.bytes[self.pos], self.bytes[self.pos + 1]];
        self.pos += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32, PostgreSqlError> {
        validate_remaining(self.remaining(), 4, field)?;
        let bytes = [
            self.bytes[self.pos],
            self.bytes[self.pos + 1],
            self.bytes[self.pos + 2],
            self.bytes[self.pos + 3],
        ];
        self.pos += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    fn read_i32(&mut self, field: &'static str) -> Result<i32, PostgreSqlError> {
        validate_remaining(self.remaining(), 4, field)?;
        let bytes = [
            self.bytes[self.pos],
            self.bytes[self.pos + 1],
            self.bytes[self.pos + 2],
            self.bytes[self.pos + 3],
        ];
        self.pos += 4;
        Ok(i32::from_be_bytes(bytes))
    }

    fn read_bytes(
        &mut self,
        count: usize,
        field: &'static str,
    ) -> Result<&'a [u8], PostgreSqlError> {
        validate_remaining(self.remaining(), count, field)?;
        let value = &self.bytes[self.pos..self.pos + count];
        self.pos += count;
        Ok(value)
    }

    fn read_cstring(&mut self, field: &'static str) -> Result<&'a str, PostgreSqlError> {
        let relative_end = self.bytes[self.pos..]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(PostgreSqlError::MissingNullTerminator { field })?;

        let value = &self.bytes[self.pos..self.pos + relative_end];
        self.pos += relative_end + 1;

        str::from_utf8(value).map_err(|_| PostgreSqlError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checks::application::postgresql::POSTGRESQL_LENGTH_FIELD_LEN;

    fn parse_bind_execute_sync_payload() -> Vec<u8> {
        let mut payload = Vec::new();

        payload.push(b'P');
        payload.extend_from_slice(&81u32.to_be_bytes());
        payload.push(0);
        payload.extend_from_slice(
            b"SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL READ COMMITTED",
        );
        payload.push(0);
        payload.extend_from_slice(&0u16.to_be_bytes());

        payload.push(b'B');
        payload.extend_from_slice(&12u32.to_be_bytes());
        payload.push(0);
        payload.push(0);
        payload.extend_from_slice(&0u16.to_be_bytes());
        payload.extend_from_slice(&0u16.to_be_bytes());
        payload.extend_from_slice(&0u16.to_be_bytes());

        payload.push(b'E');
        payload.extend_from_slice(&9u32.to_be_bytes());
        payload.push(0);
        payload.extend_from_slice(&1u32.to_be_bytes());

        payload.push(b'S');
        payload.extend_from_slice(&4u32.to_be_bytes());

        payload
    }

    #[test]
    fn parses_parse_bind_execute_sync_messages() {
        let payload = parse_bind_execute_sync_payload();

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();

        assert_eq!(packet.messages.len(), 4);
        assert_eq!(
            packet.messages[0].message_type,
            PostgreSqlMessageType::Parse
        );
        assert_eq!(packet.messages[0].length, 81);
        match &packet.messages[0].body {
            PostgreSqlMessageBody::Parse(parse) => {
                assert_eq!(parse.statement, "");
                assert_eq!(
                    parse.query,
                    "SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL READ COMMITTED"
                );
                assert!(parse.parameter_type_oids.is_empty());
            }
            other => panic!("expected Parse body, got {other:?}"),
        }

        assert_eq!(packet.messages[1].message_type, PostgreSqlMessageType::Bind);
        match &packet.messages[1].body {
            PostgreSqlMessageBody::Bind(bind) => {
                assert_eq!(bind.portal, "");
                assert_eq!(bind.statement, "");
                assert!(bind.parameter_formats.is_empty());
                assert!(bind.parameter_values.is_empty());
                assert!(bind.result_formats.is_empty());
            }
            other => panic!("expected Bind body, got {other:?}"),
        }

        assert_eq!(
            packet.messages[2].message_type,
            PostgreSqlMessageType::ErrorResponseOrExecute
        );
        match &packet.messages[2].body {
            PostgreSqlMessageBody::Execute(execute) => {
                assert_eq!(execute.portal, "");
                assert_eq!(execute.max_rows, 1);
            }
            other => panic!("expected Execute body, got {other:?}"),
        }

        assert_eq!(
            packet.messages[3].message_type,
            PostgreSqlMessageType::ParameterStatusOrSync
        );
        assert_eq!(packet.messages[3].body, PostgreSqlMessageBody::Empty);
    }

    #[test]
    fn rejects_truncated_typed_message() {
        let payload = [b'Q', 0x00, 0x00, 0x00, 0x20, b'S', b'E', b'L'];

        let err = PostgreSqlPacket::try_from(payload.as_slice()).unwrap_err();

        assert_eq!(
            err,
            PostgreSqlError::LengthMismatch {
                expected: 33,
                actual: 8
            }
        );
    }

    #[test]
    fn parses_startup_message() {
        let mut payload = Vec::new();
        let body = b"user\0postgres\0database\0postgres\0\0";
        let length = (POSTGRESQL_LENGTH_FIELD_LEN + 4 + body.len()) as u32;

        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3.to_be_bytes());
        payload.extend_from_slice(body);

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();

        assert_eq!(packet.messages.len(), 1);
        assert_eq!(
            packet.messages[0].message_type,
            PostgreSqlMessageType::StartupMessage
        );
        match &packet.messages[0].body {
            PostgreSqlMessageBody::Startup(startup) => {
                assert_eq!(startup.protocol_version, POSTGRESQL_PROTOCOL_VERSION_3);
                assert_eq!(
                    startup.parameters,
                    vec![("user", "postgres"), ("database", "postgres")]
                );
            }
            other => panic!("expected Startup body, got {other:?}"),
        }
    }

    #[test]
    fn parses_backend_parameter_status_as_raw_body() {
        let mut body = Vec::new();
        body.extend_from_slice(b"server_version");
        body.push(0);
        body.extend_from_slice(b"16.0");
        body.push(0);

        let mut payload = Vec::new();
        payload.push(b'S');
        let length = (POSTGRESQL_LENGTH_FIELD_LEN + body.len()) as u32;
        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(&body);

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();

        assert_eq!(packet.messages.len(), 1);
        assert_eq!(
            packet.messages[0].message_type,
            PostgreSqlMessageType::ParameterStatusOrSync
        );
        assert_eq!(
            packet.messages[0].body,
            PostgreSqlMessageBody::Raw(body.as_slice())
        );
    }
}
