// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::str;

use crate::{
    errors::application::postgresql::PostgreSqlError,
    parse::application::protocols::postgresql::{
        POSTGRESQL_CANCEL_REQUEST_CODE, POSTGRESQL_GSSENC_REQUEST_CODE,
        POSTGRESQL_PROTOCOL_VERSION_3_0, POSTGRESQL_PROTOCOL_VERSION_3_2,
        POSTGRESQL_SSL_REQUEST_CODE, PostgreSqlMessageType,
    },
};

pub const POSTGRESQL_TYPED_HEADER_LEN: usize = 5;
pub const POSTGRESQL_UNTYPED_HEADER_LEN: usize = 8;
pub const POSTGRESQL_LENGTH_FIELD_LEN: usize = 4;
pub const POSTGRESQL_SECRET_KEY_MIN_LEN: usize = 4;
pub const POSTGRESQL_SECRET_KEY_MAX_LEN: usize = 256;

pub fn validate_packet_not_empty(payload: &[u8]) -> Result<(), PostgreSqlError> {
    if payload.is_empty() {
        return Err(PostgreSqlError::EmptyPacket);
    }

    Ok(())
}

pub fn validate_typed_header_available(payload: &[u8]) -> Result<(), PostgreSqlError> {
    if payload.len() < POSTGRESQL_TYPED_HEADER_LEN {
        return Err(PostgreSqlError::BufferTooSmall {
            needed: POSTGRESQL_TYPED_HEADER_LEN,
            actual: payload.len(),
        });
    }

    Ok(())
}

pub fn validate_untyped_header_available(payload: &[u8]) -> Result<(), PostgreSqlError> {
    if payload.len() < POSTGRESQL_UNTYPED_HEADER_LEN {
        return Err(PostgreSqlError::BufferTooSmall {
            needed: POSTGRESQL_UNTYPED_HEADER_LEN,
            actual: payload.len(),
        });
    }

    Ok(())
}

pub fn validate_message_length(length: u32) -> Result<(), PostgreSqlError> {
    if length < POSTGRESQL_LENGTH_FIELD_LEN as u32 {
        return Err(PostgreSqlError::InvalidMessageLength { got: length });
    }

    Ok(())
}

pub fn validate_typed_message_available(
    payload: &[u8],
    length: u32,
) -> Result<usize, PostgreSqlError> {
    validate_message_length(length)?;

    let expected = 1usize + length as usize;
    if payload.len() < expected {
        return Err(PostgreSqlError::LengthMismatch {
            expected,
            actual: payload.len(),
        });
    }

    Ok(expected)
}

pub fn validate_untyped_message_available(
    payload: &[u8],
    length: u32,
) -> Result<usize, PostgreSqlError> {
    if length < POSTGRESQL_UNTYPED_HEADER_LEN as u32 {
        return Err(PostgreSqlError::InvalidMessageLength { got: length });
    }

    let expected = length as usize;
    if payload.len() != expected {
        return Err(PostgreSqlError::LengthMismatch {
            expected,
            actual: payload.len(),
        });
    }

    Ok(expected)
}

pub fn validate_remaining(
    remaining: usize,
    needed: usize,
    _field: &'static str,
) -> Result<(), PostgreSqlError> {
    if remaining < needed {
        return Err(PostgreSqlError::BufferTooSmall {
            needed,
            actual: remaining,
        });
    }

    Ok(())
}

pub fn validate_no_trailing_bytes(
    remaining: usize,
    message_type: &'static str,
) -> Result<(), PostgreSqlError> {
    if remaining != 0 {
        return Err(PostgreSqlError::TrailingBytes {
            message_type,
            remaining,
        });
    }

    Ok(())
}

pub fn validate_secret_key_length(
    length: usize,
    field: &'static str,
) -> Result<(), PostgreSqlError> {
    if !(POSTGRESQL_SECRET_KEY_MIN_LEN..=POSTGRESQL_SECRET_KEY_MAX_LEN).contains(&length) {
        return Err(PostgreSqlError::InvalidFieldLength {
            field,
            got: length as i32,
        });
    }

    Ok(())
}

