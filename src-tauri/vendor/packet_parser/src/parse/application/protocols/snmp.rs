// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::{convert::TryFrom, mem::size_of};

use crate::{
    checks::application::snmp::{
        ASN1_INTEGER_TAG, ASN1_NULL_TAG, ASN1_OBJECT_IDENTIFIER_TAG, ASN1_OCTET_STRING_TAG,
        ASN1_SEQUENCE_TAG, SNMP_COUNTER32_TAG, SNMP_COUNTER64_TAG, SNMP_END_OF_MIB_VIEW_TAG,
        SNMP_GAUGE32_TAG, SNMP_IP_ADDRESS_TAG, SNMP_NO_SUCH_INSTANCE_TAG, SNMP_NO_SUCH_OBJECT_TAG,
        SNMP_OPAQUE_TAG, SNMP_TIMETICKS_TAG, ensure_available, validate_integer_length,
        validate_no_trailing, validate_pdu_type, validate_snmp_min_length, validate_tag,
        validate_unsigned_length, validate_version,
    },
    errors::application::snmp::SnmpError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// SNMP Message
///
/// ```mermaid
/// ---
/// title: SnmpPacket
/// ---
/// packet-beta
/// 0-7: "SEQUENCE tag"
/// 8-31: "Message length BER"
/// 32-55: "Version INTEGER"
/// 56-95: "Community or v3 HeaderData"
/// 96-127: "PDU / SecurityParameters"
/// 128-191: "VarBindList / ScopedPDU variable"
/// ```
#[derive(Debug)]
pub struct SnmpPacket<'a> {
    pub version: SnmpVersion,
    pub message: SnmpMessage<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnmpVersion {
    V1,
    V2c,
    V3,
}

#[derive(Debug)]
pub enum SnmpMessage<'a> {
    V1V2c(SnmpV1V2cMessage<'a>),
    V3(SnmpV3Message<'a>),
}

#[derive(Debug)]
pub struct SnmpV1V2cMessage<'a> {
    pub community: &'a [u8],
    pub pdu: SnmpPdu<'a>,
}

#[derive(Debug)]
pub struct SnmpV3Message<'a> {
    pub message_id: i64,
    pub max_size: i64,
    pub flags: &'a [u8],
    pub security_model: i64,
    pub security_parameters: &'a [u8],
    pub data: SnmpV3Data<'a>,
}

#[derive(Debug)]
pub enum SnmpV3Data<'a> {
    ScopedPdu(SnmpScopedPdu<'a>),
    EncryptedPdu(&'a [u8]),
}

#[derive(Debug)]
pub struct SnmpScopedPdu<'a> {
    pub context_engine_id: &'a [u8],
    pub context_name: &'a [u8],
    pub pdu: SnmpPdu<'a>,
}

#[derive(Debug)]
pub struct SnmpPdu<'a> {
    pub pdu_type: SnmpPduType,
    pub raw: &'a [u8],
    pub payload: SnmpPduPayload<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnmpPduType {
    GetRequest,
    GetNextRequest,
    Response,
    SetRequest,
    TrapV1,
    GetBulkRequest,
    InformRequest,
    TrapV2,
    Report,
}

#[derive(Debug)]
pub enum SnmpPduPayload<'a> {
    Standard {
        request_id: i64,
        error_status: i64,
        error_index: i64,
        variable_bindings: Vec<SnmpVarBind<'a>>,
    },
    TrapV1 {
        enterprise: &'a [u8],
        agent_address: [u8; 4],
        generic_trap: i64,
        specific_trap: i64,
        timestamp: u64,
        variable_bindings: Vec<SnmpVarBind<'a>>,
    },
}

#[derive(Debug)]
pub struct SnmpVarBind<'a> {
    pub oid: &'a [u8],
    pub value: SnmpValue<'a>,
}

