// Copyright (c) 2026 Cyprien Avico avicocyprien@yahoo.com
//
// Licensed under the MIT License <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::fmt;

use crate::parse::application::protocols::dns::utils::{
    dns_class::DnsClass, dns_types::DnsType, name::RawRecord,
};

#[derive(Debug)]
pub struct AdditionalRecord {
    pub name: String,           // Domain name
    pub answer_type: DnsType,   // Type of record
    pub answer_class: DnsClass, // Class of record
    pub ttl: u32,               // Time to live
    pub data_length: u16,       // Length of the data
    pub address: Vec<u8>,       // Address or other data (variable length)
}

impl From<RawRecord> for AdditionalRecord {
    fn from(record: RawRecord) -> Self {
        AdditionalRecord {
            name: record.name,
            answer_type: DnsType::new(record.rtype),
            answer_class: DnsClass::new(record.rclass),
            ttl: record.ttl,
            data_length: record.data_length,
            address: record.data,
        }
    }
}

impl fmt::Display for AdditionalRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AdditionalRecord {{ name: {}, answer_type: {}, answer_class: {}, ttl: {}, data_length: {}, address: {:?} }}",
            self.name,
            self.answer_type,
            self.answer_class,
            self.ttl,
            self.data_length,
            self.address
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::application::protocols::dns::utils::{
        dns_class::DnsClasses, dns_types::DnsTypes,
    };

    #[test]
    fn test_display() {
        let record = AdditionalRecord {
            name: "example.com".to_string(),
            answer_type: DnsTypes::A,
            answer_class: DnsClasses::IN,
            ttl: 300,
            data_length: 4,
            address: vec![93, 184, 216, 34],
        };

        let rendered = record.to_string();
        assert!(rendered.starts_with("AdditionalRecord {"));
        assert!(rendered.contains("name: example.com"));
        assert!(rendered.contains("answer_type: A"));
        assert!(rendered.contains("answer_class: IN"));
        assert!(rendered.contains("ttl: 300"));
        assert!(rendered.contains("data_length: 4"));
        assert!(rendered.contains("address: [93, 184, 216, 34]"));
    }
}
