// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! The `MacAddress` module provides a structured representation of MAC addresses
//! and their associated Organizationally Unique Identifier (OUI).
//!
//! # Overview
//!
//! This module includes:
//! - A `MacAddress` struct to store and manipulate MAC addresses.
//! - A method to extract the OUI from a MAC address.
//! - A method to display the MAC address with or without its OUI.
//! - A `TryFrom<&[u8]>` implementation to convert raw bytes into a `MacAddress`.
//!
//! # Example
//!
//! ```rust
//! use packet_parser::parse::data_link::mac_addres::MacAddress;
//!
//! let bytes = [0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E];
//! let mac = MacAddress::try_from(bytes.as_ref()).expect("Valid MAC address");
//!
//! println!("{}", mac.display_with_oui()); // Expected: "ASUSTek:3c:4d:5e"
//! ```
//!
//! # MAC Address Structure
//!
//! A MAC address consists of six bytes, where:
//! - The first 3 bytes represent the **Organizationally Unique Identifier (OUI)**.
//! - The last 3 bytes are assigned by the manufacturer.
//!
//! # Methods
//!
//! - `get_oui()`: Extracts the OUI from the MAC address and returns its manufacturer if known.
//! - `display_with_oui()`: Formats the MAC address, including its manufacturer if known.

use std::convert::TryFrom;

pub mod oui;
use oui::*;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{checks::data_link::validate_mac_length, errors::data_link::mac_addres::MacParseError};

/// The fixed length of a MAC address (6 bytes).
pub const MAC_LEN: usize = 6;

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// Represents a Media Access Control (MAC) address.
///
/// A MAC address is a unique identifier assigned to a network interface
/// for communications at the data link layer of a network segment.
///
/// ```mermaid
/// ---
/// title: MacAddress
/// ---
/// packet-beta
/// 0-23: "OUI bytes[3]"
/// 24-47: "NIC Specific bytes[3]"
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MacAddress(pub [u8; MAC_LEN]);

/// Sérialisée sous forme de chaîne hexadécimale ("aa:bb:cc:dd:ee:ff"),
/// comme les anciens champs `String` de `DataLink`.
impl Serialize for MacAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for MacAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let mut addr = [0u8; MAC_LEN];
        let mut parts = s.split(':');
        for byte in addr.iter_mut() {
            let part = parts
                .next()
                .ok_or_else(|| de::Error::custom("adresse MAC invalide"))?;
            *byte = u8::from_str_radix(part, 16)
                .map_err(|_| de::Error::custom("adresse MAC invalide"))?;
        }
        if parts.next().is_some() {
            return Err(de::Error::custom("adresse MAC invalide"));
        }
        Ok(Self(addr))
    }
}

impl MacAddress {
    /// Returns a formatted string of the MAC address, including its OUI if recognized.
    ///
    /// - If the OUI is recognized, the format is: `{Manufacturer}:{Last three bytes}`
    /// - Otherwise, the MAC address is displayed in standard hexadecimal notation.
    ///
    /// # Example
    ///
    /// ```
    /// use packet_parser::parse::data_link::mac_addres::MacAddress;
    ///
    /// let mac = MacAddress([0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E]);
    /// assert_eq!(mac.display_with_oui(), "ASUSTek:3c:4d:5e");
    ///
    /// let unknown_mac = MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]);
    /// assert_eq!(unknown_mac.display_with_oui(), "00:1a:2b:3c:4d:5e");
    /// ```
    pub fn display_with_oui(&self) -> String {
        match self.get_oui() {
            Oui::Unknown => format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
            ),
            oui => format!(
                "{:?}:{:02x}:{:02x}:{:02x}",
                oui, self.0[3], self.0[4], self.0[5]
            ),
        }
    }

    /// Returns the OUI (Organizationally Unique Identifier) associated with the MAC address.
    ///
    /// # Example
    ///
    /// ```
    /// use packet_parser::parse::data_link::mac_addres::MacAddress;
    ///
    /// let mac = MacAddress([0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E]);
    /// assert_eq!(mac.get_oui().to_string(), "ASUSTek");
    /// ```
    pub fn get_oui(&self) -> Oui {
        Oui::from_bytes(&self.0[0..3])
    }
}

impl TryFrom<&[u8]> for MacAddress {
    type Error = MacParseError;

    /// Converts a byte slice into a `MacAddress`, enforcing a length of 6 bytes.
    ///
    /// # Errors
    ///
    /// Returns `MacParseError::InvalidLength` if the input slice is not exactly 6 bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use packet_parser::parse::data_link::mac_addres::MacAddress;
    /// use std::convert::TryFrom;
    ///
    /// let bytes = [0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E];
    /// let mac = MacAddress::try_from(bytes.as_ref()).expect("Valid MAC address");
    ///
    /// assert_eq!(mac, MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]));
    /// ```
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        validate_mac_length(bytes)?;
        let mut addr = [0u8; MAC_LEN];
        addr.copy_from_slice(bytes);
        Ok(Self(addr))
    }
}

/// convertir une string en MacAddress
impl TryFrom<String> for MacAddress {
    type Error = MacParseError;

