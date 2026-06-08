// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

impl Display for TlsPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TLS Packet: content_type={}, version={}, length={}, payload={:02X?}",
            self.content_type, self.version, self.length, self.payload
        )
    }
}

impl Display for TlsContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TlsContentType::ChangeCipherSpec => "ChangeCipherSpec",
            TlsContentType::Alert => "Alert",
            TlsContentType::Handshake => "Handshake",
            TlsContentType::ApplicationData => "ApplicationData",
            TlsContentType::Heartbeat => "Heartbeat",
        };
        write!(f, "{}", s)
    }
}