#[derive(Debug)]
pub enum SnmpValue<'a> {
    Integer(i64),
    OctetString(&'a [u8]),
    Null,
    ObjectIdentifier(&'a [u8]),
    IpAddress([u8; 4]),
    Counter32(u32),
    Gauge32(u32),
    TimeTicks(u32),
    Opaque(&'a [u8]),
    Counter64(u64),
    NoSuchObject,
    NoSuchInstance,
    EndOfMibView,
    Unsupported { tag: u8, data: &'a [u8] },
}

struct BerTlv<'a> {
    tag: u8,
    value: &'a [u8],
    encoded: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for SnmpPacket<'a> {
    type Error = SnmpError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        validate_snmp_min_length(packet)?;

        let mut offset = 0usize;
        let message = read_tlv(packet, &mut offset, "message")?;
        validate_tag("message", message.tag, ASN1_SEQUENCE_TAG)?;
        validate_no_trailing("message", offset, packet.len())?;

        let mut message_offset = 0usize;
        let version = read_integer_field(message.value, &mut message_offset, "version")?;
        let version = validate_version(version)?;

        let parsed_message = match version {
            SnmpVersion::V1 | SnmpVersion::V2c => {
                parse_v1_v2c_message(message.value, &mut message_offset, version)?
            }
            SnmpVersion::V3 => parse_v3_message(message.value, &mut message_offset)?,
        };
        validate_no_trailing("message body", message_offset, message.value.len())?;

        Ok(SnmpPacket {
            version,
            message: parsed_message,
        })
    }
}

fn parse_v1_v2c_message<'a>(
    body: &'a [u8],
    offset: &mut usize,
    version: SnmpVersion,
) -> Result<SnmpMessage<'a>, SnmpError> {
    let community = read_tlv(body, offset, "community")?;
    validate_tag("community", community.tag, ASN1_OCTET_STRING_TAG)?;

    let pdu_tlv = read_tlv(body, offset, "pdu")?;
    let pdu = parse_pdu(pdu_tlv, version)?;

    Ok(SnmpMessage::V1V2c(SnmpV1V2cMessage {
        community: community.value,
        pdu,
    }))
}

fn parse_v3_message<'a>(body: &'a [u8], offset: &mut usize) -> Result<SnmpMessage<'a>, SnmpError> {
    let header = read_tlv(body, offset, "v3_header_data")?;
    validate_tag("v3_header_data", header.tag, ASN1_SEQUENCE_TAG)?;
    let (message_id, max_size, flags, security_model) = parse_v3_header_data(header.value)?;

    let security_parameters = read_tlv(body, offset, "v3_security_parameters")?;
    validate_tag(
        "v3_security_parameters",
        security_parameters.tag,
        ASN1_OCTET_STRING_TAG,
    )?;

    let data = read_tlv(body, offset, "v3_data")?;
    let data = match data.tag {
        ASN1_SEQUENCE_TAG => SnmpV3Data::ScopedPdu(parse_scoped_pdu(data.value)?),
        ASN1_OCTET_STRING_TAG => SnmpV3Data::EncryptedPdu(data.value),
        tag => {
            return Err(SnmpError::InvalidTag {
                field: "v3_data",
                expected: ASN1_SEQUENCE_TAG,
                actual: tag,
            });
        }
    };

    Ok(SnmpMessage::V3(SnmpV3Message {
        message_id,
        max_size,
        flags,
        security_model,
        security_parameters: security_parameters.value,
        data,
    }))
}

fn parse_v3_header_data(header: &[u8]) -> Result<(i64, i64, &[u8], i64), SnmpError> {
    let mut offset = 0usize;
    let message_id = read_integer_field(header, &mut offset, "v3_message_id")?;
    let max_size = read_integer_field(header, &mut offset, "v3_max_size")?;

    let flags = read_tlv(header, &mut offset, "v3_flags")?;
    validate_tag("v3_flags", flags.tag, ASN1_OCTET_STRING_TAG)?;

    let security_model = read_integer_field(header, &mut offset, "v3_security_model")?;
    validate_no_trailing("v3_header_data", offset, header.len())?;

    Ok((message_id, max_size, flags.value, security_model))
}

fn parse_scoped_pdu<'a>(value: &'a [u8]) -> Result<SnmpScopedPdu<'a>, SnmpError> {
    let mut offset = 0usize;

    let context_engine_id = read_tlv(value, &mut offset, "v3_context_engine_id")?;
    validate_tag(
        "v3_context_engine_id",
        context_engine_id.tag,
        ASN1_OCTET_STRING_TAG,
    )?;

    let context_name = read_tlv(value, &mut offset, "v3_context_name")?;
    validate_tag("v3_context_name", context_name.tag, ASN1_OCTET_STRING_TAG)?;

    let pdu = read_tlv(value, &mut offset, "v3_scoped_pdu")?;
    let pdu = parse_pdu(pdu, SnmpVersion::V3)?;
    validate_no_trailing("v3_scoped_pdu", offset, value.len())?;

    Ok(SnmpScopedPdu {
        context_engine_id: context_engine_id.value,
        context_name: context_name.value,
        pdu,
    })
}

