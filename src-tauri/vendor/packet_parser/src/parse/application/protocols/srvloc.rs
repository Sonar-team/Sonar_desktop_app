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
