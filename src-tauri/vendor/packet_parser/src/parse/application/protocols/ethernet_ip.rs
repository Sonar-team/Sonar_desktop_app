// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::convert::TryFrom;

use crate::{
    checks::application::ethernet_ip::{
        ENCAPSULATION_HEADER_LEN, ensure_available, validate_common_packet_format_min_length,
        validate_cpf_consumed, validate_declared_length, validate_empty_command_data,
        validate_interface_handle, validate_min_length, validate_options,
        validate_protocol_version, validate_register_options_flags,
        validate_register_session_length,
    },
    errors::application::ethernet_ip::EtherNetIpError,
};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// EtherNet/IP Encapsulation Packet
///
/// ```mermaid
/// ---
/// title: EtherNetIpPacket
/// ---
/// packet-beta
/// 0-15: "Command u16 LE"
/// 16-31: "Length u16 LE"
/// 32-63: "Session Handle u32 LE"
/// 64-95: "Status u32 LE"
/// 96-159: "Sender Context bytes[8]"
/// 160-191: "Options u32 LE"
/// 192-255: "Command Data variable"
/// ```
#[derive(Debug)]
pub struct EtherNetIpPacket<'a> {
    pub header: EtherNetIpHeader<'a>,
    pub command_data: EtherNetIpCommandData<'a>,
}

#[derive(Debug)]
pub struct EtherNetIpHeader<'a> {
    pub command: EtherNetIpCommand,
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub sender_context: &'a [u8],
    pub options: u32,
}

#[derive(Debug)]
pub enum EtherNetIpCommandData<'a> {
    Empty,
    RegisterSession {
        protocol_version: u16,
        options_flags: u16,
    },
    CommonPacketFormat(EtherNetIpCommonPacketFormat<'a>),
    Raw(&'a [u8]),
}

#[derive(Debug)]
pub struct EtherNetIpCommonPacketFormat<'a> {
    pub interface_handle: u32,
    pub timeout: u16,
    pub items: Vec<EtherNetIpCpfItem<'a>>,
}

#[derive(Debug)]
pub struct EtherNetIpCpfItem<'a> {
    pub type_id: u16,
    pub data: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherNetIpCommand {
    Nop,
    ListServices,
    ListIdentity,
    ListInterfaces,
    RegisterSession,
    UnRegisterSession,
    SendRrData,
    SendUnitData,
    IndicateStatus,
    Cancel,
}

impl EtherNetIpCommand {
    pub fn name(self) -> &'static str {
        match self {
            EtherNetIpCommand::Nop => "NOP",
            EtherNetIpCommand::ListServices => "ListServices",
            EtherNetIpCommand::ListIdentity => "ListIdentity",
            EtherNetIpCommand::ListInterfaces => "ListInterfaces",
            EtherNetIpCommand::RegisterSession => "RegisterSession",
            EtherNetIpCommand::UnRegisterSession => "UnRegisterSession",
            EtherNetIpCommand::SendRrData => "SendRRData",
            EtherNetIpCommand::SendUnitData => "SendUnitData",
            EtherNetIpCommand::IndicateStatus => "IndicateStatus",
            EtherNetIpCommand::Cancel => "Cancel",
        }
    }
}

impl TryFrom<u16> for EtherNetIpCommand {
    type Error = EtherNetIpError;

