// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Module for parsing DHCP packets.

use std::convert::TryFrom;
use std::fmt;

use crate::{
    checks::application::dhcp::{
        validate_dhcp_min_length, validate_hardware_address_length, validate_hardware_type,
        validate_operation,
    },
    errors::application::dhcp::DhcpParseError,
};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// DHCP Packet
///
/// ```mermaid
/// ---
/// title: DhcpPacket
/// ---
/// packet-beta
/// 0-7: "Operation u8"
/// 8-15: "Hardware Type u8"
/// 16-23: "Hardware Address Length u8"
/// 24-31: "Hops u8"
/// 32-63: "Transaction ID u32"
/// 64-79: "Seconds u16"
/// 80-95: "Flags u16"
/// 96-127: "Client IP Address u32"
/// 128-159: "Your IP Address u32"
/// 160-191: "Server IP Address u32"
/// 192-223: "Gateway IP Address u32"
/// 224-351: "Client Hardware Address bytes[16]"
/// 352-863: "Server Host Name bytes[64]"
/// 864-1887: "Boot File Name bytes[128]"
/// 1888-1951: "Options variable"
/// ```
///
/// The `DhcpPacket` struct represents a parsed DHCP packet.
#[derive(Debug)]
pub struct DhcpPacket {
    pub op: u8,
    pub htype: u8,
    pub hlen: u8,
    pub hops: u8,
    pub xid: u32,
    pub secs: u16,
    pub flags: u16,
    pub ciaddr: [u8; 4],
    pub yiaddr: [u8; 4],
    pub siaddr: [u8; 4],
    pub giaddr: [u8; 4],
    pub chaddr: [u8; 16],
    pub sname: [u8; 64],
    pub file: [u8; 128],
    pub options: Vec<u8>,
}

impl fmt::Display for DhcpPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DHCP Packet: op={}, htype={}, hlen={}, hops={}, xid={:08X}, secs={}, flags={}, ciaddr={:?}, yiaddr={:?}, siaddr={:?}, giaddr={:?}, chaddr={:?}, sname={:?}, file={:?}, options={:02X?}",
            self.op,
            self.htype,
            self.hlen,
            self.hops,
            self.xid,
            self.secs,
            self.flags,
            self.ciaddr,
            self.yiaddr,
            self.siaddr,
            self.giaddr,
            self.chaddr,
            self.sname,
            self.file,
            self.options
        )
    }
}

impl TryFrom<&[u8]> for DhcpPacket {
    type Error = DhcpParseError;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        parse_dhcp_packet(payload)
    }
}

