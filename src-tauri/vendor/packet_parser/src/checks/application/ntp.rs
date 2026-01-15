// Copyright (c) 2024 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::net::Ipv4Addr;

use chrono::{DateTime, TimeZone, Utc};

use crate::{
    errors::application::ntp::NtpPacketParseError, parse::application::protocols::ntp::Refid,
};

const MINIMUM_NTP_PACKET_LENGTH: usize = 48;

/// ## NTP Validation Process
///
/// 1. The NTP packet must be at least **48 bytes** long.
/// 2. The **Version Number (VN)** in the first byte must be between `0` and `4`.
/// 3. The **Mode field** (first byte) must be in `[1, 2, 3, 4, 5]` (Client, Server, Broadcast, etc.).
/// 4. The **Stratum field** must be between `0` and `15` for valid servers.
/// 5. The **Timestamps** must be logically consistent.
pub fn validate_ntp_packet_length(payload: &[u8]) -> Result<(), NtpPacketParseError> {
    if payload.len() < MINIMUM_NTP_PACKET_LENGTH {
        return Err(NtpPacketParseError::InvalidPacketLength);
    }
    Ok(())
}

pub fn extract_flags(li_vn_mode: &u8) -> Result<(u8, u8, u8), NtpPacketParseError> {
    // check the ntp flag coherence in the first byte then retun the flags if it is valid

    // Extract the version (bits 3-5)
    let version = (li_vn_mode >> 3) & 0x07;

    // Extract the mode (bits 6-8)
    let mode = li_vn_mode & 0x07;

    // Check if version is between 1 and 4
    if !version.is_between(1, 4) {
        return Err(NtpPacketParseError::InvalidVersion { version });
    }

    // Check if mode is between 1 and 5
    if !mode.is_between(1, 5) {
        return Err(NtpPacketParseError::InvalidMode { mode });
    }

    // Extract Leap Indicator (LI)
    let li = (li_vn_mode >> 6) & 0b11;

    Ok((li, version, mode))
}

trait RangeExt<T> {
    fn is_between(&self, min: T, max: T) -> bool
    where
        T: PartialOrd<T>;
}

impl<T> RangeExt<T> for T {
    fn is_between(&self, min: T, max: T) -> bool
    where
        T: PartialOrd<T>,
    {
        *self >= min && *self <= max
    }
}

pub fn extract_stratum(stratum: &u8) -> Result<u8, NtpPacketParseError> {
    if *stratum > 15 {
        Err(NtpPacketParseError::InvalidStratum)
    } else {
        Ok(*stratum)
    }
}

pub fn extract_poll(poll: &u8) -> Result<u8, NtpPacketParseError> {
    if *poll > 127 {
        Err(NtpPacketParseError::InvalidPoll)
    } else {
        Ok(*poll)
    }
}

pub fn extract_precision(payload: &u8) -> Result<i8, NtpPacketParseError> {
    Ok(*payload as i8)
}

