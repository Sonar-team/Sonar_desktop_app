// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! The `Oui` module provides a representation of Organizationally Unique Identifiers (OUI).
//! An OUI is the first 3 bytes of a MAC address, used to identify the manufacturer of a device.
//!
//! # Overview
//!
//! The `Oui` enum represents known manufacturers based on their OUI values.
//! The `from_bytes` method attempts to match the first three bytes of a MAC address
//! to a known manufacturer.
//!
//! # Example
//!
//! ```rust
//! use packet_parser::parse::data_link::mac_addres::oui::Oui;
//!
//! let oui = Oui::from_bytes(&[0x2C, 0xFD, 0xA1, 0x12, 0x34, 0x56]);
//! assert_eq!(oui, Oui::ASUSTek);
//!
//! let unknown_oui = Oui::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
//! assert_eq!(unknown_oui, Oui::Unknown);
//! ```
//!
//! # Supported Manufacturers
//!
//! The following OUI values are recognized:
//!
//! | OUI Prefix      | Manufacturer |
//! |----------------|-------------|
//! | `2C:FD:A1`     | ASUSTek     |
//! | `E0:DC:A0`     | Siemens     |
//! | `B0:5B:99`     | Sagemcom    |
//! | `64:6E:E0`     | Intel Corporate |
//!
//! Any unrecognized OUI will be labeled as `Unknown`.

/// Represents an Organizationally Unique Identifier (OUI).
#[derive(Debug, PartialEq, Eq)]
pub enum Oui {
    /// ASUSTek Computer Inc.
    ASUSTek,
    /// Siemens AG.
    Siemens,

    SiemensN,

    SiemensD3,

    PnMc,
    /// Sagemcom Broadband SAS.
    Sagemcom,
    /// Intel Corporate.
    Intel,
    /// Unknown manufacturer.
    Unknown,
}

impl Oui {
    /// Creates a `Oui` instance from a MAC address byte slice.
    ///
    /// The method checks the first three bytes of the given slice to determine the manufacturer.
    /// If the OUI is not recognized, it returns `Oui::Unknown`.
    ///
    /// # Parameters
    ///
    /// - `bytes`: A slice of at least three bytes representing the MAC address.
    ///
    /// # Returns
    ///
    /// - `Oui::ASUSTek` for `2C:FD:A1`
    /// - `Oui::Siemens` for `E0:DC:A0`
    /// - `Oui::Sagemcom` for `B0:5B:99`
    /// - `Oui::Intel` for `64:6E:E0`
    /// - `Oui::Unknown` for any unrecognized prefix
    ///
    /// # Example
    ///
    /// ```
    /// use packet_parser::parse::data_link::mac_addres::oui::Oui;
    ///
    /// let oui = Oui::from_bytes(&[0x64, 0x6E, 0xE0, 0x12, 0x34, 0x56]);
    /// assert_eq!(oui, Oui::Intel);
    ///
    /// let unknown_oui = Oui::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    /// assert_eq!(unknown_oui, Oui::Unknown);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            [0x2C, 0xFD, 0xA1, ..] => Oui::ASUSTek,
            [0xE0, 0xDC, 0xA0, ..] => Oui::Siemens,
            [0x08, 0x00, 0x06, ..] => Oui::SiemensN,
            [0xB0, 0x5B, 0x99, ..] => Oui::Sagemcom,
            [0x64, 0x6E, 0xE0, ..] => Oui::Intel, // Ajout de Intel Corporate
            [0x01, 0x0E, 0xCF, ..] => Oui::PnMc,
            [0x00, 0x0E, 0x8C, ..] => Oui::SiemensD3,
            _ => Oui::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oui_asustek() {
        let bytes = [0x2C, 0xFD, 0xA1, 0x12, 0x34, 0x56];
        let oui = Oui::from_bytes(&bytes);
        assert_eq!(oui, Oui::ASUSTek);
    }

    #[test]
    fn test_oui_siemens() {
        let bytes = [0xE0, 0xDC, 0xA0, 0x12, 0x34, 0x56];
        let oui = Oui::from_bytes(&bytes);
        assert_eq!(oui, Oui::Siemens);
    }

    #[test]
    fn test_oui_sagemcom() {
        let bytes = [0xB0, 0x5B, 0x99, 0x12, 0x34, 0x56];
        let oui = Oui::from_bytes(&bytes);
        assert_eq!(oui, Oui::Sagemcom);
    }

    #[test]
    fn test_oui_intel() {
        let bytes = [0x64, 0x6E, 0xE0, 0x12, 0x34, 0x56];
        let oui = Oui::from_bytes(&bytes);
        assert_eq!(oui, Oui::Intel);
    }

    #[test]
    fn test_oui_unknown() {
        let bytes = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
        let oui = Oui::from_bytes(&bytes);
        assert_eq!(oui, Oui::Unknown);
    }
}