fn parse_pdu<'a>(tlv: BerTlv<'a>, version: SnmpVersion) -> Result<SnmpPdu<'a>, SnmpError> {
    let pdu_type = validate_pdu_type(tlv.tag, version)?;
    let payload = if pdu_type == SnmpPduType::TrapV1 {
        parse_trap_v1_pdu(tlv.value)?
    } else {
        parse_standard_pdu(tlv.value)?
    };

    Ok(SnmpPdu {
        pdu_type,
        raw: tlv.encoded,
        payload,
    })
}

fn parse_standard_pdu<'a>(value: &'a [u8]) -> Result<SnmpPduPayload<'a>, SnmpError> {
    let mut offset = 0usize;
    let request_id = read_integer_field(value, &mut offset, "request_id")?;
    let error_status = read_integer_field(value, &mut offset, "error_status")?;
    let error_index = read_integer_field(value, &mut offset, "error_index")?;

    let varbind_list = read_tlv(value, &mut offset, "variable_bindings")?;
    validate_tag("variable_bindings", varbind_list.tag, ASN1_SEQUENCE_TAG)?;
    let variable_bindings = parse_variable_bindings(varbind_list.value)?;
    validate_no_trailing("standard_pdu", offset, value.len())?;

    Ok(SnmpPduPayload::Standard {
        request_id,
        error_status,
        error_index,
        variable_bindings,
    })
}

fn parse_trap_v1_pdu<'a>(value: &'a [u8]) -> Result<SnmpPduPayload<'a>, SnmpError> {
    let mut offset = 0usize;

    let enterprise = read_tlv(value, &mut offset, "trap_enterprise")?;
    validate_tag(
        "trap_enterprise",
        enterprise.tag,
        ASN1_OBJECT_IDENTIFIER_TAG,
    )?;

    let agent_address = read_tlv(value, &mut offset, "trap_agent_address")?;
    validate_tag("trap_agent_address", agent_address.tag, SNMP_IP_ADDRESS_TAG)?;
    if agent_address.value.len() != 4 {
        return Err(SnmpError::InvalidIpAddressLength {
            actual: agent_address.value.len(),
        });
    }
    let agent_address = [
        agent_address.value[0],
        agent_address.value[1],
        agent_address.value[2],
        agent_address.value[3],
    ];

    let generic_trap = read_integer_field(value, &mut offset, "generic_trap")?;
    let specific_trap = read_integer_field(value, &mut offset, "specific_trap")?;

    let timestamp = read_tlv(value, &mut offset, "trap_timestamp")?;
    validate_tag("trap_timestamp", timestamp.tag, SNMP_TIMETICKS_TAG)?;
    let timestamp = parse_unsigned_value(timestamp.value, "trap_timestamp")?;

    let varbind_list = read_tlv(value, &mut offset, "variable_bindings")?;
    validate_tag("variable_bindings", varbind_list.tag, ASN1_SEQUENCE_TAG)?;
    let variable_bindings = parse_variable_bindings(varbind_list.value)?;
    validate_no_trailing("trap_v1_pdu", offset, value.len())?;

    Ok(SnmpPduPayload::TrapV1 {
        enterprise: enterprise.value,
        agent_address,
        generic_trap,
        specific_trap,
        timestamp,
        variable_bindings,
    })
}

fn parse_variable_bindings<'a>(value: &'a [u8]) -> Result<Vec<SnmpVarBind<'a>>, SnmpError> {
    let mut offset = 0usize;
    let mut variable_bindings = Vec::new();

    while offset < value.len() {
        let varbind = read_tlv(value, &mut offset, "varbind")?;
        validate_tag("varbind", varbind.tag, ASN1_SEQUENCE_TAG)?;

        let mut varbind_offset = 0usize;
        let oid = read_tlv(varbind.value, &mut varbind_offset, "varbind_oid")?;
        validate_tag("varbind_oid", oid.tag, ASN1_OBJECT_IDENTIFIER_TAG)?;

        let value_tlv = read_tlv(varbind.value, &mut varbind_offset, "varbind_value")?;
        let parsed_value = parse_snmp_value(value_tlv)?;
        validate_no_trailing("varbind", varbind_offset, varbind.value.len())?;

        variable_bindings.push(SnmpVarBind {
            oid: oid.value,
            value: parsed_value,
        });
    }

    Ok(variable_bindings)
}

