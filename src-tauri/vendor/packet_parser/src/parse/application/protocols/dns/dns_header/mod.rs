// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use crate::{
    checks::application::dns::{check_packet_length, validate_and_parse_count, verify_dns_flags},
    errors::application::dns::DnsHeaderError,
};

#[derive(Debug)]
pub struct DnsHeader {
    pub transaction_id: u16,
    pub flags: u16,
    pub counts: [u16; 4], // questions_count, answers_count, authorities_count, additionals_count
}

impl TryFrom<&[u8]> for DnsHeader {
    type Error = DnsHeaderError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        check_packet_length(bytes)?;

        let transaction_id = u16::from_be_bytes([bytes[0], bytes[1]]);
        // println!("transaction_id: {}", transaction_id);
        let flags = verify_dns_flags(u16::from_be_bytes([bytes[2], bytes[3]]))?;
        // println!("flags: {}", flags);
        let counts = validate_and_parse_count(&bytes[4..12])?;
        // println!("transaction_id: {}, flags: {}, counts: {:?}", transaction_id, flags, counts);
        Ok(Self {
            transaction_id,
            flags,
            counts,
        })
    }
}

impl fmt::Display for DnsHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DnsHeader {{ transaction_id: {}, flags: {}, questions_count: {}, answers_count: {}, authorities_count: {}, additionals_count: {} }}",
            self.transaction_id,
            self.flags,
            self.counts[0],
            self.counts[1],
            self.counts[2],
            self.counts[3],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_packet_length() {
        let short_data = vec![0; 11];
        assert!(check_packet_length(&short_data).is_err());

        let valid_data = vec![0; 12];
        assert!(check_packet_length(&valid_data).is_ok());
    }

    #[test]
    fn test_validate_and_parse_count() {
        let valid_data = vec![0, 1, 0, 2, 0, 3, 0, 4];
        let counts = validate_and_parse_count(&valid_data).unwrap();
        assert_eq!(counts, [1, 2, 3, 4]);

        let invalid_data = vec![0, 0, 0, 1, 0, 0, 0, 0];
        assert!(validate_and_parse_count(&invalid_data).is_err());
    }

    #[test]
    fn test_validate_and_parse_count_with_zero_questions() {
        let invalid_data = vec![0, 0, 0, 1, 0, 1, 0, 1];
        let result = validate_and_parse_count(&invalid_data);
        assert!(
            result.is_err(),
            "Expected an error due to zero questions and non-zero resource records"
        );
    }

    #[test]
    fn test_dns_header_try_from() {
        let data = vec![0, 1, 0, 2, 0, 1, 0, 2, 0, 3, 0, 4];
        let header = DnsHeader::try_from(&data[..]).unwrap();
        assert_eq!(header.transaction_id, 1);
        assert_eq!(header.flags, 2);
        assert_eq!(header.counts, [1, 2, 3, 4]);

        let invalid_data = vec![0, 1, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0];
        assert!(DnsHeader::try_from(&invalid_data[..]).is_err());
    }

    #[test]
    fn test_dns_header_with_zero_questions() {
        let invalid_data = vec![0, 1, 0, 2, 0, 0, 0, 1, 0, 1, 0, 1];
        let result = DnsHeader::try_from(&invalid_data[..]);
        assert!(
            result.is_err(),
            "Expected an error due to zero questions and non-zero resource records"
        );
    }
}
