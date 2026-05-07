// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt::{self, Display};

use crate::parse::application::protocols::bitcoin::BitcoinPacket;

impl Display for BitcoinPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bitcoin Packet: magic={:02X?}, command={}, length={}, checksum={:02X?}, payload={:02X?}",
            self.magic, self.command, self.length, self.checksum, self.payload
        )
    }
}