fn parse_snmp_value<'a>(tlv: BerTlv<'a>) -> Result<SnmpValue<'a>, SnmpError> {
    match tlv.tag {
        ASN1_INTEGER_TAG => Ok(SnmpValue::Integer(parse_integer_value(
            tlv.value,
            "value_integer",
        )?)),
        ASN1_OCTET_STRING_TAG => Ok(SnmpValue::OctetString(tlv.value)),
        ASN1_NULL_TAG => {
            if !tlv.value.is_empty() {
                return Err(SnmpError::InvalidPduStructure("NULL value is not empty"));
            }
            Ok(SnmpValue::Null)
        }
        ASN1_OBJECT_IDENTIFIER_TAG => Ok(SnmpValue::ObjectIdentifier(tlv.value)),
        SNMP_IP_ADDRESS_TAG => {
            if tlv.value.len() != 4 {
                return Err(SnmpError::InvalidIpAddressLength {
                    actual: tlv.value.len(),
                });
            }
            Ok(SnmpValue::IpAddress([
                tlv.value[0],
                tlv.value[1],
                tlv.value[2],
                tlv.value[3],
            ]))
        }
        SNMP_COUNTER32_TAG => Ok(SnmpValue::Counter32(parse_u32_value(
            tlv.value,
            "counter32",
        )?)),
        SNMP_GAUGE32_TAG => Ok(SnmpValue::Gauge32(parse_u32_value(tlv.value, "gauge32")?)),
        SNMP_TIMETICKS_TAG => Ok(SnmpValue::TimeTicks(parse_u32_value(
            tlv.value,
            "timeticks",
        )?)),
        SNMP_OPAQUE_TAG => Ok(SnmpValue::Opaque(tlv.value)),
        SNMP_COUNTER64_TAG => Ok(SnmpValue::Counter64(parse_unsigned_value(
            tlv.value,
            "counter64",
        )?)),
        SNMP_NO_SUCH_OBJECT_TAG => {
            validate_exception_empty(tlv.value, "no_such_object")?;
            Ok(SnmpValue::NoSuchObject)
        }
        SNMP_NO_SUCH_INSTANCE_TAG => {
            validate_exception_empty(tlv.value, "no_such_instance")?;
            Ok(SnmpValue::NoSuchInstance)
        }
        SNMP_END_OF_MIB_VIEW_TAG => {
            validate_exception_empty(tlv.value, "end_of_mib_view")?;
            Ok(SnmpValue::EndOfMibView)
        }
        tag => Ok(SnmpValue::Unsupported {
            tag,
            data: tlv.value,
        }),
    }
}

fn validate_exception_empty(value: &[u8], field: &'static str) -> Result<(), SnmpError> {
    if !value.is_empty() {
        return Err(SnmpError::InvalidPduStructure(field));
    }

    Ok(())
}

fn read_integer_field(
    data: &[u8],
    offset: &mut usize,
    field: &'static str,
) -> Result<i64, SnmpError> {
    let tlv = read_tlv(data, offset, field)?;
    validate_tag(field, tlv.tag, ASN1_INTEGER_TAG)?;
    parse_integer_value(tlv.value, field)
}

fn parse_integer_value(value: &[u8], field: &'static str) -> Result<i64, SnmpError> {
    validate_integer_length(field, value.len())?;

    let negative = value[0] & 0x80 != 0;
    let mut parsed = if negative { -1i64 } else { 0i64 };
    for &byte in value {
        parsed = (parsed << 8) | i64::from(byte);
    }

    Ok(parsed)
}

fn parse_u32_value(value: &[u8], field: &'static str) -> Result<u32, SnmpError> {
    let parsed = parse_unsigned_value(value, field)?;
    u32::try_from(parsed).map_err(|_| SnmpError::UnsignedOverflow { field })
}

fn parse_unsigned_value(value: &[u8], field: &'static str) -> Result<u64, SnmpError> {
    validate_unsigned_length(field, value.len())?;

    let mut parsed = 0u64;
    for &byte in value {
        parsed = (parsed << 8) | u64::from(byte);
    }

    Ok(parsed)
}

