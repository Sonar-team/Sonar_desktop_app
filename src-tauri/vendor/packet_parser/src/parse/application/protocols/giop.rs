use std::convert::TryFrom;
use std::str;

use thiserror::Error;

//
// =========================
//   Erreurs de parsing
// =========================
//

#[derive(Debug, Error, PartialEq)]
pub enum GiopParseError {
    #[error("Invalid GIOP packet length")]
    InvalidSize,

    #[error("Invalid GIOP magic (expected 'GIOP')")]
    InvalidMagic,

    #[error("Unsupported GIOP version {0}.{1}")]
    UnsupportedVersion(u8, u8),

    #[error("Unknown GIOP message type {0}")]
    UnknownMessageType(u8),

    #[error("Truncated GIOP body (expected {expected} bytes, got {actual})")]
    TruncatedBody { expected: usize, actual: usize },

    #[error("Invalid UTF-8 in string field")]
    InvalidUtf8,

    #[error("Unexpected end of buffer")]
    UnexpectedEof,

    #[error("Other GIOP parsing error: {0}")]
    Other(&'static str),
}

//
// =========================
//   Types de messages GIOP
// =========================
//

#[derive(Debug)]
pub enum GiopMessageType {
    Request,
    Reply,
    CancelRequest,
    LocateRequest,
    LocateReply,
    CloseConnection,
    MessageError,
    Fragment,
}

impl TryFrom<u8> for GiopMessageType {
    type Error = GiopParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use GiopMessageType::*;
        Ok(match value {
            0 => Request,
            1 => Reply,
            2 => CancelRequest,
            3 => LocateRequest,
            4 => LocateReply,
            5 => CloseConnection,
            6 => MessageError,
            7 => Fragment,
            _ => return Err(GiopParseError::UnknownMessageType(value)),
        })
    }
}

//
// =========================
//        Header GIOP
// =========================
//

#[derive(Debug)]
pub struct GiopHeader {
    pub magic: [u8; 4],    // "GIOP"
    pub major_version: u8, // 1
    pub minor_version: u8, // 0, 1, 2
    pub flags: u8,         // bit 0 = endianness du body
    pub message_type: GiopMessageType,
    pub message_length: u32, // taille du body uniquement
}

impl GiopHeader {
    pub const HEADER_LEN: usize = 12;

    fn ensure_min_len(payload: &[u8]) -> Result<(), GiopParseError> {
        if payload.len() < Self::HEADER_LEN {
            return Err(GiopParseError::InvalidSize);
        }
        Ok(())
    }

    fn parse_magic(payload: &[u8]) -> Result<[u8; 4], GiopParseError> {
        let magic: [u8; 4] = payload[0..4].try_into().unwrap();
        if &magic != b"GIOP" {
            return Err(GiopParseError::InvalidMagic);
        }
        Ok(magic)
    }
}

impl TryFrom<&[u8]> for GiopHeader {
    type Error = GiopParseError;

    fn try_from(payload: &[u8]) -> Result<Self, Self::Error> {
        GiopHeader::ensure_min_len(payload)?;

        let magic = GiopHeader::parse_magic(payload)?;
        let major_version = payload[4];
        let minor_version = payload[5];

        if major_version != 1 || minor_version > 2 {
            return Err(GiopParseError::UnsupportedVersion(
                major_version,
                minor_version,
            ));
        }

        let flags = payload[6];
        let message_type_raw = payload[7];
        let message_type = GiopMessageType::try_from(message_type_raw)?;

        // MessageSize est toujours en big-endian dans le header
        let message_length = u32::from_be_bytes(payload[8..12].try_into().unwrap());
        Ok(GiopHeader {
            magic,
            major_version,
            minor_version,
            flags,
            message_type,
            message_length,
        })
    }
}

//
// =========================
//      Structures GIOP
// =========================
//

#[derive(Debug)]
pub struct GiopPacket {
    pub header: GiopHeader,
    pub payload: GiopMessage,
}

#[derive(Debug)]
pub enum GiopMessage {
    Request(GiopRequest),
    Reply(GiopReply),
    Fragment(GiopFragment),
    Other,
    // Les autres types peuvent être ajoutés plus tard
}

#[derive(Debug)]
pub enum TargetAddress {
    KeyAddr(Vec<u8>),
    ProfileAddr(Vec<u8>),
    ReferenceAddr(Vec<u8>),
}

#[derive(Debug)]
pub struct ServiceContext {
    pub context_id: u32,
    pub context_data: Vec<u8>,
}

#[derive(Debug)]
pub struct GiopRequest {
    pub request_id: u32,
    pub response_flags: u8, // 0..3 (SyncScope)
    pub target: TargetAddress,
    pub operation: String,
    pub service_contexts: Vec<ServiceContext>,
    pub stub_data: Vec<u8>, // CDR payload (arguments), non décodé ici
}

// Placeholders pour plus tard
#[derive(Debug)]
pub struct GiopReply;

#[derive(Debug)]
pub struct GiopFragment;

//
// =========================
//   Parsing GiopPacket
// =========================
//

