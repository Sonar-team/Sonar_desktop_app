// profinet.rs
use serde::Serialize;
use std::str;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ProfinetPacket {
    pub source: String,
    pub destination: String,
    // Add other relevant fields here
}

impl ProfinetPacket {
    pub fn new(packet: &[u8]) -> Option<Self> {
        // Extract the source and destination from the Profinet packet
        // Adapt this part to extract specific information from the Profinet packet

        // Example offsets and lengths (you need to replace these with actual values)
        let source_offset = 0; // Replace with actual offset
        let source_length = 6; // Replace with actual length
        let destination_offset = 6; // Replace with actual offset
        let destination_length = 6; // Replace with actual length

        // Ensure the packet has enough data
        if packet.len() < source_offset + source_length + destination_offset + destination_length {
            return None;
        }

        // Extract and convert bytes to strings
        let source = match str::from_utf8(&packet[source_offset..source_offset + source_length]) {
            Ok(s) => s.to_string(),
            Err(_) => return None,
        };

        let destination = match str::from_utf8(
            &packet[destination_offset..destination_offset + destination_length],
        ) {
            Ok(s) => s.to_string(),
            Err(_) => return None,
        };

        // Initialize other relevant fields here

        Some(ProfinetPacket {
            source,
            destination,
            // Initialize other relevant fields here
        })
    }
}
