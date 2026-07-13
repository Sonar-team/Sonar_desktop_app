// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Module for parsing DHCP packets.

use std::convert::TryFrom;

use crate::{
    checks::application::dhcp::{
        DHCP_MIN_LEN, validate_dhcp_min_length, validate_hardware_address_length,
        validate_hardware_type, validate_magic_cookie, validate_operation,
    },
    errors::application::dhcp::DhcpParseError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
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
///
/// Parsing is zero-copy: fixed-size fields (`chaddr`, `sname`, `file`) and the
/// variable-length `options` area are borrowed slices into the original packet.
#[derive(Debug)]
pub struct DhcpPacket<'a> {
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
    pub chaddr: &'a [u8; 16],
    pub sname: &'a [u8; 64],
    pub file: &'a [u8; 128],
    pub options: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for DhcpPacket<'a> {
    type Error = DhcpParseError;

    fn try_from(payload: &'a [u8]) -> Result<Self, Self::Error> {
        parse_dhcp_packet(payload)
    }
}

/// Parses a DHCP packet from a given payload without copying any field.
pub fn parse_dhcp_packet(payload: &[u8]) -> Result<DhcpPacket<'_>, DhcpParseError> {
    // Check minimum length before any indexing.
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

    // Borrow the fixed-size areas directly from the packet (zero-copy).
    // The lengths are guaranteed by `validate_dhcp_min_length`, so these
    // conversions cannot fail; the error mapping avoids any `unwrap()`.
    let too_short = || DhcpParseError::PacketTooShort {
        expected: DHCP_MIN_LEN,
        actual: payload.len(),
    };
    let chaddr: &[u8; 16] = payload[28..44].try_into().map_err(|_| too_short())?;
    let sname: &[u8; 64] = payload[44..108].try_into().map_err(|_| too_short())?;
    let file: &[u8; 128] = payload[108..236].try_into().map_err(|_| too_short())?;

    let options = &payload[DHCP_MIN_LEN..];

    // Validate DHCP packet fields
    validate_operation(op)?;
    validate_hardware_type(htype)?;
    validate_hardware_address_length(hlen)?;
    validate_magic_cookie(options)?;

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