    fn try_from(command: u16) -> Result<Self, Self::Error> {
        match command {
            0x0000 => Ok(EtherNetIpCommand::Nop),
            0x0004 => Ok(EtherNetIpCommand::ListServices),
            0x0063 => Ok(EtherNetIpCommand::ListIdentity),
            0x0064 => Ok(EtherNetIpCommand::ListInterfaces),
            0x0065 => Ok(EtherNetIpCommand::RegisterSession),
            0x0066 => Ok(EtherNetIpCommand::UnRegisterSession),
            0x006F => Ok(EtherNetIpCommand::SendRrData),
            0x0070 => Ok(EtherNetIpCommand::SendUnitData),
            0x0072 => Ok(EtherNetIpCommand::IndicateStatus),
            0x0073 => Ok(EtherNetIpCommand::Cancel),
            _ => Err(EtherNetIpError::UnknownCommand { command }),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for EtherNetIpPacket<'a> {
    type Error = EtherNetIpError;

    fn try_from(packet: &'a [u8]) -> Result<Self, Self::Error> {
        validate_min_length(packet)?;

        let command = u16::from_le_bytes([packet[0], packet[1]]);
        let command = EtherNetIpCommand::try_from(command)?;
        let length = u16::from_le_bytes([packet[2], packet[3]]);
        validate_declared_length(length, packet.len())?;

        let session_handle = u32::from_le_bytes([packet[4], packet[5], packet[6], packet[7]]);
        let status = u32::from_le_bytes([packet[8], packet[9], packet[10], packet[11]]);
        let sender_context = &packet[12..20];
        let options = u32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]);
        validate_options(options)?;

        let data = &packet[ENCAPSULATION_HEADER_LEN..];
        let command_data = parse_command_data(command, data)?;

        Ok(EtherNetIpPacket {
            header: EtherNetIpHeader {
                command,
                length,
                session_handle,
                status,
                sender_context,
                options,
            },
            command_data,
        })
    }
}

fn parse_command_data<'a>(
    command: EtherNetIpCommand,
    data: &'a [u8],
) -> Result<EtherNetIpCommandData<'a>, EtherNetIpError> {
    match command {
        EtherNetIpCommand::RegisterSession => parse_register_session(data),
        EtherNetIpCommand::UnRegisterSession => {
            validate_empty_command_data(command, data)?;
            Ok(EtherNetIpCommandData::Empty)
        }
        EtherNetIpCommand::SendRrData | EtherNetIpCommand::SendUnitData => {
            parse_common_packet_format(data).map(EtherNetIpCommandData::CommonPacketFormat)
        }
        EtherNetIpCommand::Nop
        | EtherNetIpCommand::ListServices
        | EtherNetIpCommand::ListIdentity
        | EtherNetIpCommand::ListInterfaces
        | EtherNetIpCommand::IndicateStatus
        | EtherNetIpCommand::Cancel => {
            if data.is_empty() {
                Ok(EtherNetIpCommandData::Empty)
            } else {
                Ok(EtherNetIpCommandData::Raw(data))
            }
        }
    }
}

fn parse_register_session<'a>(
    data: &'a [u8],
) -> Result<EtherNetIpCommandData<'a>, EtherNetIpError> {
    validate_register_session_length(data)?;

    let protocol_version = u16::from_le_bytes([data[0], data[1]]);
    validate_protocol_version(protocol_version)?;

    let options_flags = u16::from_le_bytes([data[2], data[3]]);
    validate_register_options_flags(options_flags)?;

    Ok(EtherNetIpCommandData::RegisterSession {
        protocol_version,
        options_flags,
    })
}

