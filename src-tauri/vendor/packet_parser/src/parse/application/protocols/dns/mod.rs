// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

mod dns_additional;
mod dns_answers;
mod dns_authoritative;
mod dns_header;
mod dns_queries;
pub mod utils;

use crate::{
    checks::application::dns::check_dns_minimum_size, errors::application::dns::DnsPacketError,
};
use dns_additional::AdditionalRecord;
use dns_answers::Answer;
use dns_authoritative::AuthoritativeNameServer;
use dns_header::DnsHeader;
use dns_queries::DnsQueries;
use std::fmt;
use utils::name::{RawRecord, parse_resource_record};

#[cfg_attr(all(doc, feature = "doc-diagrams"), aquamarine::aquamarine)]
/// DNS Packet
///
/// ```mermaid
/// ---
/// title: DnsPacket
/// ---
/// packet-beta
/// 0-15: "Transaction ID u16"
/// 16-31: "Flags u16"
/// 32-47: "Question Count u16"
/// 48-63: "Answer Count u16"
/// 64-79: "Authority Count u16"
/// 80-95: "Additional Count u16"
/// 96-159: "Questions variable"
/// 160-223: "Answers / Authority / Additional variable"
/// ```
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

        // Les sections sont parsées sur le message complet : les pointeurs de
        // compression (RFC 1035 §4.1.4) sont des offsets depuis l'octet 0.
        let mut offset = 12;
        let queries = DnsQueries::parse(bytes, &mut offset, header.counts[0])?;
        let answers = parse_record_section::<Answer>(bytes, &mut offset, header.counts[1])?;
        let authorities =
            parse_record_section::<AuthoritativeNameServer>(bytes, &mut offset, header.counts[2])?;
        let additionals =
            parse_record_section::<AdditionalRecord>(bytes, &mut offset, header.counts[3])?;

        Ok(DnsPacket {
            header,
            queries,
            answers,
            authorities,
            additionals,
        })
    }
}

/// Parse `count` resource records à `*offset` dans `message`. Retourne `None`
/// quand la section est vide.
fn parse_record_section<T: From<RawRecord>>(
    message: &[u8],
    offset: &mut usize,
    count: u16,
) -> Result<Option<Vec<T>>, DnsPacketError> {
    if count == 0 {
        return Ok(None);
    }
    let mut records = Vec::with_capacity(count as usize);
    for _ in 0..count {
        records.push(T::from(parse_resource_record(message, offset)?));
    }
    Ok(Some(records))
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

                assert_eq!(packet.queries.queries[0].name, "us.pool.ntp.org");

                // 15 réponses A, noms compressés (pointeur c00c vers la question).
                let answers = packet.answers.as_ref().expect("answers");
                assert_eq!(answers.len(), 15);
                assert_eq!(answers[0].name, "us.pool.ntp.org");
                assert_eq!(answers[0].answer_type.0, 1); // A
                assert_eq!(answers[0].answer_class.0, 1); // IN
                assert_eq!(answers[0].ttl, 0x0d87);
                assert_eq!(answers[0].data_length, 4);
                assert_eq!(answers[0].address, vec![0x43, 0x81, 0x44, 0x09]);
                assert_eq!(answers[14].address, vec![0x42, 0x73, 0x88, 0x04]);

                // 6 enregistrements NS, avec compression dans le nom ET la rdata.
                let authorities = packet.authorities.as_ref().expect("authorities");
                assert_eq!(authorities.len(), 6);
                assert_eq!(authorities[0].name, "POOL.ntp.org");
                assert_eq!(authorities[0].answer_type.0, 2); // NS

                // 2 additionnels (A des serveurs de noms).
                let additionals = packet.additionals.as_ref().expect("additionals");
                assert_eq!(additionals.len(), 2);
                assert_eq!(additionals[0].answer_type.0, 1); // A
                assert_eq!(additionals[0].data_length, 4);
            }
            Err(e) => panic!("Error parsing DNS packet: {}", e),
        }
    }

    #[test]
    fn test_dns_query_only_packet_has_empty_sections() {
        // Question simple "example.com A IN", aucun record.
        let mut data = vec![
            0x12, 0x34, // transaction id
            0x01, 0x00, // flags : requête standard, RD
            0x00, 0x01, // 1 question
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0 answer/authority/additional
        ];
        data.push(7);
        data.extend_from_slice(b"example");
        data.push(3);
        data.extend_from_slice(b"com");
        data.push(0);
        data.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

        let packet = DnsPacket::try_from(data.as_slice()).unwrap();

        assert_eq!(packet.queries.queries[0].name, "example.com");
        assert!(packet.answers.is_none());
        assert!(packet.authorities.is_none());
        assert!(packet.additionals.is_none());
    }

    #[test]
    fn test_dns_truncated_answer_section_is_rejected() {
        // En-tête annonçant 1 réponse, mais section absente.
        let mut data = vec![
            0x12, 0x34, // transaction id
            0x81, 0x80, // flags : réponse standard
            0x00, 0x01, // 1 question
            0x00, 0x01, // 1 answer annoncée
            0x00, 0x00, 0x00, 0x00,
        ];
        data.push(1);
        data.push(b'a');
        data.push(0);
        data.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

        let result = DnsPacket::try_from(data.as_slice());

        assert!(result.is_err(), "truncated answer section must not parse");
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
                    .contains("required 1 more bytes at offset 12, but only 0 bytes available"),
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
