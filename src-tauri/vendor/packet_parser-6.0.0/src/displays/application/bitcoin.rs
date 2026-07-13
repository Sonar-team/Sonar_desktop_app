// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt::{self, Display};

use crate::parse::application::protocols::bitcoin::BitcoinPacket;

impl Display for BitcoinPacket<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bitcoin Packet: magic={:02X?}, command={}, length={}, checksum={:02X?}, payload={:02X?}",
            self.magic, self.command, self.length, self.checksum, self.payload
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::application::protocols::bitcoin::BitcoinPacket;

    #[test]
    fn test_bitcoin_packet_display() {
        let packet = BitcoinPacket {
            magic: 0xF9BEB4D9,
            command: "version",
            length: 2,
            checksum: [0xAB, 0xCD, 0xEF, 0x01],
            payload: &[0xDE, 0xAD],
        };

        let rendered = packet.to_string();
        assert!(rendered.starts_with("Bitcoin Packet:"));
        assert!(rendered.contains("command=version"));
        assert!(rendered.contains("length=2"));
        assert!(rendered.contains("payload=[DE, AD]"));
    }
}
