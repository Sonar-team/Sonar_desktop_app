// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::srvloc::SrvlocPacketParseError;

pub fn validate_packet_not_empty(payload: &[u8]) -> Result<(), SrvlocPacketParseError> {
    if payload.is_empty() {
        return Err(SrvlocPacketParseError::InvalidPacketLength);
    }

    Ok(())
}

pub fn ensure_len(buf: &[u8], needed: usize) -> Result<(), SrvlocPacketParseError> {
    if buf.len() < needed {
        Err(SrvlocPacketParseError::Truncated {
            expected_at_least: needed,
            actual: buf.len(),
        })
    } else {
        Ok(())
    }
}
