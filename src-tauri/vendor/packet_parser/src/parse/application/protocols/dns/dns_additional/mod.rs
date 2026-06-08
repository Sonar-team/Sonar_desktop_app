use std::fmt;

use crate::parse::application::protocols::dns::utils::{dns_class::DnsClass, dns_types::DnsType};

#[derive(Debug)]
pub struct AdditionalRecord {
    name: String,           // Domain name
    answer_type: DnsType,   // Type of record
    answer_class: DnsClass, // Class of record
    ttl: u32,               // Time to live
    data_length: u16,       // Length of the data
    address: Vec<u8>,       // Address or other data (variable length)
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
