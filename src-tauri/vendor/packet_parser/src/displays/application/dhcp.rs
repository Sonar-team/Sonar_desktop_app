// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt::{self, Display};

use crate::parse::application::protocols::dhcp::DhcpPacket;

impl Display for DhcpPacket<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DHCP Packet: op={}, htype={}, hlen={}, hops={}, xid={:08X}, secs={}, flags={}, ciaddr={:?}, yiaddr={:?}, siaddr={:?}, giaddr={:?}, chaddr={:02X?}, sname={:02X?}, file={:02X?}, options={:02X?}",
            self.op,
            self.htype,
            self.hlen,
            self.hops,
            self.xid,
            self.secs,
            self.flags,
            self.ciaddr,
            self.yiaddr,
            self.siaddr,
            self.giaddr,
            self.chaddr,
            self.sname,
            self.file,
            self.options
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::application::protocols::dhcp::DhcpPacket;

    #[test]
    fn test_dhcp_packet_display_fields() {
        let chaddr = [0u8; 16];
        let sname = [0u8; 64];
        let file = [0u8; 128];
        let options = [0x63u8, 0x82, 0x53, 0x63, 0xFF];

        let packet = DhcpPacket {
            op: 2,
            htype: 1,
            hlen: 6,
            hops: 0,
            xid: 0xDEADBEEF,
            secs: 1,
            flags: 0x8000,
            ciaddr: [192, 168, 1, 10],
            yiaddr: [0, 0, 0, 0],
            siaddr: [0, 0, 0, 0],
            giaddr: [0, 0, 0, 0],
            chaddr: &chaddr,
            sname: &sname,
            file: &file,
            options: &options,
        };

        let rendered = packet.to_string();
        assert!(rendered.starts_with("DHCP Packet:"));
        assert!(rendered.contains("op=2"));
        assert!(rendered.contains("xid=DEADBEEF"));
        assert!(rendered.contains("ciaddr=[192, 168, 1, 10]"));
        assert!(rendered.contains("options=[63, 82, 53, 63, FF]"));
    }
}
