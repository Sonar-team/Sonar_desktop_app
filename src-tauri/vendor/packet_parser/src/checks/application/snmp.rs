// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::{
    errors::application::snmp::SnmpError,
    parse::application::protocols::snmp::{SnmpPduType, SnmpVersion},
};

pub const ASN1_SEQUENCE_TAG: u8 = 0x30;
pub const ASN1_INTEGER_TAG: u8 = 0x02;
pub const ASN1_OCTET_STRING_TAG: u8 = 0x04;
pub const ASN1_NULL_TAG: u8 = 0x05;
pub const ASN1_OBJECT_IDENTIFIER_TAG: u8 = 0x06;

pub const SNMP_IP_ADDRESS_TAG: u8 = 0x40;
pub const SNMP_COUNTER32_TAG: u8 = 0x41;
pub const SNMP_GAUGE32_TAG: u8 = 0x42;
pub const SNMP_TIMETICKS_TAG: u8 = 0x43;
pub const SNMP_OPAQUE_TAG: u8 = 0x44;
pub const SNMP_COUNTER64_TAG: u8 = 0x46;
pub const SNMP_NO_SUCH_OBJECT_TAG: u8 = 0x80;
pub const SNMP_NO_SUCH_INSTANCE_TAG: u8 = 0x81;
pub const SNMP_END_OF_MIB_VIEW_TAG: u8 = 0x82;

pub const SNMP_MIN_PACKET_LEN: usize = 2;

pub fn validate_snmp_min_length(packet: &[u8]) -> Result<(), SnmpError> {
    if packet.len() < SNMP_MIN_PACKET_LEN {
        return Err(SnmpError::PacketTooShort {
            min: SNMP_MIN_PACKET_LEN,
            actual: packet.len(),
        });
    }

    Ok(())
}

pub fn ensure_available(
    field: &'static str,
    actual: usize,
    needed: usize,
) -> Result<(), SnmpError> {
    if actual < needed {
        return Err(SnmpError::Truncated {
            field,
            needed,
            actual,
        });
    }

    Ok(())
}

pub fn validate_tag(field: &'static str, actual: u8, expected: u8) -> Result<(), SnmpError> {
    if actual != expected {
        return Err(SnmpError::InvalidTag {
            field,
            expected,
            actual,
        });
    }

    Ok(())
}

pub fn validate_no_trailing(
    field: &'static str,
    consumed: usize,
    packet_len: usize,
) -> Result<(), SnmpError> {
    if consumed != packet_len {
        return Err(SnmpError::TrailingData {
            consumed,
            packet_len,
        });
    }

    let _ = field;
    Ok(())
}

pub fn validate_integer_length(field: &'static str, actual: usize) -> Result<(), SnmpError> {
    if actual == 0 || actual > 8 {
        return Err(SnmpError::InvalidIntegerLength { field, actual });
    }

    Ok(())
}

pub fn validate_unsigned_length(field: &'static str, actual: usize) -> Result<(), SnmpError> {
    if actual == 0 || actual > 8 {
        return Err(SnmpError::InvalidUnsignedLength { field, actual });
    }

    Ok(())
}

pub fn validate_version(version: i64) -> Result<SnmpVersion, SnmpError> {
    match version {
        0 => Ok(SnmpVersion::V1),
        1 => Ok(SnmpVersion::V2c),
        3 => Ok(SnmpVersion::V3),
        _ => Err(SnmpError::UnsupportedVersion { version }),
    }
}

pub fn validate_pdu_type(tag: u8, version: SnmpVersion) -> Result<SnmpPduType, SnmpError> {
    let pdu_type = match tag {
        0xA0 => SnmpPduType::GetRequest,
        0xA1 => SnmpPduType::GetNextRequest,
        0xA2 => SnmpPduType::Response,
        0xA3 => SnmpPduType::SetRequest,
        0xA4 => SnmpPduType::TrapV1,
        0xA5 => SnmpPduType::GetBulkRequest,
        0xA6 => SnmpPduType::InformRequest,
        0xA7 => SnmpPduType::TrapV2,
        0xA8 => SnmpPduType::Report,
        _ => return Err(SnmpError::UnsupportedPduType { tag, version }),
    };

    let supported = match version {
        SnmpVersion::V1 => matches!(
            pdu_type,
            SnmpPduType::GetRequest
                | SnmpPduType::GetNextRequest
                | SnmpPduType::Response
                | SnmpPduType::SetRequest
                | SnmpPduType::TrapV1
        ),
        SnmpVersion::V2c => matches!(
            pdu_type,
            SnmpPduType::GetRequest
                | SnmpPduType::GetNextRequest
                | SnmpPduType::Response
                | SnmpPduType::SetRequest
                | SnmpPduType::GetBulkRequest
                | SnmpPduType::InformRequest
                | SnmpPduType::TrapV2
        ),
        SnmpVersion::V3 => matches!(
            pdu_type,
            SnmpPduType::GetRequest
                | SnmpPduType::GetNextRequest
                | SnmpPduType::Response
                | SnmpPduType::SetRequest
                | SnmpPduType::GetBulkRequest
                | SnmpPduType::InformRequest
                | SnmpPduType::TrapV2
                | SnmpPduType::Report
        ),
    };

    if supported {
        Ok(pdu_type)
    } else {
        Err(SnmpError::UnsupportedPduType { tag, version })
    }
}
