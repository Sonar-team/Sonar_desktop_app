use core::convert::TryFrom;

use thiserror::Error;

pub const OPCUA_TCP_HEADER_LEN: usize = 8;

#[derive(Debug)]
pub struct OpcuaPacket<'a> {
    pub chunks: Vec<OpcuaChunk<'a>>,
}

#[derive(Debug)]
pub struct OpcuaChunk<'a> {
    pub header: OpcuaTcpHeader,
    pub payload: OpcuaPayload<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpcuaTcpHeader {
    pub message_type: OpcuaMessageType,
    pub chunk_type: OpcuaChunkType,
    pub message_size: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpcuaMessageType {
    Hello,
    Acknowledge,
    ErrorMessage,
    ReverseHello,
    OpenSecureChannel,
    Message,
    CloseSecureChannel,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpcuaChunkType {
    Final,
    Intermediate,
    Abort,
}

#[derive(Debug)]
pub enum OpcuaPayload<'a> {
    Hello(OpcuaHello<'a>),
    Acknowledge(OpcuaAcknowledge),
    Error(OpcuaError<'a>),
    ReverseHello(OpcuaReverseHello<'a>),
    SecureConversation(OpcuaSecureConversation<'a>),
    Partial(&'a [u8]),
}

#[derive(Debug)]
pub struct OpcuaHello<'a> {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
    pub endpoint_url: Option<&'a str>,
}

#[derive(Debug)]
pub struct OpcuaAcknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

#[derive(Debug)]
pub struct OpcuaError<'a> {
    pub status_code: u32,
    pub reason: Option<&'a str>,
}

#[derive(Debug)]
pub struct OpcuaReverseHello<'a> {
    pub server_uri: Option<&'a str>,
    pub endpoint_url: Option<&'a str>,
}

#[derive(Debug)]
pub struct OpcuaSecureConversation<'a> {
    pub secure_channel_id: u32,
    pub data: &'a [u8],
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OpcuaParseError {
    #[error("OPC UA packet too short: expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },

    #[error("unknown OPC UA message type: {0:?}")]
    UnknownMessageType([u8; 3]),

    #[error("unknown OPC UA chunk type: 0x{0:02x}")]
    UnknownChunkType(u8),

    #[error("invalid OPC UA message size: {size}")]
    InvalidMessageSize { size: u32 },

    #[error("truncated OPC UA chunk: expected {expected} bytes, got {actual}")]
    TruncatedChunk { expected: usize, actual: usize },

    #[error("OPC UA body too short: expected at least {expected} bytes, got {actual}")]
    BodyTooShort { expected: usize, actual: usize },

    #[error("invalid OPC UA string length: {length}")]
    InvalidStringLength { length: i32 },

    #[error("truncated OPC UA string: expected {expected} bytes, got {actual}")]
    TruncatedString { expected: usize, actual: usize },

    #[error("invalid UTF-8 in OPC UA string")]
    InvalidUtf8,
}

impl TryFrom<[u8; 3]> for OpcuaMessageType {
    type Error = OpcuaParseError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match &value {
            b"HEL" => Ok(OpcuaMessageType::Hello),
            b"ACK" => Ok(OpcuaMessageType::Acknowledge),
            b"ERR" => Ok(OpcuaMessageType::ErrorMessage),
            b"RHE" => Ok(OpcuaMessageType::ReverseHello),
            b"OPN" => Ok(OpcuaMessageType::OpenSecureChannel),
            b"MSG" => Ok(OpcuaMessageType::Message),
            b"CLO" => Ok(OpcuaMessageType::CloseSecureChannel),
            _ => Err(OpcuaParseError::UnknownMessageType(value)),
        }
    }
}

impl TryFrom<u8> for OpcuaChunkType {
    type Error = OpcuaParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'F' => Ok(OpcuaChunkType::Final),
            b'C' => Ok(OpcuaChunkType::Intermediate),
            b'A' => Ok(OpcuaChunkType::Abort),
            _ => Err(OpcuaParseError::UnknownChunkType(value)),
        }
    }
}

