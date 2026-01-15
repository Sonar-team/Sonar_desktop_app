use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BitcoinError {
    PacketTooShort {
        actual: usize,
    },
    InvalidMagic {
        magic: u32,
    },
    InvalidCommandBytes,     // non ASCII alphanum or bad padding
    NonZeroPaddingAfterNull, // strict: once '\0' starts, rest must be '\0'
    LengthMismatch {
        declared: u32,
        actual_payload_len: usize,
        actual_total_len: usize,
    },
}

impl fmt::Display for BitcoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinError::PacketTooShort { actual } => {
                write!(f, "Bitcoin packet too short: {actual} bytes (min 24)")
            }
            BitcoinError::InvalidMagic { magic } => {
                write!(f, "Invalid Bitcoin magic: 0x{magic:08X}")
            }
            BitcoinError::InvalidCommandBytes => {
                write!(
                    f,
                    "Invalid Bitcoin command field (must be ASCII alphanumeric + null padding)"
                )
            }
            BitcoinError::NonZeroPaddingAfterNull => {
                write!(
                    f,
                    "Invalid Bitcoin command padding (non-zero after null terminator)"
                )
            }
            BitcoinError::LengthMismatch {
                declared,
                actual_payload_len,
                actual_total_len,
            } => write!(
                f,
                "Bitcoin length mismatch: header declares {declared}, actual payload {actual_payload_len}, total packet {actual_total_len}"
            ),
        }
    }
}
