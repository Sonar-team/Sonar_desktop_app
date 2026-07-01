// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::internet::profinet::ProfinetPacketError, parse::internet::protocols::profinet::FrameId,
};

pub fn validate_packet_length(data: &[u8]) -> Result<(), ProfinetPacketError> {
    if data.len() < 16 {
        Err(ProfinetPacketError::PacketTooShort(data.len()))
    } else {
        Ok(())
    }
}

pub fn validate_frame_id(data: &[u8]) -> Result<FrameId, ProfinetPacketError> {
    let frame_id_value = u16::from_be_bytes([data[0], data[1]]);
    FrameId::from_u16(frame_id_value).ok_or(ProfinetPacketError::UnknownFrameId(frame_id_value))
}

pub fn validate_dcp_block(data: &[u8]) -> Result<(), ProfinetPacketError> {
    if data.len() < 16 {
        return Err(ProfinetPacketError::PacketTooShort(data.len()));
    }

    let block = &data[12..];
    if block.len() < 4 {
        return Err(ProfinetPacketError::InvalidDcpBlockLength(block.len()));
    }

    Ok(())
}

pub fn extract_name_of_station(data: &[u8]) -> Result<&str, ProfinetPacketError> {
    let block = &data[12..];
    let dcp_block_length = u16::from_be_bytes([block[2], block[3]]) as usize;

    if block.len() < (4 + dcp_block_length) {
        return Err(ProfinetPacketError::InvalidDcpBlockLength(block.len()));
    }

    std::str::from_utf8(&block[4..4 + dcp_block_length])
        .map_err(|_| ProfinetPacketError::InvalidNameOfStation)
}
