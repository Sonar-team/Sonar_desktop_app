mod dns_additional;
mod dns_answers;
mod dns_authoritative;
mod dns_header;
mod dns_queries;
pub mod utils;

use crate::errors::application::dns::DnsPacketError;
use dns_additional::AdditionalRecord;
use dns_answers::Answer;
use dns_authoritative::AuthoritativeNameServer;
use dns_header::DnsHeader;
use dns_queries::DnsQueries;
use std::fmt;

#[derive(Debug)]
pub struct DnsPacket {
    pub header: DnsHeader,
    pub queries: DnsQueries,
    pub answers: Option<Vec<Answer>>, // List of answer records
    pub authorities: Option<Vec<AuthoritativeNameServer>>, // List of authority records
    pub additionals: Option<Vec<AdditionalRecord>>, // List of additional records
}

impl TryFrom<&[u8]> for DnsPacket {
    type Error = DnsPacketError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        check_dns_minimum_size(bytes)?;

        let header = DnsHeader::try_from(bytes)?;
        let queries = DnsQueries::from_bytes(&bytes[12..], header.counts[0])?;
        let answers = None;
        let authorities = None;
        let additionals = None;

        Ok(DnsPacket {
            header,
            queries,
            answers,
            authorities,
            additionals,
        })
    }
}

fn check_dns_minimum_size(bytes: &[u8]) -> Result<(), DnsPacketError> {
    const DNS_MINIMUM_SIZE: usize = 12; // Taille minimale pour un en-tête DNS
    if bytes.len() < DNS_MINIMUM_SIZE {
        return Err(DnsPacketError::InsufficientData {
            expected: DNS_MINIMUM_SIZE,
            actual: bytes.len(),
        });
    }
    Ok(())
}

impl fmt::Display for DnsPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DnsPacket {{\n  header: {},\n  queries: {},\n  answers: {:?},\n  authorities: {:?},\n  additionals: {:?}\n}}",
            self.header, self.queries, self.answers, self.authorities, self.additionals
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_packet_parsing() {
        // Example DNS packet data
        let data = hex::decode("002b81800001000f0006000202757304706f6f6c036e7470036f72670000010001c00c0001000100000d87000443814409c00c0001000100000d870004452c393cc00c0001000100000d870004cfead1b5c00c0001000100000d870004d184b004c00c0001000100000d870004d81bb92ac00c0001000100000d87000418224f2ac00c0001000100000d870004187bcae6c00c0001000100000d8700043fa43ef9c00c0001000100000d8700044070bd0bc00c0001000100000d870004417de9cec00c0001000100000d8700044221ce05c00c0001000100000d8700044221d80bc00c0001000100000d870004425c44f6c00c0001000100000d870004426f2ec8c00c0001000100000d8700044273880404504f4f4c036e7470036f72670000020001000010d60012036e7331086d61696c776f7278036e657400c11100020001000010d6000f067573656e6574036e6574026e7a00c11100020001000010d60014067a626173656c08666f72747974776f02636800c11100020001000010d60018086176656e747572610a62686d732d67726f6570026e6c00c11100020001000010d600110e736c617274696261727466617374c18bc11100020001000010d6000f0161026e73076d61646475636bc136c12900010001000272a500044501c844c1470001000100000daf0004ca313b06").expect("Invalid hex string");

        match DnsPacket::try_from(data.as_slice()) {
            Ok(packet) => {
                // println!("{:?}", packet);
                assert_eq!(packet.header.transaction_id, 0x002b);
                assert_eq!(packet.header.flags, 0x8180);
                assert_eq!(packet.header.counts[0], 1);
                assert_eq!(packet.header.counts[1], 15);
                assert_eq!(packet.header.counts[2], 6);
                assert_eq!(packet.header.counts[3], 2);
            }
            Err(e) => panic!("Error parsing DNS packet: {}", e),
        }
    }

    #[test]
    fn test_dns_packet_parsing_return_error() {
        // Example non-DNS packet data
        let data = hex::decode("1a030aee00001bf7000014ec51ae80b7c502034c8d0e66cbc50204ecec42ee92c50204ebcf4959e6c50204ebcf4c6e6d").expect("Invalid hex string");

        match DnsPacket::try_from(data.as_slice()) {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => assert!(
                e.to_string().contains("Invalid Z field, must be 0."),
                "Unexpected error: {}",
                e
            ),
        }
    }

    #[test]
    fn test_ssl_packet_parsing_return_error() {
        // Example ssl packet data
        let data = hex::decode("8746a7014094af07a47e9b7f").expect("Invalid hex string");

        match DnsPacket::try_from(data.as_slice()) {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => assert!(
                e.to_string()
                    .contains("required 1 more bytes at offset 0, but only 0 bytes available"),
                "Unexpected error: {}",
                e
            ),
        }
    }

    #[test]
    fn test_rtcp_packet_parsing_return_error() {
        // Payload RTCP en hexadécimal
        let data = hex::decode("89cc00076f4c712d44434e53515445524d5f50494e473a3035343a3031360000")
            .expect("Invalid hex string");

        match DnsPacket::try_from(data.as_slice()) {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => assert!(
                e.to_string()
                    .contains("Invalid RCode, must be between 0 and 5"),
                "Unexpected error: {}",
                e
            ),
        }
    }

    #[test]
    fn test_check_dns_minimum_size_insufficient_data() {
        let data = vec![0; 10]; // Seulement 10 octets, donc insuffisant pour un paquet DNS
        let result = check_dns_minimum_size(&data);
        assert!(result.is_err());
        if let Err(DnsPacketError::InsufficientData { expected, actual }) = result {
            assert_eq!(expected, 12);
            assert_eq!(actual, 10);
        } else {
            panic!(
                "Expected DnsPacketError::InsufficientData, but got {:?}",
                result
            );
        }
    }

    #[test]
    fn test_check_dns_minimum_size_sufficient_data() {
        let data = vec![0; 12]; // Exactement 12 octets, ce qui est suffisant
        let result = check_dns_minimum_size(&data);
        assert!(result.is_ok());
    }
}
