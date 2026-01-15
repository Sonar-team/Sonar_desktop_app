// Copyright (c) 2025 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use application::Application;
use internet::Internet;

use serde::Serialize;
use transport::Transport;

use crate::{
    DataLink,
    errors::{ParsedPacketError, internet::InternetError, transport::TransportError},
    owned::PacketFlowOwned,
    parse::transport::protocols::TransportProtocol,
};

pub mod application;
pub mod data_link;
pub mod internet;
pub mod transport;
// You can determine either a full raw packet that will return a PacketParsed struct composed of data link network transportand application layers.
// Or if you need to, you can put your payload in a determine application try from. detemines function are not dependants.

#[derive(Debug, Clone, Serialize, Eq)]
pub struct PacketFlow<'a> {
    #[serde(flatten)]
    pub data_link: DataLink<'a>,
    #[serde(flatten)]
    pub internet: Option<Internet<'a>>,
    #[serde(flatten)]
    pub transport: Option<Transport<'a>>,
    #[serde(flatten)]
    pub application: Option<Application>,
}

impl<'a> TryFrom<&'a [u8]> for PacketFlow<'a> {
    type Error = ParsedPacketError;

    fn try_from(packets: &'a [u8]) -> Result<Self, Self::Error> {
        let data_link = DataLink::try_from(packets)?;

        let mut internet = match Internet::try_from(data_link.payload) {
            Ok(internet) => Some(internet),
            Err(InternetError::UnsupportedProtocol) => None,
            Err(e) => return Err(e.into()), // ex : DataLinkError etc.
        };

        // Étape 4 : Transport
        let transport = match internet.as_mut() {
            Some(internet) => match Transport::try_from(internet.payload) {
                Ok(transport) => Some(transport),
                Err(TransportError::UnsupportedProtocol) => {
                    internet
                        .payload_protocol
                        .take() // Option<TransportProtocol> -> Option<TransportProtocol> (move)
                        .map(TransportProtocol::to_transport)
                }
                Err(e) => return Err(e.into()),
            },
            None => None,
        };

        // Étape 5 : Application
        // handle when transport is None then application is None
        let application = match &transport {
            Some(t) => match t.payload {
                Some(p) => Application::try_from(p).ok(),
                None => None,
            },
            None => None,
        };

        Ok(PacketFlow {
            data_link,
            internet,
            transport,
            application,
        })
    }
}

impl<'a> PartialEq for PacketFlow<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.data_link == other.data_link
            && self.internet == other.internet
            && self.transport == other.transport
            && self.application == other.application
    }
}

use std::hash::{Hash, Hasher};

impl<'a> Hash for PacketFlow<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_link.hash(state);
        self.internet.hash(state);
        self.transport.hash(state);
        self.application.hash(state);
    }
}

impl<'a> PacketFlow<'a> {
    pub fn to_owned(&self) -> PacketFlowOwned {
        PacketFlowOwned::from(self.clone())
    }
}