/// Vérifie l'octet de type d'un message typé et retourne le
/// [`PostgreSqlMessageType`] correspondant.
pub fn extract_message_type(value: u8) -> Result<PostgreSqlMessageType, PostgreSqlError> {
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

/// Vérifie le cadrage d'un message typé (en-tête de 5 octets disponible,
/// champ length cohérent avec les octets restants) et retourne
/// `(length, consumed)`.
pub fn extract_typed_frame(remaining: &[u8]) -> Result<(u32, usize), PostgreSqlError> {
    // Garde locale : la fonction reste sûre même appelée hors du parseur,
    // qui a déjà validé l'en-tête pour préserver l'ordre des erreurs.
    validate_typed_header_available(remaining)?;

    let length = u32::from_be_bytes([remaining[1], remaining[2], remaining[3], remaining[4]]);
    let consumed = validate_typed_message_available(remaining, length)?;

    Ok((length, consumed))
}

/// Vérifie le cadrage d'un message sans octet de type (en-tête de 8 octets
/// disponible, champ length exactement égal à la taille du payload) et
/// retourne `(length, code, consumed)`.
pub fn extract_untyped_frame(payload: &[u8]) -> Result<(u32, u32, usize), PostgreSqlError> {
    validate_untyped_header_available(payload)?;

    let length = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let consumed = validate_untyped_message_available(payload, length)?;
    let code = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);

    Ok((length, code, consumed))
}

/// Vérifie le préfixe de longueur d'une valeur de paramètre Bind :
/// `-1` signale une valeur NULL (`None`), toute autre valeur négative est
/// rejetée, sinon la longueur est retournée en `usize`.
pub fn extract_parameter_value_length(len: i32) -> Result<Option<usize>, PostgreSqlError> {
    if len == -1 {
        return Ok(None);
    }

    if len < 0 {
        return Err(PostgreSqlError::InvalidFieldLength {
            field: "parameter_value_length",
            got: len,
        });
    }

    Ok(Some(len as usize))
}

/// Indique si le code startup correspond à une version supportée du
/// protocole PostgreSQL (3.0 ou 3.2).
pub(crate) fn postgresql_startup_protocol_version_is_supported(code: u32) -> bool {
    matches!(
        code,
        POSTGRESQL_PROTOCOL_VERSION_3_0 | POSTGRESQL_PROTOCOL_VERSION_3_2
    )
}

/// Vérifie que le code d'un message sans octet de type appartient à une
/// famille connue (StartupMessage, SSLRequest, CancelRequest, GSSENCRequest).
pub fn validate_startup_code(code: u32) -> Result<(), PostgreSqlError> {
    if postgresql_startup_protocol_version_is_supported(code)
        || matches!(
            code,
            POSTGRESQL_SSL_REQUEST_CODE
                | POSTGRESQL_CANCEL_REQUEST_CODE
                | POSTGRESQL_GSSENC_REQUEST_CODE
        )
    {
        return Ok(());
    }

    Err(PostgreSqlError::UnsupportedStartupCode(code))
}