/// Fixtures partagees : trames DHCP reelles (exemple_pcap/dhcp.pcap),
/// utilisees aussi par les tests de non-regression SRVLOC (issue #3).
#[cfg(test)]
pub mod tests_fixtures {
    /// DHCP Discover reel (op=1, cookie RFC 2131), payload UDP complet.
    pub const DHCP_DISCOVER_PAYLOAD: [u8; 272] = [
        0x01, 0x01, 0x06, 0x00, 0x00, 0x00, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b,
        0x82, 0x01, 0xfc, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x82, 0x53, 0x63,
        0x35, 0x01, 0x01, 0x3d, 0x07, 0x01, 0x00, 0x0b, 0x82, 0x01, 0xfc, 0x42, 0x32, 0x04, 0x00,
        0x00, 0x00, 0x00, 0x37, 0x04, 0x01, 0x03, 0x06, 0x2a, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    /// DHCP Offer reel (op=2, cookie RFC 2131), payload UDP complet.
    pub const DHCP_OFFER_PAYLOAD: [u8; 300] = [
        0x02, 0x01, 0x06, 0x00, 0x00, 0x00, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xc0, 0xa8, 0x00, 0x0a, 0xc0, 0xa8, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0b,
        0x82, 0x01, 0xfc, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x82, 0x53, 0x63,
        0x35, 0x01, 0x02, 0x01, 0x04, 0xff, 0xff, 0xff, 0x00, 0x3a, 0x04, 0x00, 0x00, 0x07, 0x08,
        0x3b, 0x04, 0x00, 0x00, 0x0c, 0x4e, 0x33, 0x04, 0x00, 0x00, 0x0e, 0x10, 0x36, 0x04, 0xc0,
        0xa8, 0x00, 0x01, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a valid 236-byte fixed DHCP header, followed by `options`.
    fn build_dhcp_payload(options: &[u8]) -> Vec<u8> {
        let mut payload = vec![
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
        ];
        payload.extend_from_slice(&[0x00; 64]); // sname
        payload.extend_from_slice(&[0x00; 128]); // file
        payload.extend_from_slice(options);
        payload
    }

    #[test]
    fn test_parse_dhcp_packet() {
        let dhcp_payload = build_dhcp_payload(&[
            0x63, 0x82, 0x53, 0x63, // Magic cookie
            0x35, 0x01, 0x05, // DHCP message type
            0xFF, // End option
        ]);

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
                    &[
                        0x00, 0x0C, 0x29, 0x36, 0x57, 0xD2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00
                    ]
                );
                assert_eq!(packet.sname, &[0u8; 64]);
                assert_eq!(packet.file, &[0u8; 128]);
                assert_eq!(
                    packet.options,
                    &[0x63, 0x82, 0x53, 0x63, 0x35, 0x01, 0x05, 0xFF]
                );
            }
            Err(_) => panic!("Expected DHCP packet"),
        }
    }

    #[test]
    fn test_parse_dhcp_packet_borrows_from_payload() {
        // Zero-copy check: the borrowed fields must point inside the payload.
        let dhcp_payload = build_dhcp_payload(&[0x63, 0x82, 0x53, 0x63, 0xFF]);
        let packet = DhcpPacket::try_from(dhcp_payload.as_slice()).expect("valid packet");

        let base = dhcp_payload.as_ptr() as usize;
        assert_eq!(packet.chaddr.as_ptr() as usize, base + 28);
        assert_eq!(packet.sname.as_ptr() as usize, base + 44);
        assert_eq!(packet.file.as_ptr() as usize, base + 108);
        assert_eq!(packet.options.as_ptr() as usize, base + 236);
    }

    #[test]
    fn test_parse_dhcp_packet_empty_options() {
        // Exactly the fixed header, no options at all.
        let dhcp_payload = build_dhcp_payload(&[]);
        assert_eq!(dhcp_payload.len(), 236);

        let packet = DhcpPacket::try_from(dhcp_payload.as_slice()).expect("valid packet");
        assert!(packet.options.is_empty());
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
    fn test_parse_dhcp_packet_one_byte_short_of_header() {
        // 235 bytes: one byte less than the fixed DHCP header.
        let payload = build_dhcp_payload(&[]);
        let truncated = &payload[..235];
        assert!(matches!(
            DhcpPacket::try_from(truncated),
            Err(DhcpParseError::PacketTooShort {
                expected: 236,
                actual: 235
            })
        ));
    }

    #[test]
    fn test_parse_dhcp_packet_empty_payload() {
        assert!(matches!(
            DhcpPacket::try_from(&[][..]),
            Err(DhcpParseError::PacketTooShort {
                expected: 236,
                actual: 0
            })
        ));
    }

    #[test]
    fn test_parse_dhcp_packet_invalid_op() {
        let mut invalid_op_payload = build_dhcp_payload(&[0x63, 0x82, 0x53, 0x63, 0xFF]);
        invalid_op_payload[0] = 0x03;

        match DhcpPacket::try_from(invalid_op_payload.as_slice()) {
            Ok(_) => panic!("Expected invalid DHCP packet due to invalid op code"),
            Err(err) => assert_eq!(err, DhcpParseError::InvalidOperation { op: 3 }),
        }
    }

    #[test]
    fn test_dhcp_packet_display() {
        let dhcp_payload = build_dhcp_payload(&[0x63, 0x82, 0x53, 0x63, 0xFF]);
        let packet = DhcpPacket::try_from(dhcp_payload.as_slice()).expect("valid packet");

        let rendered = packet.to_string();
        assert!(rendered.starts_with("DHCP Packet:"));
        assert!(rendered.contains("op=1"));
        assert!(rendered.contains("xid=3903F326"));
        assert!(rendered.contains("options=[63, 82, 53, 63, FF]"));
    }
}
