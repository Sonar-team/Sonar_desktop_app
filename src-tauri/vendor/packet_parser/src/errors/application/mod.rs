// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use serde::Serialize;
use thiserror::Error;

pub mod bitcoin;
pub mod dns;
pub mod mqtt;
pub mod ntp;

/// Errors related to parsing an `Application`
#[derive(Debug, Error, Clone, Serialize)]
pub enum ApplicationError {
    #[error("Packet is empty")]
    EmptyPacket,

    // #[error("Failed to parse Modbus packet")]
    // ModbusParseError,
    #[error("Failed to parse NTP packet")]
    NtpParseError,

    #[error("Failed to parse DNS packet")]
    DnsParseError,

    #[error("Failed to parse QUIC packet")]
    QuicParseError,

    #[error("Failed to parse Bitcoin packet")]
    BitcoinParseError,

    #[error("Failed to parse MQTT packet")]
    MqttParseError,
}