/// Curseur de lecture séquentielle sur le corps d'un message : chaque `read_*`
/// vérifie les octets de SON champ (via `validate_remaining`) avant de les
/// consommer et de retourner la valeur typée.
pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    pub(crate) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub(crate) fn skip(&mut self, count: usize) -> Result<(), PostgreSqlError> {
        validate_remaining(self.remaining(), count, "skip")?;
        self.pos += count;
        Ok(())
    }

    pub(crate) fn read_u16(&mut self, field: &'static str) -> Result<u16, PostgreSqlError> {
        validate_remaining(self.remaining(), 2, field)?;
        let bytes = [self.bytes[self.pos], self.bytes[self.pos + 1]];
        self.pos += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    pub(crate) fn read_u32(&mut self, field: &'static str) -> Result<u32, PostgreSqlError> {
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

    pub(crate) fn read_i32(&mut self, field: &'static str) -> Result<i32, PostgreSqlError> {
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

    pub(crate) fn read_bytes(
        &mut self,
        count: usize,
        field: &'static str,
    ) -> Result<&'a [u8], PostgreSqlError> {
        validate_remaining(self.remaining(), count, field)?;
        let value = &self.bytes[self.pos..self.pos + count];
        self.pos += count;
        Ok(value)
    }

    pub(crate) fn read_cstring(&mut self, field: &'static str) -> Result<&'a str, PostgreSqlError> {
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

    #[test]
    fn extract_message_type_maps_known_bytes_and_rejects_unknown() {
        assert_eq!(extract_message_type(b'Q'), Ok(PostgreSqlMessageType::Query));
        assert_eq!(
            extract_message_type(b'R'),
            Ok(PostgreSqlMessageType::Authentication)
        );
        assert!(matches!(
            extract_message_type(b'@'),
            Err(PostgreSqlError::InvalidMessageType(0x40))
        ));
    }

    #[test]
    fn extract_typed_frame_returns_length_and_consumed() {
        let payload = [b'S', 0x00, 0x00, 0x00, 0x04];
        assert_eq!(extract_typed_frame(&payload), Ok((4, 5)));
    }

    #[test]
    fn extract_typed_frame_rejects_short_header() {
        assert!(matches!(
            extract_typed_frame(&[b'S', 0x00]),
            Err(PostgreSqlError::BufferTooSmall {
                needed: POSTGRESQL_TYPED_HEADER_LEN,
                actual: 2
            })
        ));
    }

    #[test]
    fn extract_typed_frame_rejects_truncated_message() {
        let payload = [b'Q', 0x00, 0x00, 0x00, 0x20, b'S'];
        assert!(matches!(
            extract_typed_frame(&payload),
            Err(PostgreSqlError::LengthMismatch {
                expected: 33,
                actual: 6
            })
        ));
    }

    #[test]
    fn extract_untyped_frame_returns_length_code_and_consumed() {
        let mut payload = 8u32.to_be_bytes().to_vec();
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3_0.to_be_bytes());
        assert_eq!(
            extract_untyped_frame(&payload),
            Ok((8, POSTGRESQL_PROTOCOL_VERSION_3_0, 8))
        );
    }

    #[test]
    fn extract_untyped_frame_rejects_length_mismatch() {
        let mut payload = 12u32.to_be_bytes().to_vec();
        payload.extend_from_slice(&POSTGRESQL_PROTOCOL_VERSION_3_0.to_be_bytes());
        assert!(matches!(
            extract_untyped_frame(&payload),
            Err(PostgreSqlError::LengthMismatch {
                expected: 12,
                actual: 8
            })
        ));
    }

    #[test]
    fn extract_untyped_frame_rejects_short_header() {
        assert!(matches!(
            extract_untyped_frame(&[0x00, 0x00, 0x00]),
            Err(PostgreSqlError::BufferTooSmall {
                needed: POSTGRESQL_UNTYPED_HEADER_LEN,
                actual: 3
            })
        ));
    }

    #[test]
    fn extract_parameter_value_length_handles_null_and_negative() {
        assert_eq!(extract_parameter_value_length(-1), Ok(None));
        assert_eq!(extract_parameter_value_length(0), Ok(Some(0)));
        assert_eq!(extract_parameter_value_length(3), Ok(Some(3)));
        assert!(matches!(
            extract_parameter_value_length(-2),
            Err(PostgreSqlError::InvalidFieldLength {
                field: "parameter_value_length",
                got: -2
            })
        ));
    }

    #[test]
    fn validate_startup_code_accepts_known_codes_only() {
        for code in [
            POSTGRESQL_PROTOCOL_VERSION_3_0,
            POSTGRESQL_PROTOCOL_VERSION_3_2,
            POSTGRESQL_SSL_REQUEST_CODE,
            POSTGRESQL_CANCEL_REQUEST_CODE,
            POSTGRESQL_GSSENC_REQUEST_CODE,
        ] {
            assert_eq!(validate_startup_code(code), Ok(()));
        }

        assert!(matches!(
            validate_startup_code(12_345),
            Err(PostgreSqlError::UnsupportedStartupCode(12_345))
        ));
    }

    #[test]
    fn cursor_read_cstring_reports_missing_terminator_and_invalid_utf8() {
        let mut cur = Cursor::new(b"abc");
        assert!(matches!(
            cur.read_cstring("field"),
            Err(PostgreSqlError::MissingNullTerminator { field: "field" })
        ));

        let bytes = [0xFF, 0xFE, 0x00];
        let mut cur = Cursor::new(&bytes);
        assert_eq!(cur.read_cstring("field"), Err(PostgreSqlError::InvalidUtf8));
    }
}