/// Parses a DHCP packet from a given payload.
pub fn parse_dhcp_packet(payload: &[u8]) -> Result<DhcpPacket, DhcpParseError> {
    // Check minimum length
    validate_dhcp_min_length(payload)?;

    let op = payload[0];
    let htype = payload[1];
    let hlen = payload[2];
    let hops = payload[3];
    let xid = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let secs = u16::from_be_bytes([payload[8], payload[9]]);
    let flags = u16::from_be_bytes([payload[10], payload[11]]);
    let ciaddr = [payload[12], payload[13], payload[14], payload[15]];
    let yiaddr = [payload[16], payload[17], payload[18], payload[19]];
    let siaddr = [payload[20], payload[21], payload[22], payload[23]];
    let giaddr = [payload[24], payload[25], payload[26], payload[27]];
    let mut chaddr = [0u8; 16];
    chaddr.copy_from_slice(&payload[28..44]);
    let mut sname = [0u8; 64];
    sname.copy_from_slice(&payload[44..108]);
    let mut file = [0u8; 128];
    file.copy_from_slice(&payload[108..236]);

    let options = payload[236..].to_vec();

    // Validate DHCP packet fields
    validate_operation(op)?;
    validate_hardware_type(htype)?;
    validate_hardware_address_length(hlen)?;

    Ok(DhcpPacket {
        op,
        htype,
        hlen,
        hops,
        xid,
        secs,
        flags,
        ciaddr,
        yiaddr,
        siaddr,
        giaddr,
        chaddr,
        sname,
        file,
        options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dhcp_packet() {
        let dhcp_payload = [
            0x01, 0x01, 0x06, 0x00, // op, htype, hlen, hops
            0x39, 0x03, 0xF3, 0x26, // xid
            0x00, 0x00, // secs
            0x00, 0x00, // flags
            0x00, 0x00, 0x00, 0x00, // ciaddr
            0x00, 0x00, 0x00, 0x00, // yiaddr
            0x00, 0x00, 0x00, 0x00, // siaddr
            0x00, 0x00, 0x00, 0x00, // giaddr
            0x00, 0x0C, 0x29, 0x36, 0x57, 0xD2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, // chaddr
        ]
        .iter()
        .cloned()
        .chain([0x00; 64].iter().cloned())
        .chain([0x00; 128].iter().cloned())
        .chain(
            [
                0x63, 0x82, 0x53, 0x63, // Magic cookie
                0x35, 0x01, 0x05, // DHCP message type
                0xFF, // End option
            ]
            .iter()
            .cloned(),
        )
        .collect::<Vec<u8>>();

        match DhcpPacket::try_from(dhcp_payload.as_slice()) {
            Ok(packet) => {
                assert_eq!(packet.op, 1);
                assert_eq!(packet.htype, 1);
                assert_eq!(packet.hlen, 6);
                assert_eq!(packet.hops, 0);
                assert_eq!(packet.xid, 0x3903F326);
                assert_eq!(packet.secs, 0);
                assert_eq!(packet.flags, 0);
                assert_eq!(packet.ciaddr, [0, 0, 0, 0]);
                assert_eq!(packet.yiaddr, [0, 0, 0, 0]);
                assert_eq!(packet.siaddr, [0, 0, 0, 0]);
                assert_eq!(packet.giaddr, [0, 0, 0, 0]);
                assert_eq!(
                    packet.chaddr,
                    [
                        0x00, 0x0C, 0x29, 0x36, 0x57, 0xD2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00
                    ]
                );
                assert_eq!(packet.sname, [0u8; 64]);
                assert_eq!(packet.file, [0u8; 128]);
                assert_eq!(
                    packet.options,
                    vec![0x63, 0x82, 0x53, 0x63, 0x35, 0x01, 0x05, 0xFF]
                );
            }
            Err(_) => panic!("Expected DHCP packet"),
        }
    }

    #[test]
    fn test_parse_dhcp_packet_short_payload() {
        let short_payload = vec![0x01, 0x01, 0x06, 0x00, 0x39, 0x03, 0xF3, 0x26];
        match DhcpPacket::try_from(short_payload.as_slice()) {
            Ok(_) => panic!("Expected invalid DHCP packet due to short payload"),
            Err(err) => assert_eq!(
                err,
                DhcpParseError::PacketTooShort {
                    expected: 236,
                    actual: 8
                }
            ),
        }
    }

    #[test]
    fn test_parse_dhcp_packet_invalid_op() {
        let invalid_op_payload = vec![
            0x03, 0x01, 0x06, 0x00, 0x39, 0x03, 0xF3, 0x26, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x0C, 0x29, 0x36, 0x57, 0xD2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
        .iter()
        .cloned()
        .chain([0x00; 64].iter().cloned())
        .chain([0x00; 128].iter().cloned())
        .chain(
            [0x63, 0x82, 0x53, 0x63, 0x35, 0x01, 0x05, 0xFF]
                .iter()
                .cloned(),
        )
        .collect::<Vec<u8>>();

        match DhcpPacket::try_from(invalid_op_payload.as_slice()) {
            Ok(_) => panic!("Expected invalid DHCP packet due to invalid op code"),
            Err(err) => assert_eq!(err, DhcpParseError::InvalidOperation { op: 3 }),
        }
    }
}