impl TryFrom<&[u8]> for GiopPacket {
    type Error = GiopParseError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let header = GiopHeader::try_from(buf)?;
        let total_needed = GiopHeader::HEADER_LEN + header.message_length as usize;
        if buf.len() < total_needed {
            return Err(GiopParseError::TruncatedBody {
                expected: total_needed,
                actual: buf.len(),
            });
        }

        // let body = &buf[GiopHeader::HEADER_LEN..total_needed];
        // println!("giop body parsed");

        // Bit 0 des flags = endianness du body
        let _little_endian = (header.flags & 0x01) != 0;

        // let payload = match header.message_type {
        //     GiopMessageType::Request => {
        //         if let Ok(req) = GiopRequest::parse(body, little_endian) {
        //             println!("giop request parsed");
        //             GiopMessage::Request(req)
        //         } else {
        //             return Err(GiopParseError::Other("Failed to parse GiopRequest"));
        //         }
        //     }
        //     GiopMessageType::Reply => {
        //         // À implémenter plus tard si besoin
        //         GiopMessage::Reply(GiopReply)
        //     }
        //     GiopMessageType::Fragment => {
        //         // À implémenter plus tard si besoin
        //         GiopMessage::Fragment(GiopFragment)
        //     }
        //     _ => {
        //         println!("giop message type not implemented");
        //         return Err(GiopParseError::Other(
        //             "Message type not implemented in parser",
        //         ))
        //     }
        // };

        let payload = GiopMessage::Other;
        Ok(GiopPacket { header, payload })
    }
}

//
// =========================
//   Petit curseur de lecture
// =========================
//

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
    little_endian: bool,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8], little_endian: bool) -> Self {
        Self {
            buf,
            pos: 0,
            little_endian,
        }
    }

    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    fn read_u8(&mut self) -> Result<u8, GiopParseError> {
        if self.remaining() < 1 {
            return Err(GiopParseError::UnexpectedEof);
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }

    fn read_u32(&mut self) -> Result<u32, GiopParseError> {
        if self.remaining() < 4 {
            return Err(GiopParseError::UnexpectedEof);
        }
        let bytes: [u8; 4] = self.buf[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        Ok(if self.little_endian {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], GiopParseError> {
        if self.remaining() < len {
            return Err(GiopParseError::UnexpectedEof);
        }
        let slice = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    fn read_string(&mut self) -> Result<String, GiopParseError> {
        // String CDR : ulong length, puis bytes (souvent terminés par 0)
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        let str_bytes = if !bytes.is_empty() && bytes[len - 1] == 0 {
            &bytes[..len - 1]
        } else {
            bytes
        };

        str::from_utf8(str_bytes)
            .map(|s| s.to_string())
            .map_err(|_| GiopParseError::InvalidUtf8)
    }
}

//
// =========================
//   Parsing d'un Request
// =========================
//

impl GiopRequest {
    pub fn parse(body: &[u8], little_endian: bool) -> Result<Self, GiopParseError> {
        let mut cur = Cursor::new(body, little_endian);

        let request_id = cur.read_u32()?;
        let response_flags = cur.read_u8()?;

        // Reserved 3 octets
        let _r1 = cur.read_u8()?;
        let _r2 = cur.read_u8()?;
        let _r3 = cur.read_u8()?;

        let target = parse_target_address(&mut cur)?;
        let operation = cur.read_string()?;
        let service_contexts = parse_service_context_list(&mut cur)?;

        // Le reste = stub data (arguments CDR)
        let remaining = cur.remaining();
        let stub_data = if remaining > 0 {
            cur.read_bytes(remaining)?.to_vec()
        } else {
            Vec::new()
        };
        Ok(GiopRequest {
            request_id,
            response_flags,
            target,
            operation,
            service_contexts,
            stub_data,
        })
    }
}

fn parse_target_address(cur: &mut Cursor<'_>) -> Result<TargetAddress, GiopParseError> {
    let discriminator = cur.read_u8()?;

    match discriminator {
        0 => {
            // KeyAddr: sequence<octet>
            let len = cur.read_u32()? as usize;
            let data = cur.read_bytes(len)?.to_vec();
            Ok(TargetAddress::KeyAddr(data))
        }
        1 => {
            // ProfileAddr : brut pour l'instant
            let len = cur.read_u32()? as usize;
            let data = cur.read_bytes(len)?.to_vec();
            Ok(TargetAddress::ProfileAddr(data))
        }
        2 => {
            // ReferenceAddr : brut pour l'instant
            let len = cur.read_u32()? as usize;
            let data = cur.read_bytes(len)?.to_vec();
            Ok(TargetAddress::ReferenceAddr(data))
        }
        _ => Err(GiopParseError::Other("Unknown TargetAddress discriminator")),
    }
}

fn parse_service_context_list(cur: &mut Cursor<'_>) -> Result<Vec<ServiceContext>, GiopParseError> {
    let count = cur.read_u32()? as usize;
    let mut contexts = Vec::with_capacity(count);

    for _ in 0..count {
        let context_id = cur.read_u32()?;
        let len = cur.read_u32()? as usize;
        let data = cur.read_bytes(len)?.to_vec();
        contexts.push(ServiceContext {
            context_id,
            context_data: data,
        });
    }

    Ok(contexts)
}
