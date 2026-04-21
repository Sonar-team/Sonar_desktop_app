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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_too_short_display() {
        let err = BitcoinError::PacketTooShort { actual: 12 };

        assert_eq!(
            err.to_string(),
            "Bitcoin packet too short: 12 bytes (min 24)"
        );
    }

    #[test]
    fn test_invalid_magic_display() {
        let err = BitcoinError::InvalidMagic { magic: 0xD9B4BEF9 };

        assert_eq!(err.to_string(), "Invalid Bitcoin magic: 0xD9B4BEF9");
    }

    #[test]
    fn test_invalid_command_bytes_display() {
        let err = BitcoinError::InvalidCommandBytes;

        assert_eq!(
            err.to_string(),
            "Invalid Bitcoin command field (must be ASCII alphanumeric + null padding)"
        );
    }

    #[test]
    fn test_non_zero_padding_after_null_display() {
        let err = BitcoinError::NonZeroPaddingAfterNull;

        assert_eq!(
            err.to_string(),
            "Invalid Bitcoin command padding (non-zero after null terminator)"
        );
    }

    #[test]
    fn test_length_mismatch_display() {
        let err = BitcoinError::LengthMismatch {
            declared: 100,
            actual_payload_len: 96,
            actual_total_len: 120,
        };

        assert_eq!(
            err.to_string(),
            "Bitcoin length mismatch: header declares 100, actual payload 96, total packet 120"
        );
    }

    #[test]
    fn test_packet_too_short_equality() {
        let left = BitcoinError::PacketTooShort { actual: 10 };
        let right = BitcoinError::PacketTooShort { actual: 10 };

        assert_eq!(left, right);
    }

    #[test]
    fn test_invalid_magic_equality() {
        let left = BitcoinError::InvalidMagic { magic: 0x12345678 };
        let right = BitcoinError::InvalidMagic { magic: 0x12345678 };

        assert_eq!(left, right);
    }

    #[test]
    fn test_length_mismatch_equality() {
        let left = BitcoinError::LengthMismatch {
            declared: 42,
            actual_payload_len: 40,
            actual_total_len: 64,
        };
        let right = BitcoinError::LengthMismatch {
            declared: 42,
            actual_payload_len: 40,
            actual_total_len: 64,
        };

        assert_eq!(left, right);
    }

    #[test]
    fn test_clone_keeps_same_value() {
        let err = BitcoinError::InvalidMagic { magic: 0xAABBCCDD };
        let cloned = err.clone();

        assert_eq!(err, cloned);
    }

    #[test]
    fn test_debug_contains_variant_name() {
        let err = BitcoinError::InvalidCommandBytes;
        let debug = format!("{err:?}");

        assert!(debug.contains("InvalidCommandBytes"));
    }
}
