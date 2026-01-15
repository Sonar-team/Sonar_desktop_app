// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt::{self, Display, Formatter};

use crate::parse::PacketFlow;

pub(crate) mod application;
pub(crate) mod data_link;
pub(crate) mod internet;
pub(crate) mod transport;

impl Display for PacketFlow<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "ParsedPacket :")?;
        writeln!(f, "  Data Link Layer: {}", self.data_link)?;

        if let Some(internet) = &self.internet {
            writeln!(f, "  Internet Layer: {internet}")?;
        }

        if let Some(trans) = &self.transport {
            writeln!(f, "  Transport Layer: {trans}")?;
        }
        if let Some(app) = &self.application {
            writeln!(f, "  Application Layer: {app}")?;
        }
        write!(f, "")
    }
}