pub fn extract_root_delay(payload: &[u8]) -> Result<u32, NtpPacketParseError> {
    Ok(u32::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

pub fn extract_root_dispersion(payload: &[u8]) -> Result<u32, NtpPacketParseError> {
    Ok(u32::from_be_bytes([
        payload[0], payload[1], payload[2], payload[3],
    ]))
}

/// Extrait le Reference ID d’un paquet NTP en vérifiant le Stratum.
pub fn extract_reference_id(stratum: u8, payload: &[u8]) -> Result<Refid, NtpPacketParseError> {
    // println!("payload : {:02X?}", payload);
    if payload.len() < 4 {
        return Err(NtpPacketParseError::InvalidReferenceIdForHigherStratum);
    }

    let ref_id_bytes = [payload[0], payload[1], payload[2], payload[3]];
    let ref_str = String::from_utf8_lossy(&ref_id_bytes).to_string();

    match stratum {
        0 => {
            if ref_str == "\0\0\0\0" {
                Ok(Refid::KissCode("NULL".to_string())) // Stratum 0 peut être "NULL"
            } else if KISS_CODES.contains(&ref_str.trim()) {
                Ok(Refid::KissCode(ref_str.trim().to_string()))
            } else {
                Err(NtpPacketParseError::InvalidReferenceIdForStratum0)
            }
        }
        1 => {
            if CLOCK_SOURCES.contains(&ref_str.trim()) {
                Ok(Refid::ClockSource(ref_str.trim().to_string()))
            } else {
                Err(NtpPacketParseError::InvalidReferenceIdForStratum1)
            }
        }
        2..=15 => {
            let ip = Ipv4Addr::new(
                ref_id_bytes[0],
                ref_id_bytes[1],
                ref_id_bytes[2],
                ref_id_bytes[3],
            );
            if ip.is_unspecified() || ip.is_multicast() {
                Err(NtpPacketParseError::InvalidReferenceIdForHigherStratum)
            } else {
                Ok(Refid::Ipv4(ip))
            }
        }
        _ => Err(NtpPacketParseError::InvalidReferenceIdForHigherStratum),
    }
}

/// Liste des Kiss Codes valides
const KISS_CODES: &[&str] = &[
    "ACST", "AUTH", "AUTO", "BCST", "CRYP", "DENY", "DROP", "RSTR", "INIT", "MCST", "NKEY", "NTSN",
    "RATE", "RMOT", "STEP",
];

/// Liste des Clock Sources valides pour Stratum 1
const CLOCK_SOURCES: &[&str] = &[
    "GOES", "GPS", "GAL", "PPS", "IRIG", "WWVB", "DCF", "HBG", "MSF", "JJY", "LORC", "TDF", "CHU",
    "WWV", "WWVH", "NIST", "ACTS", "USNO", "PTB", "DFM",
];

const NTP_TO_UNIX_EPOCH: i64 = 2_208_988_800;

/// Extracts the NTP transmit timestamp from an NTP packet and returns a `DateTime<Utc>`.
pub fn extract_timestamp(payload: &[u8]) -> Result<DateTime<Utc>, NtpPacketParseError> {
    validate_epoch(payload)?;
    // Extraction des 4 premiers octets pour obtenir les secondes NTP.
    let ntp_seconds = u32::from_be_bytes(payload[0..4].try_into().unwrap());

    // Extraction des 4 derniers octets pour obtenir la fraction de seconde NTP.
    let ntp_fraction = u32::from_be_bytes(payload[4..8].try_into().unwrap());

    if ntp_seconds == 0 && ntp_fraction == 0 {
        return Ok(Utc.timestamp_opt(0, 0).unwrap()); // Retourne 1970-01-01 00:00:00 UTC
    }
    // Conversion des secondes NTP en secondes UNIX (décalage de 1900 à 1970).
    let unix_seconds = ntp_seconds as i64 - NTP_TO_UNIX_EPOCH;

    // Conversion de la fraction de seconde NTP en nanosecondes.
    let nanos = ((ntp_fraction as u64) * 1_000_000_000) / (1 << 32);
    let nanos = nanos as u32;
    // Construction du `DateTime<Utc>` et validation de la date.
    let datetime = Utc.timestamp_opt(unix_seconds, nanos).single().ok_or(
        NtpPacketParseError::TimestampConversionError {
            seconds: unix_seconds,
            nanos,
        },
    )?;

    Ok(datetime)
}

fn validate_epoch(payload: &[u8]) -> Result<(), NtpPacketParseError> {
    if payload.len() != 8 {
        return Err(NtpPacketParseError::InvalidTimestampSize {
            received: payload.len(),
        });
    }

    // Extraction des 4 premiers octets pour obtenir les secondes NTP.
    let ntp_seconds = u32::from_be_bytes(payload[0..4].try_into().unwrap());

    // Vérification si le timestamp est avant 1970
    if (ntp_seconds as i64) < 0 {
        println!("Invalid timestamp: {ntp_seconds}");
        return Err(NtpPacketParseError::InvalidTime);
    }

    Ok(())
}

pub fn validate_datetime_ordering(
    reference: DateTime<Utc>,
    originate: DateTime<Utc>,
    receive: DateTime<Utc>,
    transmit: DateTime<Utc>,
) -> Result<(), NtpPacketParseError> {
    if reference > originate || originate > receive || receive > transmit {
        return Err(NtpPacketParseError::InconsistentTimestamps);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_valid_ordering() {
        let reference = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let originate = Utc.timestamp_opt(1_700_000_001, 0).unwrap();
        let receive = Utc.timestamp_opt(1_700_000_002, 0).unwrap();
        let transmit = Utc.timestamp_opt(1_700_000_003, 0).unwrap();

        let result = validate_datetime_ordering(reference, originate, receive, transmit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_ordering() {
        let reference = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let originate = Utc.timestamp_opt(1_700_000_005, 0).unwrap();
        let receive = Utc.timestamp_opt(1_700_000_002, 0).unwrap(); // Problème ici (reçoit avant originate)
        let transmit = Utc.timestamp_opt(1_700_000_003, 0).unwrap();

        let result = validate_datetime_ordering(reference, originate, receive, transmit);
        assert!(matches!(
            result,
            Err(NtpPacketParseError::InconsistentTimestamps)
        ));
    }
    #[test]
    fn test_stratum_1_valid_ascii() {
        let stratum = 1;
        let payload = b"GPS ";
        let result = extract_reference_id(stratum, payload);
        assert_eq!(result, Ok(Refid::ClockSource("GPS".to_string())));
    }

    #[test]
    fn test_stratum_1_invalid_ascii() {
        let stratum = 1;
        let payload = [0x80, 0x00, 0x00, 0x00]; // Valeur non ASCII
        let result = extract_reference_id(stratum, &payload);
        assert!(matches!(
            result,
            Err(NtpPacketParseError::InvalidReferenceIdForStratum1)
        ));
    }

    #[test]
    fn test_stratum_2_valid_ipv4() {
        let stratum = 2;
        let payload = [8, 8, 8, 8]; // Adresse IPv4 valide (Google DNS)
        let result = extract_reference_id(stratum, &payload);
        assert_eq!(result, Ok(Refid::Ipv4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn test_stratum_2_invalid_ipv4() {
        let stratum = 2;
        let payload = [224, 0, 0, 1]; // Adresse multicast (invalide en NTP)
        let result = extract_reference_id(stratum, &payload);
        assert!(matches!(
            result,
            Err(NtpPacketParseError::InvalidReferenceIdForHigherStratum)
        ));
    }

    #[test]
    fn test_stratum_0_invalid() {
        let stratum = 0;
        let payload = [192, 168, 1, 1];
        let result = extract_reference_id(stratum, &payload);
        assert!(matches!(
            result,
            Err(NtpPacketParseError::InvalidReferenceIdForStratum0)
        ));
    }
}
