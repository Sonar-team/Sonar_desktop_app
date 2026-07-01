// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::giop::GiopParseError;

pub const GIOP_HEADER_LEN: usize = 12;

pub fn ensure_min_len(payload: &[u8]) -> Result<(), GiopParseError> {
    if payload.len() < GIOP_HEADER_LEN {
        return Err(GiopParseError::InvalidSize);
    }

    Ok(())
}

pub fn parse_magic(payload: &[u8]) -> Result<[u8; 4], GiopParseError> {
    let magic: [u8; 4] = payload[0..4].try_into().unwrap();
    if &magic != b"GIOP" {
        return Err(GiopParseError::InvalidMagic);
    }

    Ok(magic)
}

pub fn validate_version(major_version: u8, minor_version: u8) -> Result<(), GiopParseError> {
    if major_version != 1 || minor_version > 2 {
        return Err(GiopParseError::UnsupportedVersion(
            major_version,
            minor_version,
        ));
    }

    Ok(())
}

pub fn validate_total_length(total_needed: usize, actual: usize) -> Result<(), GiopParseError> {
    if actual < total_needed {
        return Err(GiopParseError::TruncatedBody {
            expected: total_needed,
            actual,
        });
    }

    Ok(())
}
