// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use crate::errors::application::quic::QuicError;

/// QUIC v1 version number (RFC 9000).
pub const QUIC_V1: u32 = 1;

/// Bounds-checked cursor over a QUIC packet slice.
///
/// Controlled-extraction helper: every read validates the remaining length
/// before slicing, so the parser never indexes out of bounds and stays
/// zero-copy (all reads return sub-slices of the original packet).
#[derive(Debug, Clone, Copy)]
pub struct QuicCursor<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> QuicCursor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    /// Number of bytes left to read.
    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.offset)
    }

    /// Take `n` bytes as a borrowed sub-slice, validating bounds first.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], QuicError> {
        if self.remaining() < n {
            return Err(QuicError::Truncated {
                needed: n,
                remaining: self.remaining(),
            });
        }
        let slice = &self.buf[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    /// Take a single byte, validating bounds first.
    pub fn take_u8(&mut self) -> Result<u8, QuicError> {
        let slice = self.take(1)?;
        Ok(slice[0])
    }

    /// Take all remaining bytes as a borrowed sub-slice.
    pub fn take_rest(&mut self) -> &'a [u8] {
        let slice = &self.buf[self.offset..];
        self.offset = self.buf.len();
        slice
    }

    /// Read a QUIC variable-length integer (RFC 9000 §16) with bounds checking.
    ///
    /// The two most significant bits of the first byte encode the total
    /// length (1, 2, 4 or 8 bytes); the remaining 6 bits plus the
    /// continuation bytes form the big-endian value.
    pub fn read_varint(&mut self) -> Result<u64, QuicError> {
        let first = self.take_u8()?;
        let total_len = 1usize << (first >> 6); // 1, 2, 4 or 8
        let mut value = (first & 0b0011_1111) as u64;

        let continuation = total_len - 1;
        if self.remaining() < continuation {
            return Err(QuicError::TruncatedVarint {
                needed: continuation,
                remaining: self.remaining(),
            });
        }
        for &byte in self.take(continuation)? {
            value = (value << 8) | (byte as u64);
        }
        Ok(value)
    }
}

/// The header form bit must be 1 for a Long Header packet (RFC 9000 §17.2).
pub fn validate_long_header(header_form_long: bool) -> Result<(), QuicError> {
    if !header_form_long {
        return Err(QuicError::NotLongHeader);
    }
    Ok(())
}

/// The fixed bit must be 1 (RFC 9000 §17.2).
pub fn validate_fixed_bit(fixed_bit: bool) -> Result<(), QuicError> {
    if !fixed_bit {
        return Err(QuicError::FixedBitNotSet);
    }
    Ok(())
}

/// Only QUIC v1 is accepted.
pub fn validate_version(version: u32) -> Result<(), QuicError> {
    if version != QUIC_V1 {
        return Err(QuicError::UnsupportedVersion(version));
    }
    Ok(())
}

/// The announced payload length must fit in the remaining bytes.
pub fn validate_payload_available(available: usize, expected: usize) -> Result<(), QuicError> {
    if available < expected {
        return Err(QuicError::PayloadTooShort {
            expected,
            available,
        });
    }
    Ok(())
}