    /// # Errors
    ///
    /// Returns [`MacParseError::InvalidComponent`] when a component is not a
    /// valid hexadecimal byte, and [`MacParseError::InvalidLength`] when the
    /// address does not have exactly 6 components.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = value
            .split(':')
            .map(|s| u8::from_str_radix(s, 16).map_err(|_| MacParseError::InvalidComponent))
            .collect::<Result<Vec<u8>, _>>()?;
        Self::try_from(&bytes[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_mac_address_with_known_oui() {
        // Adresse MAC avec OUI correspondant à ASUSTek
        let bytes = [0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E];
        let mac = MacAddress::try_from(&bytes[..]).expect("Conversion should succeed");

        // Affichage attendu : "ASUSTek:3c:4d:5e"
        assert_eq!(mac.display_with_oui(), "ASUSTek:3c:4d:5e");
    }

    #[test]
    fn display_mac_address_with_unknown_oui() {
        // Adresse MAC avec OUI inconnu
        let bytes = [0xAA, 0xBB, 0xCC, 0x3C, 0x4D, 0x5E];
        let mac = MacAddress::try_from(&bytes[..]).expect("Conversion should succeed");

        // Affichage attendu : "aa:bb:cc:3c:4d:5e"
        assert_eq!(mac.display_with_oui(), "aa:bb:cc:3c:4d:5e");
    }

    #[test]
    fn valid_mac_address_conversion() {
        // Tableau de bytes représentant une adresse MAC valide
        let bytes = [0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E];
        let mac = MacAddress::try_from(&bytes[..]).expect("Conversion should succeed");

        // Vérification de l'égalité entre l'adresse MAC et le tableau d'origine
        assert_eq!(mac, MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]));
    }

    #[test]
    fn invalid_mac_address_length() {
        // Tableau de bytes de taille incorrecte (5 bytes)
        let bytes = [0x00, 0x1A, 0x2B, 0x3C, 0x4D];
        let result = MacAddress::try_from(&bytes[..]);

        // Vérification que l'erreur retournée correspond à InvalidLength avec la taille effective
        assert_eq!(
            result,
            Err(MacParseError::InvalidLength {
                actual: bytes.len()
            })
        );
    }

    #[test]
    fn display_mac_address_format() {
        // Adresse MAC valide pour tester le format d'affichage
        let mac = MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]);

        // Vérification de l'affichage formaté
        assert_eq!(mac.to_string(), "00:1a:2b:3c:4d:5e");
    }

    #[test]
    fn valid_mac_address_conversion_all_zeros() {
        // Adresse MAC avec tous les octets à zéro
        let bytes = [0x00; 6];
        let mac = MacAddress::try_from(&bytes[..]).expect("Conversion should succeed");

        // Vérification de l'égalité entre l'adresse MAC et le tableau d'origine
        assert_eq!(mac, MacAddress([0x00; 6]));
    }

    #[test]
    fn invalid_mac_address_format_too_long() {
        // Tableau de bytes de taille incorrecte (7 bytes)
        let bytes = [0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F];
        let result = MacAddress::try_from(&bytes[..]);

        // Vérification que l'erreur retournée correspond à InvalidLength avec la taille effective
        assert_eq!(
            result,
            Err(MacParseError::InvalidLength {
                actual: bytes.len()
            })
        );
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;

    #[test]
    fn serialize_as_hex_string() {
        let mac = MacAddress([0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E]);
        let json = serde_json::to_string(&mac).unwrap();
        assert_eq!(json, "\"2c:fd:a1:3c:4d:5e\"");
    }

    #[test]
    fn deserialize_from_hex_string() {
        let mac: MacAddress = serde_json::from_str("\"2c:fd:a1:3c:4d:5e\"").unwrap();
        assert_eq!(mac, MacAddress([0x2C, 0xFD, 0xA1, 0x3C, 0x4D, 0x5E]));
    }

    #[test]
    fn serde_round_trip() {
        let mac = MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]);
        let json = serde_json::to_string(&mac).unwrap();
        let back: MacAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(mac, back);
    }

    #[test]
    fn deserialize_rejects_invalid_strings() {
        // trop court, trop long, hex invalide, pas une chaîne
        for bad in [
            "\"2c:fd:a1:3c:4d\"",
            "\"2c:fd:a1:3c:4d:5e:6f\"",
            "\"zz:fd:a1:3c:4d:5e\"",
        ] {
            assert!(
                serde_json::from_str::<MacAddress>(bad).is_err(),
                "aurait dû échouer : {bad}"
            );
        }
        assert!(serde_json::from_str::<MacAddress>("[0,1,2,3,4,5]").is_err());
    }

    #[test]
    fn try_from_string_valid() {
        let mac = MacAddress::try_from("00:1a:2b:3c:4d:5e".to_string()).unwrap();
        assert_eq!(mac, MacAddress([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]));
    }

    #[test]
    fn try_from_string_wrong_length() {
        assert!(MacAddress::try_from("00:1a:2b".to_string()).is_err());
    }
}
