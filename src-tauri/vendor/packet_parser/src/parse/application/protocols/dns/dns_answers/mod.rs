use std::fmt;

use crate::parse::application::protocols::dns::utils::{dns_class::DnsClass, dns_types::DnsType};

// more can be a list of this possible struct (those strcut may on may not be on the liste: "more"):
#[derive(Debug)]
pub struct Answer {
    name: String,           // Domain name
    answer_type: DnsType,   // Type of record (e.g., A, AAAA, MX, etc.)
    answer_class: DnsClass, // Class of record (typically IN for Internet)
    ttl: u32,               // Time to live
    data_length: u16,       // Length of the data
    address: Vec<u8>,       // Address or other data (variable length)
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Answer {{ name: {}, answer_type: {}, answer_class: {}, ttl: {}, data_length: {}, address: {:?} }}",
            self.name,
            self.answer_type,
            self.answer_class,
            self.ttl,
            self.data_length,
            self.address
        )
    }
}