/// The Length field covers Packet Number + payload, so it must be at least
/// `packet_number_length`. Returns the payload length (Length - PN length).
pub fn validate_length_field(
    length_field: u64,
    packet_number_length: u8,
) -> Result<usize, QuicError> {
    (length_field as usize)
        .checked_sub(packet_number_length as usize)
        .ok_or(QuicError::LengthFieldTooSmall {
            length_field,
            pn_length: packet_number_length,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_take_within_bounds() {
        let data = [1u8, 2, 3, 4];
        let mut cur = QuicCursor::new(&data);
        assert_eq!(cur.take(2).unwrap(), &[1, 2]);
        assert_eq!(cur.remaining(), 2);
        assert_eq!(cur.take(2).unwrap(), &[3, 4]);
        assert_eq!(cur.remaining(), 0);
    }

    #[test]
    fn test_cursor_take_out_of_bounds() {
        let data = [1u8, 2];
        let mut cur = QuicCursor::new(&data);
        assert_eq!(
            cur.take(3),
            Err(QuicError::Truncated {
                needed: 3,
                remaining: 2
            })
        );
        // A failed take must not advance the cursor.
        assert_eq!(cur.remaining(), 2);
    }

    #[test]
    fn test_cursor_take_u8() {
        let data = [0xABu8];
        let mut cur = QuicCursor::new(&data);
        assert_eq!(cur.take_u8(), Ok(0xAB));
        assert_eq!(
            cur.take_u8(),
            Err(QuicError::Truncated {
                needed: 1,
                remaining: 0
            })
        );
    }

    #[test]
    fn test_cursor_take_rest() {
        let data = [1u8, 2, 3];
        let mut cur = QuicCursor::new(&data);
        cur.take_u8().unwrap();
        assert_eq!(cur.take_rest(), &[2, 3]);
        assert_eq!(cur.remaining(), 0);
        assert_eq!(cur.take_rest(), &[] as &[u8]);
    }

    #[test]
    fn test_read_varint_all_lengths() {
        // 1 byte: 0x25 = 37
        let mut cur = QuicCursor::new(&[0x25]);
        assert_eq!(cur.read_varint(), Ok(37));

        // 2 bytes: 0x7BBD = 15293 (RFC 9000 §A.1 example)
        let mut cur = QuicCursor::new(&[0x7B, 0xBD]);
        assert_eq!(cur.read_varint(), Ok(15293));

        // 4 bytes: 0x9D7F3E7D = 494878333 (RFC 9000 §A.1 example)
        let mut cur = QuicCursor::new(&[0x9D, 0x7F, 0x3E, 0x7D]);
        assert_eq!(cur.read_varint(), Ok(494_878_333));

        // 8 bytes: 0xC2197C5EFF14E88C = 151288809941952652 (RFC 9000 §A.1 example)
        let mut cur = QuicCursor::new(&[0xC2, 0x19, 0x7C, 0x5E, 0xFF, 0x14, 0xE8, 0x8C]);
        assert_eq!(cur.read_varint(), Ok(151_288_809_941_952_652));
    }

    #[test]
    fn test_read_varint_truncated() {
        // Empty buffer: not even the first byte.
        let mut cur = QuicCursor::new(&[]);
        assert_eq!(
            cur.read_varint(),
            Err(QuicError::Truncated {
                needed: 1,
                remaining: 0
            })
        );

        // Prefix announces 2 bytes but no continuation byte follows.
        let mut cur = QuicCursor::new(&[0x40]);
        assert_eq!(
            cur.read_varint(),
            Err(QuicError::TruncatedVarint {
                needed: 1,
                remaining: 0
            })
        );

        // Prefix announces 8 bytes but only 3 continuation bytes follow.
        let mut cur = QuicCursor::new(&[0xC0, 0x01, 0x02, 0x03]);
        assert_eq!(
            cur.read_varint(),
            Err(QuicError::TruncatedVarint {
                needed: 7,
                remaining: 3
            })
        );
    }

    #[test]
    fn test_validate_long_header() {
        assert_eq!(validate_long_header(true), Ok(()));
        assert_eq!(validate_long_header(false), Err(QuicError::NotLongHeader));
    }

    #[test]
    fn test_validate_fixed_bit() {
        assert_eq!(validate_fixed_bit(true), Ok(()));
        assert_eq!(validate_fixed_bit(false), Err(QuicError::FixedBitNotSet));
    }

    #[test]
    fn test_validate_version() {
        assert_eq!(validate_version(QUIC_V1), Ok(()));
        assert_eq!(
            validate_version(0xff00_001d),
            Err(QuicError::UnsupportedVersion(0xff00_001d))
        );
    }

    #[test]
    fn test_validate_payload_available() {
        assert_eq!(validate_payload_available(10, 10), Ok(()));
        assert_eq!(validate_payload_available(10, 4), Ok(()));
        assert_eq!(
            validate_payload_available(3, 10),
            Err(QuicError::PayloadTooShort {
                expected: 10,
                available: 3
            })
        );
    }

    #[test]
    fn test_validate_length_field() {
        assert_eq!(validate_length_field(22, 1), Ok(21));
        assert_eq!(validate_length_field(4, 4), Ok(0));
        assert_eq!(
            validate_length_field(1, 2),
            Err(QuicError::LengthFieldTooSmall {
                length_field: 1,
                pn_length: 2
            })
        );
    }
}
