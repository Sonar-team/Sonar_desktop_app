// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::postgresql::PostgreSqlError;

pub const POSTGRESQL_TYPED_HEADER_LEN: usize = 5;
pub const POSTGRESQL_UNTYPED_HEADER_LEN: usize = 8;
pub const POSTGRESQL_LENGTH_FIELD_LEN: usize = 4;

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