impl TryFrom<&[u8]> for OpcuaTcpHeader {
    type Error = OpcuaParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < OPCUA_TCP_HEADER_LEN {
            return Err(OpcuaParseError::PacketTooShort {
                expected: OPCUA_TCP_HEADER_LEN,
                actual: bytes.len(),
            });
        }

        let message_type = OpcuaMessageType::try_from([bytes[0], bytes[1], bytes[2]])?;
        let chunk_type = OpcuaChunkType::try_from(bytes[3])?;
        let message_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

        if message_size < OPCUA_TCP_HEADER_LEN as u32 {
            return Err(OpcuaParseError::InvalidMessageSize { size: message_size });
        }

        Ok(OpcuaTcpHeader {
            message_type,
            chunk_type,
            message_size,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for OpcuaPacket<'a> {
    type Error = OpcuaParseError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() < OPCUA_TCP_HEADER_LEN {
            return Err(OpcuaParseError::PacketTooShort {
                expected: OPCUA_TCP_HEADER_LEN,
                actual: bytes.len(),
            });
        }

        let mut chunks = Vec::new();
        let mut offset = 0usize;

        while offset < bytes.len() {
            let remaining = &bytes[offset..];
            if remaining.len() < OPCUA_TCP_HEADER_LEN {
                return Err(OpcuaParseError::PacketTooShort {
                    expected: OPCUA_TCP_HEADER_LEN,
                    actual: remaining.len(),
                });
            }

            let header = OpcuaTcpHeader::try_from(remaining)?;
            let message_size = header.message_size as usize;

            let (consumed, payload) = if remaining.len() < message_size {
                let body = &remaining[OPCUA_TCP_HEADER_LEN..];
                (remaining.len(), OpcuaPayload::Partial(body))
            } else {
                let body = &remaining[OPCUA_TCP_HEADER_LEN..message_size];
                (message_size, parse_payload(header.message_type, body)?)
            };

            chunks.push(OpcuaChunk { header, payload });
            offset += consumed;
        }

        Ok(OpcuaPacket { chunks })
    }
}

fn parse_payload<'a>(
    message_type: OpcuaMessageType,
    body: &'a [u8],
) -> Result<OpcuaPayload<'a>, OpcuaParseError> {
    match message_type {
        OpcuaMessageType::Hello => parse_hello(body).map(OpcuaPayload::Hello),
        OpcuaMessageType::Acknowledge => parse_acknowledge(body).map(OpcuaPayload::Acknowledge),
        OpcuaMessageType::ErrorMessage => parse_error(body).map(OpcuaPayload::Error),
        OpcuaMessageType::ReverseHello => parse_reverse_hello(body).map(OpcuaPayload::ReverseHello),
        OpcuaMessageType::OpenSecureChannel
        | OpcuaMessageType::Message
        | OpcuaMessageType::CloseSecureChannel => {
            parse_secure_conversation(body).map(OpcuaPayload::SecureConversation)
        }
    }
}

fn parse_hello(bytes: &[u8]) -> Result<OpcuaHello<'_>, OpcuaParseError> {
    const FIXED_LEN: usize = 20;
    if bytes.len() < FIXED_LEN {
        return Err(OpcuaParseError::BodyTooShort {
            expected: FIXED_LEN,
            actual: bytes.len(),
        });
    }

    let (endpoint_url, _) = parse_ua_string(&bytes[FIXED_LEN..])?;

    Ok(OpcuaHello {
        protocol_version: read_u32_le(bytes, 0),
        receive_buffer_size: read_u32_le(bytes, 4),
        send_buffer_size: read_u32_le(bytes, 8),
        max_message_size: read_u32_le(bytes, 12),
        max_chunk_count: read_u32_le(bytes, 16),
        endpoint_url,
    })
}

