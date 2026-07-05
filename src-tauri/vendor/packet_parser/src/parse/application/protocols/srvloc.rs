// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use core::convert::TryFrom;

use crate::{
    checks::application::srvloc::{ensure_len, validate_packet_not_empty},
    errors::application::srvloc::SrvlocPacketParseError,
};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// Service Location Protocol Packet
///
/// ```mermaid
/// ---
/// title: SrvlocPacket
/// ---
/// packet-beta
/// 0-7: "Version u8"
/// 8-15: "Function u8"
/// 16-39: "Packet Length u16/u24"
/// 40-55: "Flags / Dialect"
/// 56-79: "Extension Offset / Language"
/// 80-95: "Transaction ID u16"
/// 96-111: "Language Tag Length u16"
/// 112-175: "Language Tag / Payload variable"
/// ```
#[derive(Debug)]
pub struct SrvlocPacket {
    pub header: SrvlocHeader,
    pub payload: SrvlocMessage,
}

#[derive(Debug)]
pub enum SrvlocHeader {
    V1(SrvlocHeaderV1),
    V2(SrvlocHeaderV2),
}

#[derive(Debug)]
pub struct SrvlocHeaderV2 {
    pub version: u8,
    pub function: u8,

    // 3 octets sur le fil -> on le stocke dans un u32
    pub packet_length: u32,

    // 2 octets sur le fil
    pub flags: u16,

    // 3 octets sur le fil -> u32
    pub next_extension_offset: u32,

    // 2 octets sur le fil -> u16
    pub xid: u16,

    // 2 octets sur le fil -> u16
    pub lang_tag_len: u16,

    // chaîne UTF-8 ("en", "fr", etc.)
    pub lang_tag: String,
}

#[derive(Debug)]
pub struct SrvlocHeaderV1 {
    pub version: u8,
    pub function: u8,
    pub packet_length: u16, // 2 octets
    pub flags: u8,
    pub dialect: u8,
    pub language: String, // 2 bytes ASCII -> "en"

    pub encoding: u8,
    pub transaction_id: u16,
    pub error_code: u16,

    pub url_length: u16,
    pub url: String,

    pub scope_list_lengh: u16,
    pub scope_list: String,
}

/// Pour l’instant on garde le payload simple.
/// Tu pourras ajouter plus tard `V1DaAdvert`, `V2DaAdvert`, etc.
#[derive(Debug)]
pub enum SrvlocMessage {
    Raw(Vec<u8>),
}