fn read_tlv<'a>(
    data: &'a [u8],
    offset: &mut usize,
    field: &'static str,
) -> Result<BerTlv<'a>, SnmpError> {
    let start = *offset;
    let header_needed = start
        .checked_add(2)
        .ok_or(SnmpError::LengthOverflow { field })?;
    ensure_available(field, data.len(), header_needed)?;

    let tag = data[start];
    let first_len = data[start + 1];
    *offset = header_needed;

    let length = if first_len & 0x80 == 0 {
        usize::from(first_len)
    } else {
        let len_len = usize::from(first_len & 0x7F);
        if len_len == 0 {
            return Err(SnmpError::UnsupportedIndefiniteLength { field });
        }
        if len_len > size_of::<usize>() {
            return Err(SnmpError::UnsupportedLengthSize {
                field,
                actual: len_len,
            });
        }

        let length_end = (*offset)
            .checked_add(len_len)
            .ok_or(SnmpError::LengthOverflow { field })?;
        ensure_available(field, data.len(), length_end)?;

        let mut length = 0usize;
        for &byte in &data[*offset..length_end] {
            length = length
                .checked_mul(256)
                .and_then(|value| value.checked_add(usize::from(byte)))
                .ok_or(SnmpError::LengthOverflow { field })?;
        }
        *offset = length_end;
        length
    };

    let value_start = *offset;
    let value_end = value_start
        .checked_add(length)
        .ok_or(SnmpError::LengthOverflow { field })?;
    ensure_available(field, data.len(), value_end)?;
    *offset = value_end;

    Ok(BerTlv {
        tag,
        value: &data[value_start..value_end],
        encoded: &data[start..value_end],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_v2c_get_request() {
        let bytes = hex::decode(
            "302302010104067075626c6963a016020101020100020100300b300906052b060102010500",
        )
        .expect("invalid hex fixture");

        let packet = SnmpPacket::try_from(bytes.as_slice()).expect("valid SNMP v2c packet");

        assert_eq!(packet.version, SnmpVersion::V2c);
        let SnmpMessage::V1V2c(message) = packet.message else {
            panic!("expected v1/v2c message");
        };
        assert_eq!(message.community, b"public");
        assert_eq!(message.pdu.pdu_type, SnmpPduType::GetRequest);

        let SnmpPduPayload::Standard {
            request_id,
            error_status,
            error_index,
            variable_bindings,
        } = message.pdu.payload
        else {
            panic!("expected standard PDU");
        };

        assert_eq!(request_id, 1);
        assert_eq!(error_status, 0);
        assert_eq!(error_index, 0);
        assert_eq!(variable_bindings.len(), 1);
        assert_eq!(variable_bindings[0].oid, &[0x2B, 0x06, 0x01, 0x02, 0x01]);
        assert!(matches!(variable_bindings[0].value, SnmpValue::Null));
    }

    #[test]
    fn parse_v3_scoped_response() {
        let bytes = hex::decode(
            "3027020103300d020101020205dc0401000201030400301104000400a20b0201010201000201003000",
        )
        .expect("invalid hex fixture");

        let packet = SnmpPacket::try_from(bytes.as_slice()).expect("valid SNMP v3 packet");

        assert_eq!(packet.version, SnmpVersion::V3);
        let SnmpMessage::V3(message) = packet.message else {
            panic!("expected v3 message");
        };
        assert_eq!(message.message_id, 1);
        assert_eq!(message.max_size, 1500);
        assert_eq!(message.flags, &[0]);
        assert_eq!(message.security_model, 3);
        assert!(message.security_parameters.is_empty());

        let SnmpV3Data::ScopedPdu(scoped_pdu) = message.data else {
            panic!("expected scoped PDU");
        };
        assert!(scoped_pdu.context_engine_id.is_empty());
        assert!(scoped_pdu.context_name.is_empty());
        assert_eq!(scoped_pdu.pdu.pdu_type, SnmpPduType::Response);
    }

    #[test]
    fn reject_packet_too_short() {
        let err = SnmpPacket::try_from(&[0x30][..]).unwrap_err();

        assert_eq!(err, SnmpError::PacketTooShort { min: 2, actual: 1 });
    }

    #[test]
    fn reject_invalid_top_level_tag() {
        let err = SnmpPacket::try_from(&[0x31, 0x00][..]).unwrap_err();

        assert_eq!(
            err,
            SnmpError::InvalidTag {
                field: "message",
                expected: ASN1_SEQUENCE_TAG,
                actual: 0x31
            }
        );
    }

    #[test]
    fn reject_unsupported_version() {
        let err = SnmpPacket::try_from(&[0x30, 0x03, 0x02, 0x01, 0x02][..]).unwrap_err();

        assert_eq!(err, SnmpError::UnsupportedVersion { version: 2 });
    }

    #[test]
    fn reject_v1_get_bulk() {
        let err = SnmpPacket::try_from(&[0x30, 0x07, 0x02, 0x01, 0x00, 0x04, 0x00, 0xA5, 0x00][..])
            .unwrap_err();

        assert_eq!(
            err,
            SnmpError::UnsupportedPduType {
                tag: 0xA5,
                version: SnmpVersion::V1
            }
        );
    }
}