fn parse_acknowledge(bytes: &[u8]) -> Result<OpcuaAcknowledge, OpcuaParseError> {
    const ACK_LEN: usize = 20;
    if bytes.len() < ACK_LEN {
        return Err(OpcuaParseError::BodyTooShort {
            expected: ACK_LEN,
            actual: bytes.len(),
        });
    }

    Ok(OpcuaAcknowledge {
        protocol_version: read_u32_le(bytes, 0),
        receive_buffer_size: read_u32_le(bytes, 4),
        send_buffer_size: read_u32_le(bytes, 8),
        max_message_size: read_u32_le(bytes, 12),
        max_chunk_count: read_u32_le(bytes, 16),
    })
}

fn parse_error(bytes: &[u8]) -> Result<OpcuaError<'_>, OpcuaParseError> {
    const STATUS_LEN: usize = 4;
    if bytes.len() < STATUS_LEN {
        return Err(OpcuaParseError::BodyTooShort {
            expected: STATUS_LEN,
            actual: bytes.len(),
        });
    }

    let (reason, _) = parse_ua_string(&bytes[STATUS_LEN..])?;

    Ok(OpcuaError {
        status_code: read_u32_le(bytes, 0),
        reason,
    })
}

fn parse_reverse_hello(bytes: &[u8]) -> Result<OpcuaReverseHello<'_>, OpcuaParseError> {
    let (server_uri, consumed) = parse_ua_string(bytes)?;
    let (endpoint_url, _) = parse_ua_string(&bytes[consumed..])?;

    Ok(OpcuaReverseHello {
        server_uri,
        endpoint_url,
    })
}

fn parse_secure_conversation(bytes: &[u8]) -> Result<OpcuaSecureConversation<'_>, OpcuaParseError> {
    const SECURE_CHANNEL_ID_LEN: usize = 4;
    if bytes.len() < SECURE_CHANNEL_ID_LEN {
        return Err(OpcuaParseError::BodyTooShort {
            expected: SECURE_CHANNEL_ID_LEN,
            actual: bytes.len(),
        });
    }

    Ok(OpcuaSecureConversation {
        secure_channel_id: read_u32_le(bytes, 0),
        data: &bytes[SECURE_CHANNEL_ID_LEN..],
    })
}

