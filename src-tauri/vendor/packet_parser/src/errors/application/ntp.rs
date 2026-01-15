// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use thiserror::Error;

/// Error types for NTP packet parsing.
#[derive(Debug, Error, PartialEq)]
pub enum NtpPacketParseError {
    #[error("Invalid NTP packet length")]
    InvalidPacketLength,

    #[error("Invalid NTP version: {version}")]
    InvalidVersion { version: u8 },

    #[error("Invalid NTP mode: {mode}")]
    InvalidMode { mode: u8 },

    #[error("Invalid stratum")]
    InvalidStratum,

    #[error("Invalid poll interval")]
    InvalidPoll,

    #[error("Failed to parse NTP timestamp")]
    InvalidTime,

    #[error("La taille du timestamp NTP est incorrecte. Attendu: 8 octets, Reçu: {received}")]
    InvalidTimestampSize { received: usize },

    #[error(
        "Erreur lors de la conversion du timestamp NTP en `DateTime<Utc>`. Unix Seconds: {seconds}, Nanos: {nanos}"
    )]
    TimestampConversionError { seconds: i64, nanos: u32 },

    #[error("NTP timestamps are not in ascending order: Originate ≤ Receive ≤ Transmit violated")]
    InconsistentTimestamps,

    #[error("Invalid Reference ID: Stratum 0 should not have a Reference ID")]
    InvalidReferenceIdForStratum0,

    #[error("Invalid Reference ID: Stratum 1 should have ASCII characters")]
    InvalidReferenceIdForStratum1,

    #[error("Invalid Reference ID: Stratum ≥ 2 should have a valid IPv4 address")]
    InvalidReferenceIdForHigherStratum,
}
