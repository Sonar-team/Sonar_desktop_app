// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use pcap_file::pcap::{PcapPacket, PcapWriter};
use std::{
    fmt::{self, Write},
    fs::File,
};

/// # PacketConverter
/// Une crate pour convertir et afficher des paquets réseau en Rust.
///
/// ## Fonctionnalités :
/// - Conversion d'une chaîne hexadécimale en `Vec<u8>`
/// - Conversion d'un `Vec<u8>` en chaîne hexadécimale
/// - Affichage formaté des paquets réseau
///   Structure représentant un paquet réseau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub data: Vec<u8>,
}

impl Packet {
    pub fn packet_to_pcap(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create("output.pcap")?;

        // Configurer le PacketWriter avec les paramètres par défaut
        let writer = PcapWriter::new(file);

        let orig_len = self.data.len() as u32;
        let timestamp: std::time::Duration =
            std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?;
        let packet = PcapPacket::new(timestamp, orig_len, &self.data);

        // Ajouter les données du paquet
        writer?.write_packet(&packet)?;

        Ok(())
    }
}

/// Implémentation du trait `From<&str>` pour convertir une chaîne hexadécimale en `Packet`.
impl From<&str> for Packet {
    fn from(hex: &str) -> Self {
        Packet {
            data: hex_stream_to_bytes(hex),
        }
    }
}

/// Implémentation du trait `Display` pour afficher un paquet de manière lisible.
impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_hex_array(&self.data))
    }
}

/// Convertit un flux hexadécimal Wireshark en `Vec<u8>`.
pub fn hex_stream_to_bytes(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    assert!(
        hex.len().is_multiple_of(2),
        "La chaîne hexadécimale doit avoir une longueur paire"
    );
    for i in (0..hex.len()).step_by(2) {
        let byte_str = &hex[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16).expect("Valeur hex invalide");
        bytes.push(byte);
    }
    bytes
}

/// Convertit un slice `&[u8]` en une chaîne hexadécimale.
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut hex_string, "{byte:02X}").unwrap();
    }
    hex_string
}

/// Retourne un tableau formaté de bytes sous forme de chaîne Rust.
pub fn format_hex_array(bytes: &[u8]) -> String {
    let mut formatted = String::from("[\n");
    for (i, byte) in bytes.iter().enumerate() {
        formatted.push_str(&format!("    0x{byte:02X},"));
        if (i + 1) % 8 == 0 {
            formatted.push('\n');
        } else {
            formatted.push(' ');
        }
    }
    formatted.push_str("\n];");
    formatted
}

/// Affiche un paquet réseau de manière lisible.
pub fn display_packet(bytes: &[u8]) {
    println!("Packet: {}", format_hex_array(bytes));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_hex_stream_to_bytes() {
        let hex = "48656C6C6F"; // "Hello" in ASCII
        let expected_bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
        assert_eq!(hex_stream_to_bytes(hex), expected_bytes);
    }

    #[test]
    fn test_bytes_to_hex_string() {
        let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
        let expected_hex = "48656C6C6F";
        assert_eq!(bytes_to_hex_string(&bytes), expected_hex);
    }

    // #[test]
    // fn test_format_hex_array() {
    //     let bytes = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
    //     let expected_format = "[\n    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, \n    0x08, 0x09, \n];";
    //     assert_eq!(format_hex_array(&bytes), expected_format);
    // }

    // #[test]
    // fn test_packet_display() {
    //     let packet = Packet::from("48656C6C6F"); // "Hello"
    //     assert_eq!(packet.to_string(), "[\n    0x48, 0x65, 0x6C, 0x6C, 0x6F, \n];");
    // }

    #[test]
    fn test_packet_to_pcap() {
        let packet = Packet::from("48656C6C6F"); // "Hello"
        let result = packet.packet_to_pcap();

        // Ensure no error occurred
        assert!(result.is_ok());

        // Check if the file was created
        let pcap_path = Path::new("output.pcap");
        assert!(pcap_path.exists());

        // Cleanup the test file
        fs::remove_file(pcap_path).unwrap();
    }
}
