// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

impl Display for DhcpPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DHCP Packet: op={}, htype={}, hlen={}, hops={}, xid={:08X}, secs={}, flags={}, ciaddr={:?}, yiaddr={:?}, siaddr={:?}, giaddr={:?}, chaddr={:?}, sname={:?}, file={:?}, options={:02X?}",
            self.op, self.htype, self.hlen, self.hops, self.xid, self.secs, self.flags, self.ciaddr, self.yiaddr, self.siaddr, self.giaddr, self.chaddr, self.sname, self.file, self.options
        )
    }
}
