// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::convert::TryFrom;
use std::str;

use crate::{
    checks::application::postgresql::{
        POSTGRESQL_SECRET_KEY_MAX_LEN, POSTGRESQL_SECRET_KEY_MIN_LEN, POSTGRESQL_TYPED_HEADER_LEN,
        POSTGRESQL_UNTYPED_HEADER_LEN, validate_no_trailing_bytes, validate_packet_not_empty,
        validate_remaining, validate_secret_key_length, validate_typed_header_available,
        validate_typed_message_available, validate_untyped_header_available,
        validate_untyped_message_available,
    },
    errors::application::postgresql::PostgreSqlError,
};

const POSTGRESQL_PROTOCOL_VERSION_3_0: u32 = 196_608;
const POSTGRESQL_PROTOCOL_VERSION_3_2: u32 = 196_610;
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
    Query {
        query: &'a str,
    },
    Startup(PostgreSqlStartup<'a>),
    CancelRequest {
        process_id: u32,
        secret_key: &'a [u8],
    },
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

pub(crate) fn is_likely_postgresql_payload(payload: &[u8]) -> bool {
    let Ok(packet) = PostgreSqlPacket::try_from(payload) else {
        return false;
    };

    !packet.messages.is_empty()
        && packet
            .messages
            .iter()
            .all(message_has_detection_compatible_shape)
        && packet
            .messages
            .iter()
            .any(message_has_strong_detection_evidence)
}

fn message_has_strong_detection_evidence(message: &PostgreSqlMessage<'_>) -> bool {
    match (&message.message_type, &message.body) {
        (PostgreSqlMessageType::StartupMessage, PostgreSqlMessageBody::Startup(startup)) => {
            message.payload.last() == Some(&0) && startup_has_known_parameter(startup)
        }
        (PostgreSqlMessageType::SslRequest | PostgreSqlMessageType::GssEncRequest, _) => {
            message.length == POSTGRESQL_UNTYPED_HEADER_LEN as u32 && message.payload.is_empty()
        }
        (
            PostgreSqlMessageType::CancelRequest,
            PostgreSqlMessageBody::CancelRequest {
                process_id: _,
                secret_key,
            },
        ) => {
            postgresql_secret_key_len_is_valid(secret_key.len())
                && message.payload.len() == 4 + secret_key.len()
        }
        (PostgreSqlMessageType::Parse, PostgreSqlMessageBody::Parse(parse)) => {
            looks_like_sql(parse.query)
        }
        (PostgreSqlMessageType::Query, PostgreSqlMessageBody::Query { query }) => {
            looks_like_sql(query)
        }
        (PostgreSqlMessageType::Authentication, PostgreSqlMessageBody::Raw(body)) => {
            authentication_body_has_strong_evidence(body)
        }
        (PostgreSqlMessageType::CloseOrCommandComplete, PostgreSqlMessageBody::Raw(body)) => {
            command_complete_body_is_likely(body)
        }
        (PostgreSqlMessageType::ErrorResponseOrExecute, PostgreSqlMessageBody::Raw(body))
        | (PostgreSqlMessageType::NoticeResponse, PostgreSqlMessageBody::Raw(body)) => {
            error_or_notice_body_is_likely(body)
        }
        (PostgreSqlMessageType::ParameterStatusOrSync, PostgreSqlMessageBody::Raw(body)) => {
            parameter_status_body_is_likely(body)
        }
        _ => false,
    }
}

fn message_has_detection_compatible_shape(message: &PostgreSqlMessage<'_>) -> bool {
    if message_has_strong_detection_evidence(message) {
        return true;
    }

    match (&message.message_type, &message.body) {
        (_, PostgreSqlMessageBody::Empty) => message.payload.is_empty(),
        (_, PostgreSqlMessageBody::Bind(_))
        | (_, PostgreSqlMessageBody::Execute(_))
        | (_, PostgreSqlMessageBody::CancelRequest { .. }) => true,
        (PostgreSqlMessageType::Parse, PostgreSqlMessageBody::Parse(parse)) => {
            is_plain_text(parse.statement) && is_plain_text(parse.query)
        }
        (PostgreSqlMessageType::Query, PostgreSqlMessageBody::Query { query }) => {
            is_plain_text(query)
        }
        (PostgreSqlMessageType::StartupMessage, PostgreSqlMessageBody::Startup(startup)) => {
            message.payload.last() == Some(&0)
                && !startup.parameters.is_empty()
                && startup
                    .parameters
                    .iter()
                    .all(|(key, value)| is_plain_text(key) && is_plain_text(value))
        }
        (PostgreSqlMessageType::ErrorResponseOrExecute, PostgreSqlMessageBody::Raw(body)) => {
            error_or_notice_body_is_likely(body)
        }
        (PostgreSqlMessageType::NoticeResponse, PostgreSqlMessageBody::Raw(body)) => {
            error_or_notice_body_is_likely(body)
        }
        (PostgreSqlMessageType::ParameterStatusOrSync, PostgreSqlMessageBody::Raw(body)) => {
            body.is_empty() || parameter_status_body_is_likely(body)
        }
        (PostgreSqlMessageType::Authentication, PostgreSqlMessageBody::Raw(body)) => {
            authentication_body_is_compatible(body)
        }
        (PostgreSqlMessageType::BackendKeyData, PostgreSqlMessageBody::Raw(body)) => {
            backend_key_data_body_is_compatible(body)
        }
        (PostgreSqlMessageType::CloseOrCommandComplete, PostgreSqlMessageBody::Raw(body)) => {
            close_body_is_likely(body) || command_complete_body_is_likely(body)
        }
        (PostgreSqlMessageType::ReadyForQuery, PostgreSqlMessageBody::Raw(body)) => {
            matches!(body, [b'I' | b'T' | b'E'])
        }
        (_, PostgreSqlMessageBody::Raw(_)) => true,
        _ => false,
    }
}

fn startup_has_known_parameter(startup: &PostgreSqlStartup<'_>) -> bool {
    startup.parameters.iter().any(|(key, value)| {
        !value.is_empty()
            && matches!(
                *key,
                "user"
                    | "database"
                    | "application_name"
                    | "client_encoding"
                    | "options"
                    | "replication"
            )
    })
}

fn postgresql_startup_protocol_version_is_supported(code: u32) -> bool {
    matches!(
        code,
        POSTGRESQL_PROTOCOL_VERSION_3_0 | POSTGRESQL_PROTOCOL_VERSION_3_2
    )
}

fn looks_like_sql(query: &str) -> bool {
    let Some(token) = first_ascii_token(query) else {
        return false;
    };

    is_sql_keyword(token)
}

fn is_sql_keyword(token: &str) -> bool {
    const SQL_KEYWORDS: &[&str] = &[
        "ABORT",
        "ALTER",
        "ANALYZE",
        "BEGIN",
        "CALL",
        "CHECKPOINT",
        "CLOSE",
        "CLUSTER",
        "COMMENT",
        "COMMIT",
        "COPY",
        "CREATE",
        "DEALLOCATE",
        "DECLARE",
        "DELETE",
        "DISCARD",
        "DO",
        "DROP",
        "EXECUTE",
        "EXPLAIN",
        "FETCH",
        "GRANT",
        "INSERT",
        "LISTEN",
        "LOAD",
        "LOCK",
        "MERGE",
        "MOVE",
        "NOTIFY",
        "PREPARE",
        "REASSIGN",
        "REFRESH",
        "REINDEX",
        "RELEASE",
        "RESET",
        "REVOKE",
        "ROLLBACK",
        "SAVEPOINT",
        "SELECT",
        "SET",
        "SHOW",
        "START",
        "TRUNCATE",
        "UNLISTEN",
        "UPDATE",
        "VACUUM",
        "VALUES",
        "WITH",
    ];

    SQL_KEYWORDS
        .iter()
        .any(|keyword| token.eq_ignore_ascii_case(keyword))
}

fn first_ascii_token(query: &str) -> Option<&str> {
    let mut rest = query.trim_start();

    loop {
        if let Some(stripped) = rest.strip_prefix("--") {
            let newline = stripped.find('\n')?;
            rest = stripped[newline + 1..].trim_start();
            continue;
        }

        if let Some(stripped) = rest.strip_prefix("/*") {
            let end = stripped.find("*/")?;
            rest = stripped[end + 2..].trim_start();
            continue;
        }

        break;
    }

    let token_end = rest
        .bytes()
        .position(|byte| !byte.is_ascii_alphabetic())
        .unwrap_or(rest.len());

    if token_end == 0 {
        return None;
    }

    let token = &rest[..token_end];
    if !token.is_ascii() {
        return None;
    }

    Some(token)
}

fn authentication_body_has_strong_evidence(body: &[u8]) -> bool {
    if body.len() < 4 {
        return false;
    }

    let auth_code = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
    auth_code == 10 && has_cstring_list(&body[4..])
}

fn authentication_body_is_compatible(body: &[u8]) -> bool {
    if body.len() < 4 {
        return false;
    }

    let auth_code = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
    match auth_code {
        0 | 2 | 3 | 6 | 7 | 8 => body.len() == 4,
        5 => body.len() == 8,
        10 => has_cstring_list(&body[4..]),
        11 | 12 => body.len() >= 4,
        _ => false,
    }
}

fn backend_key_data_body_is_compatible(body: &[u8]) -> bool {
    body.len() >= 4 + POSTGRESQL_SECRET_KEY_MIN_LEN
        && body.len() <= 4 + POSTGRESQL_SECRET_KEY_MAX_LEN
}

fn parameter_status_body_is_likely(body: &[u8]) -> bool {
    let Some((key, value)) = parse_two_cstrings(body) else {
        return false;
    };

    !value.is_empty()
        && matches!(
            key,
            "application_name"
                | "client_encoding"
                | "DateStyle"
                | "default_transaction_read_only"
                | "in_hot_standby"
                | "integer_datetimes"
                | "IntervalStyle"
                | "is_superuser"
                | "server_encoding"
                | "server_version"
                | "session_authorization"
                | "standard_conforming_strings"
                | "TimeZone"
        )
}

fn command_complete_body_is_likely(body: &[u8]) -> bool {
    let Some(tag) = parse_single_cstring(body) else {
        return false;
    };
    let Some(token) = first_ascii_token(tag) else {
        return false;
    };

    looks_like_sql(token)
}

fn close_body_is_likely(body: &[u8]) -> bool {
    matches!(body.first(), Some(b'S' | b'P')) && str_from_cstring(&body[1..]).is_some()
}

fn error_or_notice_body_is_likely(body: &[u8]) -> bool {
    if body.len() < 2 || body.last() != Some(&0) {
        return false;
    }

    let mut offset = 0usize;
    let mut saw_message = false;
    let mut saw_sqlstate = false;
    let mut saw_severity = false;

    while offset < body.len() - 1 {
        let field = body[offset];
        if !field.is_ascii_alphabetic() {
            return false;
        }
        offset += 1;

        let Some(end) = body[offset..].iter().position(|byte| *byte == 0) else {
            return false;
        };
        if end == 0 {
            return false;
        }

        let value = &body[offset..offset + end];
        let Ok(value) = str::from_utf8(value) else {
            return false;
        };
        if !is_plain_text(value) {
            return false;
        }

        match field {
            b'C' => saw_sqlstate = value.len() == 5 && value.bytes().all(|byte| byte.is_ascii()),
            b'M' => saw_message = true,
            b'S' | b'V' => saw_severity = true,
            _ => {}
        }

        offset += end + 1;
    }

    offset == body.len() - 1 && (saw_message || saw_sqlstate || saw_severity)
}

fn has_cstring_list(body: &[u8]) -> bool {
    if body.len() < 2 || body.last() != Some(&0) {
        return false;
    }

    let mut offset = 0usize;
    let mut count = 0usize;

    while offset < body.len() - 1 {
        let Some(end) = body[offset..].iter().position(|byte| *byte == 0) else {
            return false;
        };
        if end == 0 {
            return false;
        }
        if str::from_utf8(&body[offset..offset + end])
            .map(|value| !is_plain_text(value))
            .unwrap_or(true)
        {
            return false;
        }
        offset += end + 1;
        count += 1;
    }

    count > 0 && offset == body.len() - 1
}

fn parse_single_cstring(body: &[u8]) -> Option<&str> {
    if body.is_empty() || body.last() != Some(&0) {
        return None;
    }

    let value = &body[..body.len() - 1];
    if value.contains(&0) {
        return None;
    }

    let value = str::from_utf8(value).ok()?;
    is_plain_text(value).then_some(value)
}

fn parse_two_cstrings(body: &[u8]) -> Option<(&str, &str)> {
    let first_end = body.iter().position(|byte| *byte == 0)?;
    let first = str::from_utf8(&body[..first_end]).ok()?;
    let rest = &body[first_end + 1..];
    let second = parse_single_cstring(rest)?;

    (is_plain_text(first) && is_plain_text(second)).then_some((first, second))
}

fn str_from_cstring(body: &[u8]) -> Option<&str> {
    parse_single_cstring(body)
}

fn is_plain_text(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| !ch.is_control() || matches!(ch, '\t' | '\n' | '\r'))
}

