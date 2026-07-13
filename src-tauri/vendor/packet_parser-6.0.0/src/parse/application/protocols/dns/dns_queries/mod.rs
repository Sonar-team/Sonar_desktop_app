// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use crate::{
    checks::application::dns::check_dns_query_size,
    errors::application::dns::DnsQueryParseError,
    parse::application::protocols::dns::utils::{
        dns_class::DnsClass, dns_types::DnsType, name::parse_dns_name,
    },
};

#[derive(Debug, PartialEq)]
pub struct DnsQueries {
    pub queries: Vec<DnsQuery>,
}

impl DnsQueries {
    /// Parse `count` questions au début de `bytes`. Pour un message avec
    /// compression, préférer [`DnsQueries::parse`] avec le message complet,
    /// les pointeurs étant relatifs au début du message.
    pub fn from_bytes(bytes: &[u8], count: u16) -> Result<Self, DnsQueryParseError> {
        let mut offset = 0;
        Self::parse(bytes, &mut offset, count)
    }

    /// Parse `count` questions à `*offset` dans `message` (message DNS
    /// complet, en-tête inclus) et avance l'offset après la section.
    pub fn parse(
        message: &[u8],
        offset: &mut usize,
        count: u16,
    ) -> Result<Self, DnsQueryParseError> {
        let mut queries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            check_dns_query_size(message, *offset, 1)?;
            queries.push(DnsQuery::from_bytes(message, offset)?);
        }
        Ok(DnsQueries { queries })
    }
}

impl fmt::Display for DnsQueries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DnsQueries {{ queries: [")?;
        for query in &self.queries {
            write!(f, " {query},")?;
        }
        write!(f, "] }}")
    }
}

#[derive(Debug, PartialEq)]
pub struct DnsQuery {
    pub name: String,
    pub qtype: DnsType,
    pub qclass: DnsClass,
}

impl DnsQuery {
    pub fn from_bytes(bytes: &[u8], offset: &mut usize) -> Result<Self, DnsQueryParseError> {
        let (name, new_offset) = parse_dns_name(bytes, *offset)?;
        *offset = new_offset;

        check_dns_query_size(bytes, *offset, 4)?;

        let qtype = DnsType::new(u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]));
        let qclass = DnsClass::new(u16::from_be_bytes([bytes[*offset + 2], bytes[*offset + 3]]));
        *offset += 4;

        Ok(DnsQuery {
            name,
            qtype,
            qclass,
        })
    }
}

impl fmt::Display for DnsQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DnsQuery {{ name: {}, qtype: {}, qclass: {} }}",
            self.name, self.qtype, self.qclass
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::application::protocols::dns::utils::name::parse_dns_name as parse_name;

    #[test]
    fn test_parse_name() {
        let data = vec![
            0x03, 0x77, 0x77, 0x77, // "www"
            0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, // "google"
            0x03, 0x63, 0x6f, 0x6d, // "com"
            0x00, // Null terminator of the domain name
        ];
        let (name, offset) = parse_name(&data, 0).unwrap();
        assert_eq!(name, "www.google.com");
        assert_eq!(offset, 16);
    }

    #[test]
    fn test_parse_name_invalid_utf8() {
        // This data includes bytes that do not form valid UTF-8 sequences for labels.
        let data = vec![
            0x02, 0xFF, 0xFF, // Invalid UTF-8 bytes
            0x00, // Null terminator
        ];

        let result = parse_name(&data, 0);
        assert!(result.is_err());
        if let Err(DnsQueryParseError::Utf8Error(_)) = result {
            // Passed: The error is as expected.
        } else {
            panic!("Expected Utf8Error, but got {:?}", result);
        }
    }

    #[test]
    fn test_dns_query_from_bytes() {
        let data = vec![
            3, b'w', b'w', b'w', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, 0,
            1, 0, 1,
        ];
        let mut offset = 0;
        let query = DnsQuery::from_bytes(&data, &mut offset).unwrap();
        assert_eq!(query.name, "www.google.com");
        assert_eq!(query.qtype, DnsType(1));
        assert_eq!(query.qclass, DnsClass(1));
        assert_eq!(offset, 20);
    }

    #[test]
    fn test_dns_queries_from_bytes() {
        let data = vec![
            3, b'w', b'w', b'w', 6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm', 0, 0,
            1, 0, 1, 3, b'f', b'o', b'o', 3, b'b', b'a', b'r', 3, b'c', b'o', b'm', 0, 0, 2, 0, 1,
        ];
        let queries = DnsQueries::from_bytes(&data, 2).unwrap();
        assert_eq!(queries.queries.len(), 2);
        assert_eq!(queries.queries[0].name, "www.google.com");
        assert_eq!(queries.queries[0].qtype, DnsType(1));
        assert_eq!(queries.queries[0].qclass, DnsClass(1));
        assert_eq!(queries.queries[1].name, "foo.bar.com");
        assert_eq!(queries.queries[1].qtype, DnsType(2));
        assert_eq!(queries.queries[1].qclass, DnsClass(1));
    }
}