fn parse_common_packet_format<'a>(
    data: &'a [u8],
) -> Result<EtherNetIpCommonPacketFormat<'a>, EtherNetIpError> {
    validate_common_packet_format_min_length(data)?;

    let interface_handle = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    validate_interface_handle(interface_handle)?;

    let timeout = u16::from_le_bytes([data[4], data[5]]);
    let item_count = u16::from_le_bytes([data[6], data[7]]) as usize;

    let mut offset = 8usize;
    let mut items = Vec::with_capacity(item_count);

    for _ in 0..item_count {
        let item_header_end = offset + 4;
        ensure_available("cpf_item_header", data.len(), item_header_end)?;

        let type_id = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let length = u16::from_le_bytes([data[offset + 2], data[offset + 3]]) as usize;
        offset = item_header_end;

        let item_end = offset + length;
        ensure_available("cpf_item_data", data.len(), item_end)?;

        items.push(EtherNetIpCpfItem {
            type_id,
            data: &data[offset..item_end],
        });
        offset = item_end;
    }

    validate_cpf_consumed(offset, data.len())?;

    Ok(EtherNetIpCommonPacketFormat {
        interface_handle,
        timeout,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_register_session_request() {
        let bytes = hex::decode("65000400000000000000000001020304050607080000000001000000")
            .expect("invalid hex fixture");

        let packet =
            EtherNetIpPacket::try_from(bytes.as_slice()).expect("valid RegisterSession packet");

        assert_eq!(packet.header.command, EtherNetIpCommand::RegisterSession);
        assert_eq!(packet.header.length, 4);
        assert_eq!(packet.header.session_handle, 0);
        assert_eq!(packet.header.status, 0);
        assert_eq!(packet.header.sender_context, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(packet.header.options, 0);

        let EtherNetIpCommandData::RegisterSession {
            protocol_version,
            options_flags,
        } = packet.command_data
        else {
            panic!("expected RegisterSession command data");
        };

        assert_eq!(protocol_version, 1);
        assert_eq!(options_flags, 0);
    }

    #[test]
    fn parse_send_rr_data_common_packet_format() {
        let bytes = hex::decode(
            "6f0014007856341200000000000000000000000000000000000000000000020000000000b20004004c022001",
        )
        .expect("invalid hex fixture");

        let packet = EtherNetIpPacket::try_from(bytes.as_slice()).expect("valid SendRRData packet");

        assert_eq!(packet.header.command, EtherNetIpCommand::SendRrData);
        assert_eq!(packet.header.length, 20);
        assert_eq!(packet.header.session_handle, 0x12345678);

        let EtherNetIpCommandData::CommonPacketFormat(cpf) = packet.command_data else {
            panic!("expected common packet format");
        };

        assert_eq!(cpf.interface_handle, 0);
        assert_eq!(cpf.timeout, 0);
        assert_eq!(cpf.items.len(), 2);
        assert_eq!(cpf.items[0].type_id, 0x0000);
        assert!(cpf.items[0].data.is_empty());
        assert_eq!(cpf.items[1].type_id, 0x00B2);
        assert_eq!(cpf.items[1].data, &[0x4C, 0x02, 0x20, 0x01]);
    }

    #[test]
    fn reject_packet_too_short() {
        let err = EtherNetIpPacket::try_from(&[0u8; 23][..]).unwrap_err();

        assert_eq!(
            err,
            EtherNetIpError::PacketTooShort {
                expected: ENCAPSULATION_HEADER_LEN,
                actual: 23
            }
        );
    }

    #[test]
    fn reject_unknown_command() {
        let bytes = hex::decode("990000000000000000000000000000000000000000000000")
            .expect("invalid hex fixture");

        let err = EtherNetIpPacket::try_from(bytes.as_slice()).unwrap_err();

        assert_eq!(err, EtherNetIpError::UnknownCommand { command: 0x0099 });
    }

    #[test]
    fn reject_length_mismatch() {
        let bytes = hex::decode("65000500000000000000000000000000000000000000000001000000")
            .expect("invalid hex fixture");

        let err = EtherNetIpPacket::try_from(bytes.as_slice()).unwrap_err();

        assert_eq!(
            err,
            EtherNetIpError::LengthMismatch {
                expected: 29,
                actual: 28
            }
        );
    }

    #[test]
    fn reject_invalid_register_session_version() {
        let bytes = hex::decode("65000400000000000000000000000000000000000000000002000000")
            .expect("invalid hex fixture");

        let err = EtherNetIpPacket::try_from(bytes.as_slice()).unwrap_err();

        assert_eq!(
            err,
            EtherNetIpError::UnsupportedProtocolVersion { version: 2 }
        );
    }
}
