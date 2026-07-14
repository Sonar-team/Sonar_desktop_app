// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Typed views of the Differentiated Services field (RFC 2474) shared by
//! IPv4 (ToS octet) and IPv6 (Traffic Class).
//!
//! [`Dscp`] carries the 6-bit codepoint with its IANA-registered name when
//! one exists (CS0–CS7, AF11–AF43, EF, VA, LE); [`Ecn`] decodes the 2-bit
//! Explicit Congestion Notification field (RFC 3168).

use serde::Serialize;
use std::fmt;

/// Differentiated Services Code Point: the 6 high bits of the DS field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Dscp(pub u8);

impl Dscp {
    /// Builds a `Dscp` from the full DS octet (ToS / Traffic Class).
    pub const fn from_ds_field(ds: u8) -> Self {
        Self(ds >> 2)
    }

    /// IANA-registered name of the codepoint, when it has one.
    ///
    /// Covers the Class Selectors (RFC 2474), Assured Forwarding (RFC 2597),
    /// Expedited Forwarding (RFC 3246), Voice-Admit (RFC 5865) and
    /// Lower-Effort (RFC 8622) pools.
    pub const fn name(&self) -> Option<&'static str> {
        Some(match self.0 {
            0 => "CS0",
            8 => "CS1",
            16 => "CS2",
            24 => "CS3",
            32 => "CS4",
            40 => "CS5",
            48 => "CS6",
            56 => "CS7",
            10 => "AF11",
            12 => "AF12",
            14 => "AF13",
            18 => "AF21",
            20 => "AF22",
            22 => "AF23",
            26 => "AF31",
            28 => "AF32",
            30 => "AF33",
            34 => "AF41",
            36 => "AF42",
            38 => "AF43",
            44 => "VA",
            46 => "EF",
            1 => "LE",
            _ => return None,
        })
    }

    /// Raw 6-bit codepoint value.
    pub const fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Dscp {
    /// Wraps a raw 6-bit codepoint (not the full DS octet — use
    /// [`Dscp::from_ds_field`] for that).
    fn from(codepoint: u8) -> Self {
        Self(codepoint)
    }
}

impl fmt::Display for Dscp {
    /// Formats as `CS0 (0)` when the codepoint is registered, `13` otherwise.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => write!(f, "{name} ({})", self.0),
            None => write!(f, "{}", self.0),
        }
    }
}

/// Explicit Congestion Notification: the 2 low bits of the DS field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Ecn {
    /// 0b00 — Not ECN-Capable Transport.
    NotEct,
    /// 0b01 — ECN-Capable Transport, ECT(1).
    Ect1,
    /// 0b10 — ECN-Capable Transport, ECT(0).
    Ect0,
    /// 0b11 — Congestion Experienced.
    Ce,
}

impl Ecn {
    /// Builds an `Ecn` from the full DS octet (ToS / Traffic Class).
    pub const fn from_ds_field(ds: u8) -> Self {
        Self::from_bits(ds & 0x03)
    }

    /// Builds an `Ecn` from the raw 2-bit field value.
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::NotEct,
            1 => Self::Ect1,
            2 => Self::Ect0,
            _ => Self::Ce,
        }
    }

    /// Standard name of the field value (RFC 3168 terminology).
    pub const fn name(&self) -> &'static str {
        match self {
            Self::NotEct => "Not-ECT",
            Self::Ect1 => "ECT(1)",
            Self::Ect0 => "ECT(0)",
            Self::Ce => "CE",
        }
    }

    /// Raw 2-bit field value.
    pub const fn value(&self) -> u8 {
        match self {
            Self::NotEct => 0,
            Self::Ect1 => 1,
            Self::Ect0 => 2,
            Self::Ce => 3,
        }
    }
}

impl From<u8> for Ecn {
    /// Wraps the raw 2-bit field value (not the full DS octet — use
    /// [`Ecn::from_ds_field`] for that).
    fn from(bits: u8) -> Self {
        Self::from_bits(bits)
    }
}

impl fmt::Display for Ecn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dscp_named_codepoints() {
        assert_eq!(Dscp(0).name(), Some("CS0"));
        assert_eq!(Dscp(46).name(), Some("EF"));
        assert_eq!(Dscp(34).name(), Some("AF41"));
        assert_eq!(Dscp(1).name(), Some("LE"));
        assert_eq!(Dscp(13).name(), None);
    }

    #[test]
    fn dscp_display() {
        assert_eq!(Dscp(0).to_string(), "CS0 (0)");
        assert_eq!(Dscp(46).to_string(), "EF (46)");
        assert_eq!(Dscp(13).to_string(), "13");
    }

    #[test]
    fn dscp_from_ds_field_keeps_high_bits() {
        // EF (46) with CE: DS octet = 0b1011_1011.
        assert_eq!(Dscp::from_ds_field(0xBB), Dscp(46));
    }

    #[test]
    fn ecn_from_bits_covers_all_values() {
        assert_eq!(Ecn::from_bits(0), Ecn::NotEct);
        assert_eq!(Ecn::from_bits(1), Ecn::Ect1);
        assert_eq!(Ecn::from_bits(2), Ecn::Ect0);
        assert_eq!(Ecn::from_bits(3), Ecn::Ce);
    }

    #[test]
    fn ecn_display_uses_rfc_names() {
        assert_eq!(Ecn::NotEct.to_string(), "Not-ECT");
        assert_eq!(Ecn::Ect0.to_string(), "ECT(0)");
        assert_eq!(Ecn::from_ds_field(0xBB).to_string(), "CE");
    }

    #[test]
    fn roundtrip_values() {
        for bits in 0..=3u8 {
            assert_eq!(Ecn::from_bits(bits).value(), bits);
        }
        assert_eq!(Dscp(46).value(), 46);
    }
}