fn parse_ua_string(bytes: &[u8]) -> Result<(Option<&str>, usize), OpcuaParseError> {
    const LEN_FIELD: usize = 4;
    if bytes.len() < LEN_FIELD {
        return Err(OpcuaParseError::BodyTooShort {
            expected: LEN_FIELD,
            actual: bytes.len(),
        });
    }

    let length = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if length == -1 {
        return Ok((None, LEN_FIELD));
    }
    if length < -1 {
        return Err(OpcuaParseError::InvalidStringLength { length });
    }

    let length = length as usize;
    let end = LEN_FIELD + length;
    if bytes.len() < end {
        return Err(OpcuaParseError::TruncatedString {
            expected: end,
            actual: bytes.len(),
        });
    }

    let value =
        core::str::from_utf8(&bytes[LEN_FIELD..end]).map_err(|_| OpcuaParseError::InvalidUtf8)?;

    Ok((Some(value), end))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::hex_stream_to_bytes;

    #[test]
    fn parse_hello_chunk() {
        let mut bytes = b"HELF".to_vec();
        bytes.extend_from_slice(&40u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&65536u32.to_le_bytes());
        bytes.extend_from_slice(&65536u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&8i32.to_le_bytes());
        bytes.extend_from_slice(b"opc.tcp:");

        let packet = OpcuaPacket::try_from(bytes.as_slice()).unwrap();
        assert_eq!(packet.chunks.len(), 1);
        assert_eq!(
            packet.chunks[0].header.message_type,
            OpcuaMessageType::Hello
        );

        match &packet.chunks[0].payload {
            OpcuaPayload::Hello(hello) => {
                assert_eq!(hello.receive_buffer_size, 65536);
                assert_eq!(hello.endpoint_url, Some("opc.tcp:"));
            }
            _ => panic!("expected hello payload"),
        }
    }

    #[test]
    fn parse_acknowledge_chunk() {
        let mut bytes = b"ACKF".to_vec();
        bytes.extend_from_slice(&28u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&65536u32.to_le_bytes());
        bytes.extend_from_slice(&65536u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());

        let packet = OpcuaPacket::try_from(bytes.as_slice()).unwrap();
        assert_eq!(packet.chunks.len(), 1);

        match &packet.chunks[0].payload {
            OpcuaPayload::Acknowledge(ack) => {
                assert_eq!(ack.send_buffer_size, 65536);
                assert_eq!(ack.max_chunk_count, 0);
            }
            _ => panic!("expected acknowledge payload"),
        }
    }

    #[test]
    fn reject_unknown_message_type() {
        let mut bytes = b"BADF".to_vec();
        bytes.extend_from_slice(&8u32.to_le_bytes());

        let err = OpcuaPacket::try_from(bytes.as_slice()).unwrap_err();
        assert_eq!(err, OpcuaParseError::UnknownMessageType(*b"BAD"));
    }

    #[test]
    fn parse_truncated_chunk_as_partial() {
        let mut bytes = b"ACKF".to_vec();
        bytes.extend_from_slice(&28u32.to_le_bytes());

        let packet = OpcuaPacket::try_from(bytes.as_slice()).unwrap();
        assert_eq!(packet.chunks.len(), 1);
        assert_eq!(packet.chunks[0].header.message_size, 28);
        match packet.chunks[0].payload {
            OpcuaPayload::Partial(body) => assert!(body.is_empty()),
            _ => panic!("expected partial OPC UA payload"),
        }
    }

    #[test]
    fn parse_opcua_conversation_message_creatsessionresponse() {
        let hex_stream = "4d534746961f0000601900000100000035000000030000000100d001452e3c585c2bca0101000000000000000000000000000000020a00c99305000100801900000000004ced4020000000505b24529069ea6f8619dbf09f97cbfea17e5ea5d9aa7f28fec034c549085e0eeb030000308203e7308202d3a00302010202100733ad3259aa9c834b6bdc847ebc9756300906052b0e03021d05003049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f3230343829301e170d3039303831373138313633365a170d3139303831373138313633365a3049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f323034382930820122300d06092a864886f70d01010105000382010f003082010a02820101008c5922676748f8dffb65315b678d9ed77ad37322003a6bc1b7a779fc79298cccd3f52aceb9c88bb5d5b39f3ce479c1a7852d4e4b1c082ea05a863007914d0648b41e149e6ba3b2cf3d805a9eb97ac06b7d3005aab64dce3e4ed227f2615b2cc684fbf28c49d07e5087821febb0722ae5a370f041f8de3d7c395dc0ed05c2d29b6a546fd795c514c1d67e0935d6ce4a40310b23df2cb1e032c0a0c8cbb34c048820b98c661b93f63a05db5830753b1d8211382fcb5d6b88b440a03c9c83443efb137dc1d84336fad55bd87613995f2e488a80786f7f6e5269a4272f7795eb9272bfd6ef60e9952dc8bc7a0a714585ef855225275a278e3e68b2daf2a69f70dfe50203010001a381d23081cf301d0603551d0e04160414324f56e634d0b001e390d2d5055799f1ad24153c301f0603551d01041830168014324f56e634d0b001e390d2d5055799f1ad24153c300c0603551d130101ff04023000300e0603551d0f0101ff0404030202f430200603551d250101ff0416301406082b0601050507030106082b06010505070302304d0603551d07044630448634687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829820c766d2d78702d73746576656e300906052b0e03021d050003820101001c90f3f487d80ab4bef63297717a14998ab9a17362fd940b87a1ba7f1ec7eeb97878871f68099852e757f75644e5aed1670e7a25f2fa0ef0fe403b1e6b82b5dd529c8ebea9c3cb9478237064c3df5ba17bf988cf7d708e641af47f0ab32a4fed98a88425e2a147647cf88f25a6606ac30693f581c93965bc4064e391b2ead5df78869e8ce8934247a5bebb4f84c8f3bb93c3fc0781128afc0adb091537521ad306d717ebb119127a651ac83611c37f6906afcddf8f0a73e7d973fed39753a7742221b20bb013cbc276fc3fa0bc53956c5d41ab613f26af3cd69da8d2f30a89d3c42e49af64c754168aa4fdbd7742cea0c1bfc5402eb5d1211310d9974eff6f4a05000000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f3230343834000000687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829ffffffff0220000000554120537461636b54657374205365727665722028416e7369432f323034382900000000ffffffffffffffff01000000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f32303438eb030000308203e7308202d3a00302010202100733ad3259aa9c834b6bdc847ebc9756300906052b0e03021d05003049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f3230343829301e170d3039303831373138313633365a170d3139303831373138313633365a3049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f323034382930820122300d06092a864886f70d01010105000382010f003082010a02820101008c5922676748f8dffb65315b678d9ed77ad37322003a6bc1b7a779fc79298cccd3f52aceb9c88bb5d5b39f3ce479c1a7852d4e4b1c082ea05a863007914d0648b41e149e6ba3b2cf3d805a9eb97ac06b7d3005aab64dce3e4ed227f2615b2cc684fbf28c49d07e5087821febb0722ae5a370f041f8de3d7c395dc0ed05c2d29b6a546fd795c514c1d67e0935d6ce4a40310b23df2cb1e032c0a0c8cbb34c048820b98c661b93f63a05db5830753b1d8211382fcb5d6b88b440a03c9c83443efb137dc1d84336fad55bd87613995f2e488a80786f7f6e5269a4272f7795eb9272bfd6ef60e9952dc8bc7a0a714585ef855225275a278e3e68b2daf2a69f70dfe50203010001a381d23081cf301d0603551d0e04160414324f56e634d0b001e390d2d5055799f1ad24153c301f0603551d01041830168014324f56e634d0b001e390d2d5055799f1ad24153c300c0603551d130101ff04023000300e0603551d0f0101ff0404030202f430200603551d250101ff0416301406082b0601050507030106082b06010505070302304d0603551d07044630448634687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829820c766d2d78702d73746576656e300906052b0e03021d050003820101001c90f3f487d80ab4bef63297717a14998ab9a17362fd940b87a1ba7f1ec7eeb97878871f68099852e757f75644e5aed1670e7a25f2fa0ef0fe403b1e6b82b5dd529c8ebea9c3cb9478237064c3df5ba17bf988cf7d708e641af47f0ab32a4fed98a88425e2a147647cf88f25a6606ac30693f581c93965bc4064e391b2ead5df78869e8ce8934247a5bebb4f84c8f3bb93c3fc0781128afc0adb091537521ad306d717ebb119127a651ac83611c37f6906afcddf8f0a73e7d973fed39753a7742221b20bb013cbc276fc3fa0bc53956c5d41ab613f26af3cd69da8d2f30a89d3c42e49af64c754168aa4fdbd7742cea0c1bfc5402eb5d1211310d9974eff6f4a010000002f000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f5365637572697479506f6c696379234e6f6e6501000000010000003000000000ffffffffffffffff33000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f5365637572697479506f6c69637923426173696332353634000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f70726f66696c65732f7472616e73706f72742f756174637000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f3230343834000000687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829ffffffff0220000000554120537461636b54657374205365727665722028416e7369432f323034382900000000ffffffffffffffff01000000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f32303438eb030000308203e7308202d3a00302010202100733ad3259aa9c834b6bdc847ebc9756300906052b0e03021d05003049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f3230343829301e170d3039303831373138313633365a170d3139303831373138313633365a3049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f323034382930820122300d06092a864886f70d01010105000382010f003082010a02820101008c5922676748f8dffb65315b678d9ed77ad37322003a6bc1b7a779fc79298cccd3f52aceb9c88bb5d5b39f3ce479c1a7852d4e4b1c082ea05a863007914d0648b41e149e6ba3b2cf3d805a9eb97ac06b7d3005aab64dce3e4ed227f2615b2cc684fbf28c49d07e5087821febb0722ae5a370f041f8de3d7c395dc0ed05c2d29b6a546fd795c514c1d67e0935d6ce4a40310b23df2cb1e032c0a0c8cbb34c048820b98c661b93f63a05db5830753b1d8211382fcb5d6b88b440a03c9c83443efb137dc1d84336fad55bd87613995f2e488a80786f7f6e5269a4272f7795eb9272bfd6ef60e9952dc8bc7a0a714585ef855225275a278e3e68b2daf2a69f70dfe50203010001a381d23081cf301d0603551d0e04160414324f56e634d0b001e390d2d5055799f1ad24153c301f0603551d01041830168014324f56e634d0b001e390d2d5055799f1ad24153c300c0603551d130101ff04023000300e0603551d0f0101ff0404030202f430200603551d250101ff0416301406082b0601050507030106082b06010505070302304d0603551d07044630448634687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829820c766d2d78702d73746576656e300906052b0e03021d050003820101001c90f3f487d80ab4bef63297717a14998ab9a17362fd940b87a1ba7f1ec7eeb97878871f68099852e757f75644e5aed1670e7a25f2fa0ef0fe403b1e6b82b5dd529c8ebea9c3cb9478237064c3df5ba17bf988cf7d708e641af47f0ab32a4fed98a88425e2a147647cf88f25a6606ac30693f581c93965bc4064e391b2ead5df78869e8ce8934247a5bebb4f84c8f3bb93c3fc0781128afc0adb091537521ad306d717ebb119127a651ac83611c37f6906afcddf8f0a73e7d973fed39753a7742221b20bb013cbc276fc3fa0bc53956c5d41ab613f26af3cd69da8d2f30a89d3c42e49af64c754168aa4fdbd7742cea0c1bfc5402eb5d1211310d9974eff6f4a0300000038000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f5365637572697479506f6c696379234261736963313238527361313501000000010000003000000000ffffffffffffffffffffffff34000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f70726f66696c65732f7472616e73706f72742f756174637000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f3230343834000000687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829ffffffff0220000000554120537461636b54657374205365727665722028416e7369432f323034382900000000ffffffffffffffff01000000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f32303438eb030000308203e7308202d3a00302010202100733ad3259aa9c834b6bdc847ebc9756300906052b0e03021d05003049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f3230343829301e170d3039303831373138313633365a170d3139303831373138313633365a3049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f323034382930820122300d06092a864886f70d01010105000382010f003082010a02820101008c5922676748f8dffb65315b678d9ed77ad37322003a6bc1b7a779fc79298cccd3f52aceb9c88bb5d5b39f3ce479c1a7852d4e4b1c082ea05a863007914d0648b41e149e6ba3b2cf3d805a9eb97ac06b7d3005aab64dce3e4ed227f2615b2cc684fbf28c49d07e5087821febb0722ae5a370f041f8de3d7c395dc0ed05c2d29b6a546fd795c514c1d67e0935d6ce4a40310b23df2cb1e032c0a0c8cbb34c048820b98c661b93f63a05db5830753b1d8211382fcb5d6b88b440a03c9c83443efb137dc1d84336fad55bd87613995f2e488a80786f7f6e5269a4272f7795eb9272bfd6ef60e9952dc8bc7a0a714585ef855225275a278e3e68b2daf2a69f70dfe50203010001a381d23081cf301d0603551d0e04160414324f56e634d0b001e390d2d5055799f1ad24153c301f0603551d01041830168014324f56e634d0b001e390d2d5055799f1ad24153c300c0603551d130101ff04023000300e0603551d0f0101ff0404030202f430200603551d250101ff0416301406082b0601050507030106082b06010505070302304d0603551d07044630448634687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829820c766d2d78702d73746576656e300906052b0e03021d050003820101001c90f3f487d80ab4bef63297717a14998ab9a17362fd940b87a1ba7f1ec7eeb97878871f68099852e757f75644e5aed1670e7a25f2fa0ef0fe403b1e6b82b5dd529c8ebea9c3cb9478237064c3df5ba17bf988cf7d708e641af47f0ab32a4fed98a88425e2a147647cf88f25a6606ac30693f581c93965bc4064e391b2ead5df78869e8ce8934247a5bebb4f84c8f3bb93c3fc0781128afc0adb091537521ad306d717ebb119127a651ac83611c37f6906afcddf8f0a73e7d973fed39753a7742221b20bb013cbc276fc3fa0bc53956c5d41ab613f26af3cd69da8d2f30a89d3c42e49af64c754168aa4fdbd7742cea0c1bfc5402eb5d1211310d9974eff6f4a0200000038000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f5365637572697479506f6c696379234261736963313238527361313501000000010000003000000000ffffffffffffffffffffffff34000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f70726f66696c65732f7472616e73706f72742f756174637000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f3230343834000000687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829ffffffff0220000000554120537461636b54657374205365727665722028416e7369432f323034382900000000ffffffffffffffff01000000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f32303438eb030000308203e7308202d3a00302010202100733ad3259aa9c834b6bdc847ebc9756300906052b0e03021d05003049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f3230343829301e170d3039303831373138313633365a170d3139303831373138313633365a3049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f323034382930820122300d06092a864886f70d01010105000382010f003082010a02820101008c5922676748f8dffb65315b678d9ed77ad37322003a6bc1b7a779fc79298cccd3f52aceb9c88bb5d5b39f3ce479c1a7852d4e4b1c082ea05a863007914d0648b41e149e6ba3b2cf3d805a9eb97ac06b7d3005aab64dce3e4ed227f2615b2cc684fbf28c49d07e5087821febb0722ae5a370f041f8de3d7c395dc0ed05c2d29b6a546fd795c514c1d67e0935d6ce4a40310b23df2cb1e032c0a0c8cbb34c048820b98c661b93f63a05db5830753b1d8211382fcb5d6b88b440a03c9c83443efb137dc1d84336fad55bd87613995f2e488a80786f7f6e5269a4272f7795eb9272bfd6ef60e9952dc8bc7a0a714585ef855225275a278e3e68b2daf2a69f70dfe50203010001a381d23081cf301d0603551d0e04160414324f56e634d0b001e390d2d5055799f1ad24153c301f0603551d01041830168014324f56e634d0b001e390d2d5055799f1ad24153c300c0603551d130101ff04023000300e0603551d0f0101ff0404030202f430200603551d250101ff0416301406082b0601050507030106082b06010505070302304d0603551d07044630448634687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829820c766d2d78702d73746576656e300906052b0e03021d050003820101001c90f3f487d80ab4bef63297717a14998ab9a17362fd940b87a1ba7f1ec7eeb97878871f68099852e757f75644e5aed1670e7a25f2fa0ef0fe403b1e6b82b5dd529c8ebea9c3cb9478237064c3df5ba17bf988cf7d708e641af47f0ab32a4fed98a88425e2a147647cf88f25a6606ac30693f581c93965bc4064e391b2ead5df78869e8ce8934247a5bebb4f84c8f3bb93c3fc0781128afc0adb091537521ad306d717ebb119127a651ac83611c37f6906afcddf8f0a73e7d973fed39753a7742221b20bb013cbc276fc3fa0bc53956c5d41ab613f26af3cd69da8d2f30a89d3c42e49af64c754168aa4fdbd7742cea0c1bfc5402eb5d1211310d9974eff6f4a0300000033000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f5365637572697479506f6c69637923426173696332353601000000010000003000000000ffffffffffffffffffffffff34000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f70726f66696c65732f7472616e73706f72742f756174637000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f3230343834000000687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829ffffffff0220000000554120537461636b54657374205365727665722028416e7369432f323034382900000000ffffffffffffffff01000000370000006f70632e7463703a2f2f766d2d78702d73746576656e3a31323030312f537461636b546573745365727665722f416e7369432f32303438eb030000308203e7308202d3a00302010202100733ad3259aa9c834b6bdc847ebc9756300906052b0e03021d05003049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f3230343829301e170d3039303831373138313633365a170d3139303831373138313633365a3049311c301a060a0992268993f22c640119160c766d2d78702d73746576656e3129302706035504031320554120537461636b54657374205365727665722028416e7369432f323034382930820122300d06092a864886f70d01010105000382010f003082010a02820101008c5922676748f8dffb65315b678d9ed77ad37322003a6bc1b7a779fc79298cccd3f52aceb9c88bb5d5b39f3ce479c1a7852d4e4b1c082ea05a863007914d0648b41e149e6ba3b2cf3d805a9eb97ac06b7d3005aab64dce3e4ed227f2615b2cc684fbf28c49d07e5087821febb0722ae5a370f041f8de3d7c395dc0ed05c2d29b6a546fd795c514c1d67e0935d6ce4a40310b23df2cb1e032c0a0c8cbb34c048820b98c661b93f63a05db5830753b1d8211382fcb5d6b88b440a03c9c83443efb137dc1d84336fad55bd87613995f2e488a80786f7f6e5269a4272f7795eb9272bfd6ef60e9952dc8bc7a0a714585ef855225275a278e3e68b2daf2a69f70dfe50203010001a381d23081cf301d0603551d0e04160414324f56e634d0b001e390d2d5055799f1ad24153c301f0603551d01041830168014324f56e634d0b001e390d2d5055799f1ad24153c300c0603551d130101ff04023000300e0603551d0f0101ff0404030202f430200603551d250101ff0416301406082b0601050507030106082b06010505070302304d0603551d07044630448634687474703a2f2f766d2d78702d73746576656e2f554120537461636b54657374205365727665722028416e7369432f3230343829820c766d2d78702d73746576656e300906052b0e03021d050003820101001c90f3f487d80ab4bef63297717a14998ab9a17362fd940b87a1ba7f1ec7eeb97878871f68099852e757f75644e5aed1670e7a25f2fa0ef0fe403b1e6b82b5dd529c8ebea9c3cb9478237064c3df5ba17bf988cf7d708e641af47f0ab32a4fed98a88425e2a147647cf88f25a6606ac30693f581c93965bc4064e391b2ead5df78869e8ce8934247a5bebb4f84c8f3bb93c3fc0781128afc0adb091537521ad306d717ebb119127a651ac83611c37f6906afcddf8f0a73e7d973fed39753a7742221b20bb013cbc276fc3fa0bc53956c5d41ab613f26af3cd69da8d2f30a89d3c42e49af64c754168aa4fdbd7742cea0c1bfc5402eb5d1211310d9974eff6f4a0200000033000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f5365637572697479506f6c69637923426173696332353601000000010000003000000000ffffffffffffffffffffffff34000000687474703a2f2f6f7063666f756e646174696f6e2e6f72672f55412f70726f66696c65732f7472616e73706f72742f75617463700000000000ffffffffffffffff00004000";
        let packet = hex_stream_to_bytes(hex_stream);
        let opcua_packet = OpcuaPacket::try_from(packet.as_slice());
        assert!(
            opcua_packet.is_ok(),
            "failed to parse OPC UA CreateSessionResponse payload: {:?}",
            opcua_packet.unwrap_err()
        );
    }
}
