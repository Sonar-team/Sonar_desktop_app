use thiserror::Error;

#[derive(Debug, Error)]
pub enum DnsPacketError {
    #[error("Insufficient data: expected at least {expected} bytes, but got {actual}")]
    InsufficientData { expected: usize, actual: usize },
    #[error("DNS header parsing error: {0}")]
    HeaderError(#[from] DnsHeaderError),
    #[error("DNS Query parsing error: {0}")]
    QueryError(#[from] DnsQueryParseError),
}

#[derive(Debug, Error)]
pub enum DnsHeaderError {
    #[error("Packet too short to be a DNS packet")]
    PacketTooShort,
    #[error("Invalid DNS packet: non-zero resource record counts with zero questions")]
    InvalidCounts,
    #[error("DNS Flags parsing error: {0}")]
    FlagsError(#[from] DnsFlagsError),
}

#[derive(Debug, Error)]
pub enum DnsQueryParseError {
    #[error(
        "Insufficient data: required {required} more bytes at offset {offset}, but only {available} bytes available"
    )]
    InsufficientData {
        required: usize,
        offset: usize,
        available: usize,
    },
    #[error("Out of bound parse")]
    OutOfBoundParse,
    #[error("UTF-8 parsing error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, Error, PartialEq)]
pub enum DnsFlagsError {
    #[error("Invalid Z field, must be 0. Here it's: {0}")]
    InvalidZField(u16),

    #[error("Invalid Opcode, must be between 0 and 5. Here it's: {0}")]
    InvalidOpcode(u16),

    #[error("Invalid RCode, must be between 0 and 5. Here it's: {0}")]
    InvalidRCode(u16),

    #[error("RA must be 0 in queries. Here it's: {0}")]
    RaInQuery(u16),

    #[error("AA and TC must be 0 in STATUS responses. Here AA is: {0}, TC is: {1}")]
    AaTcInStatusResponse(u16, u16),

    #[error("Rcode = 2, so AA must be 0 in Server failure responses. Here it's: {0}")]
    AaInServerFailure(u16),

    #[error("Rcode = 3, AA must be 1 in Name Error responses. Here it's: {0}")]
    AaInNameError(u16),

    #[error("Rcode = 5, AA must be 0 in Refused responses. Here it's: {0}")]
    AaInRefused(u16),
}