fn postgresql_secret_key_len_is_valid(length: usize) -> bool {
    (POSTGRESQL_SECRET_KEY_MIN_LEN..=POSTGRESQL_SECRET_KEY_MAX_LEN).contains(&length)
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
        code if postgresql_startup_protocol_version_is_supported(code) => (
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
            validate_secret_key_length(cur.remaining(), "secret_key")?;
            let secret_key = cur.read_bytes(cur.remaining(), "secret_key")?;
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
    fn likely_payload_accepts_parse_bind_execute_sync_messages() {
        let payload = parse_bind_execute_sync_payload();

        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_accepts_lowercase_query_message() {
        let mut payload = Vec::new();
        let query = b"select 1\0";

        payload.push(b'Q');
        let length = (POSTGRESQL_LENGTH_FIELD_LEN + query.len()) as u32;
        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(query);

        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_rejects_single_sync_message() {
        let payload = [b'S', 0x00, 0x00, 0x00, 0x04];

        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_ok());
        assert!(!is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_rejects_raw_message_without_strong_evidence() {
        let payload = [b'C', 0x00, 0x00, 0x00, 0x05, b'x'];

        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_ok());
        assert!(!is_likely_postgresql_payload(payload.as_slice()));
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
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3_0.to_be_bytes());
        payload.extend_from_slice(body);

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();

        assert_eq!(packet.messages.len(), 1);
        assert_eq!(
            packet.messages[0].message_type,
            PostgreSqlMessageType::StartupMessage
        );
        match &packet.messages[0].body {
            PostgreSqlMessageBody::Startup(startup) => {
                assert_eq!(startup.protocol_version, POSTGRESQL_PROTOCOL_VERSION_3_0);
                assert_eq!(
                    startup.parameters,
                    vec![("user", "postgres"), ("database", "postgres")]
                );
            }
            other => panic!("expected Startup body, got {other:?}"),
        }
    }

    #[test]
    fn parses_protocol_3_2_startup_message() {
        let mut payload = Vec::new();
        let body = b"user\0postgres\0database\0postgres\0\0";
        let length = (POSTGRESQL_LENGTH_FIELD_LEN + 4 + body.len()) as u32;

        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3_2.to_be_bytes());
        payload.extend_from_slice(body);

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();

        match &packet.messages[0].body {
            PostgreSqlMessageBody::Startup(startup) => {
                assert_eq!(startup.protocol_version, POSTGRESQL_PROTOCOL_VERSION_3_2);
            }
            other => panic!("expected Startup body, got {other:?}"),
        }
    }

    #[test]
    fn likely_payload_accepts_startup_message() {
        let mut payload = Vec::new();
        let body = b"user\0postgres\0database\0postgres\0\0";
        let length = (POSTGRESQL_LENGTH_FIELD_LEN + 4 + body.len()) as u32;

        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3_0.to_be_bytes());
        payload.extend_from_slice(body);

        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn parses_cancel_request_with_variable_secret_key() {
        let secret_key = b"0123456789abcdef";
        let mut payload = Vec::new();
        let length = (POSTGRESQL_UNTYPED_HEADER_LEN + 4 + secret_key.len()) as u32;

        payload.extend_from_slice(&length.to_be_bytes());
        payload.extend_from_slice(&POSTGRESQL_CANCEL_REQUEST_CODE.to_be_bytes());
        payload.extend_from_slice(&42u32.to_be_bytes());
        payload.extend_from_slice(secret_key);

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();

        match &packet.messages[0].body {
            PostgreSqlMessageBody::CancelRequest {
                process_id,
                secret_key: parsed_secret_key,
            } => {
                assert_eq!(*process_id, 42);
                assert_eq!(*parsed_secret_key, secret_key);
            }
            other => panic!("expected CancelRequest body, got {other:?}"),
        }

        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_rejects_single_ready_for_query_message() {
        let payload = [b'Z', 0x00, 0x00, 0x00, 0x05, b'I'];

        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_ok());
        assert!(!is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_rejects_single_backend_key_data_message() {
        let mut payload = Vec::new();
        payload.push(b'K');
        payload.extend_from_slice(&12u32.to_be_bytes());
        payload.extend_from_slice(&42u32.to_be_bytes());
        payload.extend_from_slice(&24u32.to_be_bytes());

        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_ok());
        assert!(!is_likely_postgresql_payload(payload.as_slice()));
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

#[cfg(test)]
mod detection_tests {
    use super::*;

    fn typed(kind: u8, body: &[u8]) -> Vec<u8> {
        let mut payload = vec![kind];
        payload.extend_from_slice(&((4 + body.len()) as u32).to_be_bytes());
        payload.extend_from_slice(body);
        payload
    }

    #[test]
    fn message_type_mapping_is_complete() {
        let cases: &[(u8, PostgreSqlMessageType, &str)] = &[
            (b'R', PostgreSqlMessageType::Authentication, "Authentication"),
            (b'K', PostgreSqlMessageType::BackendKeyData, "BackendKeyData"),
            (b'B', PostgreSqlMessageType::Bind, "Bind"),
            (b'2', PostgreSqlMessageType::BindComplete, "BindComplete"),
            (b'C', PostgreSqlMessageType::CloseOrCommandComplete, "CloseOrCommandComplete"),
            (b'3', PostgreSqlMessageType::CloseComplete, "CloseComplete"),
            (b'W', PostgreSqlMessageType::CopyBothResponse, "CopyBothResponse"),
            (b'd', PostgreSqlMessageType::CopyData, "CopyData"),
            (b'c', PostgreSqlMessageType::CopyDone, "CopyDone"),
            (b'f', PostgreSqlMessageType::CopyFail, "CopyFail"),
            (b'G', PostgreSqlMessageType::CopyInResponse, "CopyInResponse"),
            (b'D', PostgreSqlMessageType::DataRowOrDescribe, "DataRowOrDescribe"),
            (b'I', PostgreSqlMessageType::EmptyQueryResponse, "EmptyQueryResponse"),
            (b'E', PostgreSqlMessageType::ErrorResponseOrExecute, "ErrorResponseOrExecute"),
            (b'H', PostgreSqlMessageType::FlushOrCopyOutResponse, "FlushOrCopyOutResponse"),
            (b'F', PostgreSqlMessageType::FunctionCall, "FunctionCall"),
            (b'V', PostgreSqlMessageType::FunctionCallResponse, "FunctionCallResponse"),
            (b'n', PostgreSqlMessageType::NoData, "NoData"),
            (b'N', PostgreSqlMessageType::NoticeResponse, "NoticeResponse"),
            (b'A', PostgreSqlMessageType::NotificationResponse, "NotificationResponse"),
            (b't', PostgreSqlMessageType::ParameterDescription, "ParameterDescription"),
            (b'S', PostgreSqlMessageType::ParameterStatusOrSync, "ParameterStatusOrSync"),
            (b'P', PostgreSqlMessageType::Parse, "Parse"),
            (b'1', PostgreSqlMessageType::ParseComplete, "ParseComplete"),
            (b's', PostgreSqlMessageType::PortalSuspended, "PortalSuspended"),
            (b'Q', PostgreSqlMessageType::Query, "Query"),
            (b'Z', PostgreSqlMessageType::ReadyForQuery, "ReadyForQuery"),
            (b'T', PostgreSqlMessageType::RowDescription, "RowDescription"),
            (b'X', PostgreSqlMessageType::Terminate, "Terminate"),
        ];

        for (byte, expected, expected_name) in cases {
            let message_type = PostgreSqlMessageType::try_from(*byte).unwrap();
            assert_eq!(message_type, *expected);
            assert_eq!(message_type.name(), *expected_name);
        }

        assert!(matches!(
            PostgreSqlMessageType::try_from(b'@'),
            Err(PostgreSqlError::InvalidMessageType(_))
        ));

        // Types sans octet sur le fil, juste le nom
        assert_eq!(PostgreSqlMessageType::CancelRequest.name(), "CancelRequest");
        assert_eq!(PostgreSqlMessageType::GssEncRequest.name(), "GssEncRequest");
        assert_eq!(PostgreSqlMessageType::SslRequest.name(), "SslRequest");
        assert_eq!(PostgreSqlMessageType::StartupMessage.name(), "StartupMessage");
    }

    #[test]
    fn parses_startup_message_with_parameters() {
        let mut body = Vec::new();
        body.extend_from_slice(b"user\0admin\0database\0app\0");
        body.push(0); // terminateur de liste

        let mut payload = ((8 + body.len()) as u32).to_be_bytes().to_vec();
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3_0.to_be_bytes());
        payload.extend_from_slice(&body);

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
        assert_eq!(packet.messages.len(), 1);
        assert_eq!(
            packet.messages[0].message_type,
            PostgreSqlMessageType::StartupMessage
        );
        match &packet.messages[0].body {
            PostgreSqlMessageBody::Startup(startup) => {
                assert_eq!(startup.protocol_version, POSTGRESQL_PROTOCOL_VERSION_3_0);
                assert_eq!(
                    startup.parameters,
                    vec![("user", "admin"), ("database", "app")]
                );
            }
            other => panic!("attendu Startup, obtenu {other:?}"),
        }

        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn parses_ssl_and_gssenc_requests() {
        for (code, expected) in [
            (POSTGRESQL_SSL_REQUEST_CODE, PostgreSqlMessageType::SslRequest),
            (POSTGRESQL_GSSENC_REQUEST_CODE, PostgreSqlMessageType::GssEncRequest),
        ] {
            let mut payload = 8u32.to_be_bytes().to_vec();
            payload.extend_from_slice(&code.to_be_bytes());

            let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
            assert_eq!(packet.messages[0].message_type, expected);
            assert_eq!(packet.messages[0].body, PostgreSqlMessageBody::Empty);
            assert!(is_likely_postgresql_payload(payload.as_slice()));
        }
    }

    #[test]
    fn parses_cancel_request() {
        let mut payload = 16u32.to_be_bytes().to_vec();
        payload.extend_from_slice(&POSTGRESQL_CANCEL_REQUEST_CODE.to_be_bytes());
        payload.extend_from_slice(&1234u32.to_be_bytes()); // process id
        payload.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // secret key

        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
        match &packet.messages[0].body {
            PostgreSqlMessageBody::CancelRequest {
                process_id,
                secret_key,
            } => {
                assert_eq!(*process_id, 1234);
                assert_eq!(*secret_key, &[0xAA, 0xBB, 0xCC, 0xDD]);
            }
            other => panic!("attendu CancelRequest, obtenu {other:?}"),
        }
        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn rejects_unsupported_startup_code() {
        let mut payload = 8u32.to_be_bytes().to_vec();
        payload.extend_from_slice(&12345u32.to_be_bytes());
        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_err());
    }

    #[test]
    fn likely_payload_accepts_backend_session_establishment() {
        // Authentication OK + BackendKeyData + ParameterStatus + ReadyForQuery,
        // avec un CommandComplete SQL comme preuve forte
        let mut payload = Vec::new();
        payload.extend_from_slice(&typed(b'R', &0u32.to_be_bytes())); // auth ok
        let mut key_body = 42u32.to_be_bytes().to_vec();
        key_body.extend_from_slice(&[1, 2, 3, 4]);
        payload.extend_from_slice(&typed(b'K', &key_body));
        payload.extend_from_slice(&typed(b'S', b"TimeZone\0UTC\0"));
        payload.extend_from_slice(&typed(b'C', b"SELECT 1\0"));
        payload.extend_from_slice(&typed(b'Z', b"I"));

        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_accepts_error_response() {
        let error_body = b"SERROR\0C42P01\0Mrelation does not exist\0\0";
        let payload = typed(b'E', error_body);
        assert!(is_likely_postgresql_payload(payload.as_slice()));

        let notice = typed(b'N', error_body);
        assert!(is_likely_postgresql_payload(notice.as_slice()));
    }

    #[test]
    fn likely_payload_accepts_sasl_authentication() {
        // Authentication code 10 (SASL) suivi d'une liste de mécanismes
        let mut body = 10u32.to_be_bytes().to_vec();
        body.extend_from_slice(b"SCRAM-SHA-256\0\0"); // liste terminée par un NUL supplémentaire
        let payload = typed(b'R', &body);
        assert!(is_likely_postgresql_payload(payload.as_slice()));
    }

    #[test]
    fn likely_payload_rejects_garbage() {
        assert!(!is_likely_postgresql_payload(&[0xDE, 0xAD, 0xBE, 0xEF]));
        assert!(!is_likely_postgresql_payload(&[]));
    }

    #[test]
    fn parses_bind_with_parameter_values() {
        let mut body = Vec::new();
        body.extend_from_slice(b"portal\0stmt\0");
        body.extend_from_slice(&1u16.to_be_bytes()); // 1 format
        body.extend_from_slice(&1u16.to_be_bytes()); // format binaire
        body.extend_from_slice(&2u16.to_be_bytes()); // 2 valeurs
        body.extend_from_slice(&3i32.to_be_bytes());
        body.extend_from_slice(b"abc");
        body.extend_from_slice(&(-1i32).to_be_bytes()); // NULL
        body.extend_from_slice(&1u16.to_be_bytes()); // 1 format résultat
        body.extend_from_slice(&0u16.to_be_bytes());

        let payload = typed(b'B', &body);
        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
        match &packet.messages[0].body {
            PostgreSqlMessageBody::Bind(bind) => {
                assert_eq!(bind.portal, "portal");
                assert_eq!(bind.statement, "stmt");
                assert_eq!(bind.parameter_formats, vec![1]);
                assert_eq!(bind.parameter_values, vec![Some(&b"abc"[..]), None]);
                assert_eq!(bind.result_formats, vec![0]);
            }
            other => panic!("attendu Bind, obtenu {other:?}"),
        }
    }

    #[test]
    fn parses_parse_with_parameter_oids() {
        let mut body = Vec::new();
        body.extend_from_slice(b"s1\0SELECT $1\0");
        body.extend_from_slice(&1u16.to_be_bytes());
        body.extend_from_slice(&23u32.to_be_bytes()); // oid int4

        let payload = typed(b'P', &body);
        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
        match &packet.messages[0].body {
            PostgreSqlMessageBody::Parse(parse) => {
                assert_eq!(parse.statement, "s1");
                assert_eq!(parse.query, "SELECT $1");
                assert_eq!(parse.parameter_type_oids, vec![23]);
            }
            other => panic!("attendu Parse, obtenu {other:?}"),
        }
    }

    #[test]
    fn bind_rejects_negative_value_length() {
        let mut body = Vec::new();
        body.extend_from_slice(b"\0\0");
        body.extend_from_slice(&0u16.to_be_bytes());
        body.extend_from_slice(&1u16.to_be_bytes());
        body.extend_from_slice(&(-2i32).to_be_bytes());

        let payload = typed(b'B', &body);
        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_err());
    }

    #[test]
    fn query_rejects_missing_null_terminator() {
        let payload = typed(b'Q', b"SELECT 1"); // pas de NUL final
        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_err());
    }

    #[test]
    fn query_rejects_invalid_utf8() {
        let payload = typed(b'Q', &[0xFF, 0xFE, 0x00]);
        assert!(PostgreSqlPacket::try_from(payload.as_slice()).is_err());
    }

    #[test]
    fn empty_query_response_and_terminate() {
        let mut payload = typed(b'I', &[]);
        payload.extend_from_slice(&typed(b'X', &[]));
        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
        assert_eq!(packet.messages[0].body, PostgreSqlMessageBody::Empty);
        assert_eq!(packet.messages[1].body, PostgreSqlMessageBody::Empty);
    }

    #[test]
    fn flush_with_body_stays_raw() {
        let payload = typed(b'H', &[0x01]);
        let packet = PostgreSqlPacket::try_from(payload.as_slice()).unwrap();
        assert!(matches!(
            packet.messages[0].body,
            PostgreSqlMessageBody::Raw(_)
        ));
    }

    #[test]
    fn first_ascii_token_handles_comments() {
        assert_eq!(first_ascii_token("SELECT 1"), Some("SELECT"));
        assert_eq!(first_ascii_token("  -- comment\nSELECT 1"), Some("SELECT"));
        assert_eq!(first_ascii_token("/* bloc */ UPDATE t"), Some("UPDATE"));
        assert_eq!(first_ascii_token("123"), None);
        assert_eq!(first_ascii_token(""), None);
        assert_eq!(first_ascii_token("-- sans fin"), None);
        assert_eq!(first_ascii_token("/* sans fin"), None);
    }

    #[test]
    fn looks_like_sql_matches_keywords_case_insensitive() {
        assert!(looks_like_sql("select * from t"));
        assert!(looks_like_sql("INSERT INTO t VALUES (1)"));
        assert!(!looks_like_sql("bonjour"));
        assert!(!looks_like_sql(""));
    }

    #[test]
    fn authentication_body_compatibility_codes() {
        for code in [0u32, 2, 3, 6, 7, 8] {
            assert!(authentication_body_is_compatible(&code.to_be_bytes()));
            // longueur incorrecte
            let mut long = code.to_be_bytes().to_vec();
            long.push(0);
            assert!(!authentication_body_is_compatible(&long));
        }

        // MD5 : code 5 + sel de 4 octets
        let mut md5 = 5u32.to_be_bytes().to_vec();
        md5.extend_from_slice(&[1, 2, 3, 4]);
        assert!(authentication_body_is_compatible(&md5));

        // SASL continue/final : codes 11 et 12
        assert!(authentication_body_is_compatible(&11u32.to_be_bytes()));
        assert!(authentication_body_is_compatible(&12u32.to_be_bytes()));

        // code inconnu, corps trop court
        assert!(!authentication_body_is_compatible(&99u32.to_be_bytes()));
        assert!(!authentication_body_is_compatible(&[0, 0]));
    }

    #[test]
    fn error_body_heuristic_edge_cases() {
        assert!(!error_or_notice_body_is_likely(&[]));
        assert!(!error_or_notice_body_is_likely(b"M")); // pas de NUL final
        assert!(!error_or_notice_body_is_likely(&[0x01, b'x', 0, 0])); // champ non alpha
        assert!(!error_or_notice_body_is_likely(b"M\0\0")); // valeur vide
        assert!(!error_or_notice_body_is_likely(b"Zvalue\0\0")); // aucun champ significatif
        assert!(error_or_notice_body_is_likely(b"Mmessage\0\0"));
    }

    #[test]
    fn cstring_helpers_edge_cases() {
        assert!(has_cstring_list(b"one\0two\0\0")); // liste terminée par un NUL supplémentaire
        assert!(!has_cstring_list(b"one\0two\0")); // sans terminateur de liste
        assert!(!has_cstring_list(b""));
        assert!(!has_cstring_list(b"one"));
        assert!(!has_cstring_list(b"\0\0"));

        assert_eq!(parse_single_cstring(b"tag\0"), Some("tag"));
        assert_eq!(parse_single_cstring(b""), None);
        assert_eq!(parse_single_cstring(b"a\0b\0"), None); // NUL au milieu

        assert_eq!(parse_two_cstrings(b"k\0v\0"), Some(("k", "v")));
        assert_eq!(parse_two_cstrings(b"k\0"), None);

        assert!(close_body_is_likely(b"Sstmt\0"));
        assert!(close_body_is_likely(b"Pportal\0"));
        assert!(!close_body_is_likely(b"Xstmt\0"));

        assert!(command_complete_body_is_likely(b"SELECT 1\0"));
        assert!(!command_complete_body_is_likely(b"NOPE 1\0"));
        assert!(!command_complete_body_is_likely(b""));
    }

    #[test]
    fn secret_key_length_bounds() {
        assert!(!postgresql_secret_key_len_is_valid(3));
        assert!(postgresql_secret_key_len_is_valid(4));
        assert!(postgresql_secret_key_len_is_valid(256));
        assert!(!postgresql_secret_key_len_is_valid(257));
    }
}