fn read_u16(buf: &[u8], offset: &mut usize) -> Result<u16, SrvlocPacketParseError> {
    ensure_len(buf, *offset + 2)?;
    let v = u16::from_be_bytes([buf[*offset], buf[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

fn read_u24(buf: &[u8], offset: &mut usize) -> Result<u32, SrvlocPacketParseError> {
    ensure_len(buf, *offset + 3)?;
    let v = ((buf[*offset] as u32) << 16)
        | ((buf[*offset + 1] as u32) << 8)
        | (buf[*offset + 2] as u32);
    *offset += 3;
    Ok(v)
}

fn read_string(
    buf: &[u8],
    offset: &mut usize,
    len: usize,
    field: &'static str,
) -> Result<String, SrvlocPacketParseError> {
    ensure_len(buf, *offset + len)?;
    let slice = &buf[*offset..*offset + len];
    *offset += len;
    String::from_utf8(slice.to_vec()).map_err(|_| SrvlocPacketParseError::InvalidUtf8(field))
}

impl TryFrom<&[u8]> for SrvlocPacket {
    type Error = SrvlocPacketParseError;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        validate_packet_not_empty(payload)?;

        let version = payload[0];

        match version {
            1 => parse_v1_packet(payload),
            2 => parse_v2_packet(payload),
            other => Err(SrvlocPacketParseError::UnsupportedVersion(other)),
        }
    }
}

/// Parse un paquet SLP v1 (DA Advert dans ton cas)
fn parse_v1_packet(payload: &[u8]) -> Result<SrvlocPacket, SrvlocPacketParseError> {
    // Layout v1 (d’après Wireshark pour DA Advertisement) :
    //  0 : Version (1)
    //  1 : Function (1)
    //  2-3 : Packet Length (u16)
    //  4 : Flags (u8)
    //  5 : Dialect (u8)
    //  6-7 : Language (2 bytes, "en")
    //
    //  8 : Encoding (u8)
    //  9-10 : Transaction ID (u16)
    //  11-12 : Error Code (u16)
    //  13-14 : URL Length (u16)
    //  ... : URL
    //  ... : Scope List Length (u16)
    //  ... : Scope List

    ensure_len(payload, 8)?;
    let version = payload[0];
    let function = payload[1];

    let packet_length = u16::from_be_bytes([payload[2], payload[3]]);
    let flags = payload[4];
    let dialect = payload[5];

    let lang_bytes = [payload[6], payload[7]];
    let language = String::from_utf8(lang_bytes.to_vec())
        .map_err(|_| SrvlocPacketParseError::InvalidUtf8("language"))?;

    let mut offset = 8;

    // Body spécifique DA Advert (ce que Wireshark te montre)
    ensure_len(payload, offset + 1)?;
    let encoding = payload[offset];
    offset += 1;

    let transaction_id = read_u16(payload, &mut offset)?;
    let error_code = read_u16(payload, &mut offset)?;

    let url_length = read_u16(payload, &mut offset)?;
    let url = read_string(payload, &mut offset, url_length as usize, "url")?;

    let scope_list_lengh = read_u16(payload, &mut offset)?;
    let scope_list = read_string(
        payload,
        &mut offset,
        scope_list_lengh as usize,
        "scope_list",
    )?;

    let header_v1 = SrvlocHeaderV1 {
        version,
        function,
        packet_length,
        flags,
        dialect,
        language,
        encoding,
        transaction_id,
        error_code,
        url_length,
        url,
        scope_list_lengh,
        scope_list,
    };

    // On met ce qu’il reste (normalement rien) dans le payload brut
    let remaining = if offset < payload.len() {
        payload[offset..].to_vec()
    } else {
        Vec::new()
    };

    Ok(SrvlocPacket {
        header: SrvlocHeader::V1(header_v1),
        payload: SrvlocMessage::Raw(remaining),
    })
}

/// Parse un paquet SLP v2 (DA Advert dans ton cas)
fn parse_v2_packet(payload: &[u8]) -> Result<SrvlocPacket, SrvlocPacketParseError> {
    // Layout SLP v2 :
    //  0 : Version (u8)
    //  1 : Function (u8)
    //  2-4 : Packet Length (u24)
    //  5-6 : Flags (u16)
    //  7-9 : Next Extension Offset (u24)
    //  10-11 : XID (u16)
    //  12-13 : Lang Tag Len (u16)
    //  14.. : Lang Tag (UTF-8)

    ensure_len(payload, 14)?;
    let mut offset = 0;

    let version = payload[offset];
    offset += 1;

    let function = payload[offset];
    offset += 1;

    let packet_length = read_u24(payload, &mut offset)?;
    let flags = read_u16(payload, &mut offset)?;
    let next_extension_offset = read_u24(payload, &mut offset)?;
    let xid = read_u16(payload, &mut offset)?;
    let lang_tag_len = read_u16(payload, &mut offset)?;
    let lang_tag = read_string(payload, &mut offset, lang_tag_len as usize, "lang_tag")?;

    let header_v2 = SrvlocHeaderV2 {
        version,
        function,
        packet_length,
        flags,
        next_extension_offset,
        xid,
        lang_tag_len,
        lang_tag,
    };

    // Pour l’instant, on laisse le body v2 dans Raw.
    // Tu pourras l’étendre plus tard (Error Code, Timestamp, URL, Scope, etc.)
    let remaining = if offset < payload.len() {
        payload[offset..].to_vec()
    } else {
        Vec::new()
    };

    Ok(SrvlocPacket {
        header: SrvlocHeader::V2(header_v2),
        payload: SrvlocMessage::Raw(remaining),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DA Advert SLP v1 : url "svc" et scope "sc"
    fn build_v1_packet() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(1); // version
        bytes.push(8); // function : DA Advert
        bytes.extend_from_slice(&24u16.to_be_bytes()); // packet length
        bytes.push(0x20); // flags
        bytes.push(0); // dialect
        bytes.extend_from_slice(b"en"); // language
        bytes.push(3); // encoding
        bytes.extend_from_slice(&0x1234u16.to_be_bytes()); // transaction id
        bytes.extend_from_slice(&0u16.to_be_bytes()); // error code
        bytes.extend_from_slice(&3u16.to_be_bytes()); // url length
        bytes.extend_from_slice(b"svc");
        bytes.extend_from_slice(&2u16.to_be_bytes()); // scope list length
        bytes.extend_from_slice(b"sc");
        bytes
    }

    /// Header SLP v2 : lang tag "en" + body brut
    fn build_v2_packet(body: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(2); // version
        bytes.push(8); // function
        let total = (16 + body.len()) as u32;
        bytes.extend_from_slice(&total.to_be_bytes()[1..]); // packet length u24
        bytes.extend_from_slice(&0x2000u16.to_be_bytes()); // flags
        bytes.extend_from_slice(&[0, 0, 0]); // next extension offset u24
        bytes.extend_from_slice(&0x4242u16.to_be_bytes()); // xid
        bytes.extend_from_slice(&2u16.to_be_bytes()); // lang tag len
        bytes.extend_from_slice(b"en");
        bytes.extend_from_slice(body);
        bytes
    }

    #[test]
    fn test_parse_v1_packet() {
        let bytes = build_v1_packet();
        let packet = SrvlocPacket::try_from(bytes.as_slice()).expect("paquet v1 valide");

        match &packet.header {
            SrvlocHeader::V1(header) => {
                assert_eq!(header.version, 1);
                assert_eq!(header.function, 8);
                assert_eq!(header.packet_length, 24);
                assert_eq!(header.flags, 0x20);
                assert_eq!(header.dialect, 0);
                assert_eq!(header.language, "en");
                assert_eq!(header.encoding, 3);
                assert_eq!(header.transaction_id, 0x1234);
                assert_eq!(header.error_code, 0);
                assert_eq!(header.url_length, 3);
                assert_eq!(header.url, "svc");
                assert_eq!(header.scope_list_lengh, 2);
                assert_eq!(header.scope_list, "sc");
            }
            other => panic!("attendu header V1, obtenu {other:?}"),
        }

        let SrvlocMessage::Raw(rest) = &packet.payload;
        assert!(rest.is_empty());
    }

    #[test]
    fn test_parse_v1_packet_with_trailing_bytes() {
        let mut bytes = build_v1_packet();
        bytes.extend_from_slice(&[0xCA, 0xFE]);

        let packet = SrvlocPacket::try_from(bytes.as_slice()).expect("paquet v1 valide");
        let SrvlocMessage::Raw(rest) = &packet.payload;
        assert_eq!(rest, &vec![0xCA, 0xFE]);
    }

    #[test]
    fn test_parse_v2_packet() {
        let bytes = build_v2_packet(&[]);
        let packet = SrvlocPacket::try_from(bytes.as_slice()).expect("paquet v2 valide");

        match &packet.header {
            SrvlocHeader::V2(header) => {
                assert_eq!(header.version, 2);
                assert_eq!(header.function, 8);
                assert_eq!(header.packet_length, 16);
                assert_eq!(header.flags, 0x2000);
                assert_eq!(header.next_extension_offset, 0);
                assert_eq!(header.xid, 0x4242);
                assert_eq!(header.lang_tag_len, 2);
                assert_eq!(header.lang_tag, "en");
            }
            other => panic!("attendu header V2, obtenu {other:?}"),
        }
    }

    #[test]
    fn test_parse_v2_packet_with_body() {
        let bytes = build_v2_packet(&[0x01, 0x02, 0x03]);
        let packet = SrvlocPacket::try_from(bytes.as_slice()).expect("paquet v2 valide");
        let SrvlocMessage::Raw(rest) = &packet.payload;
        assert_eq!(rest, &vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_empty_packet() {
        assert!(SrvlocPacket::try_from(&[][..]).is_err());
    }

    #[test]
    fn test_unsupported_version() {
        assert!(matches!(
            SrvlocPacket::try_from(&[3u8, 0, 0, 0][..]),
            Err(SrvlocPacketParseError::UnsupportedVersion(3))
        ));
    }

    #[test]
    fn test_v1_truncated_header() {
        // version 1 mais seulement 4 octets
        assert!(matches!(
            SrvlocPacket::try_from(&[1u8, 8, 0, 24][..]),
            Err(SrvlocPacketParseError::Truncated { .. })
        ));
    }

    #[test]
    fn test_v1_truncated_url() {
        // url_length annonce 100 octets absents
        let mut bytes = build_v1_packet();
        let url_len_offset = 13;
        bytes[url_len_offset] = 0;
        bytes[url_len_offset + 1] = 100;
        assert!(matches!(
            SrvlocPacket::try_from(bytes.as_slice()),
            Err(SrvlocPacketParseError::Truncated { .. })
        ));
    }

    #[test]
    fn test_v1_invalid_utf8_url() {
        let mut bytes = build_v1_packet();
        bytes[15] = 0xFF; // premier octet de l'url
        assert!(matches!(
            SrvlocPacket::try_from(bytes.as_slice()),
            Err(SrvlocPacketParseError::InvalidUtf8("url"))
        ));
    }

    #[test]
    fn test_v2_truncated_header() {
        assert!(matches!(
            SrvlocPacket::try_from(&[2u8, 8, 0, 0, 20][..]),
            Err(SrvlocPacketParseError::Truncated { .. })
        ));
    }

    #[test]
    fn test_v2_invalid_utf8_lang_tag() {
        let mut bytes = build_v2_packet(&[]);
        bytes[14] = 0xFF; // premier octet du lang tag
        assert!(matches!(
            SrvlocPacket::try_from(bytes.as_slice()),
            Err(SrvlocPacketParseError::InvalidUtf8("lang_tag"))
        ));
    }
}
