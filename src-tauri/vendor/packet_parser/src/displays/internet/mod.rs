use std::fmt;

use crate::parse::internet::Internet;

impl<'a> fmt::Display for Internet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n    Protocol: {}\n    Payload Protocol: {}\n    Source IP: {}\n    Source IP Type: {}\n    Destination IP: {}\n    Destination IP Type: {}\n",
            self.protocol_name,
            self.payload_protocol
                .as_ref()
                .map(|proto| format!("{proto:?}"))
                .unwrap_or_else(|| "None".to_string()),
            self.source
                .as_ref()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "None".to_string()),
            self.source_type
                .as_ref()
                .map(|ip_type| ip_type.to_string())
                .unwrap_or_else(|| "None".to_string()),
            self.destination
                .as_ref()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| "None".to_string()),
            self.destination_type
                .as_ref()
                .map(|ip_type| ip_type.to_string())
                .unwrap_or_else(|| "None".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{IpType, parse::transport::protocols::TransportProtocol};

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_display_internet_with_all_fields() {
        let internet = Internet {
            protocol_name: "IPv4".to_string(),
            payload_protocol: Some(TransportProtocol::Tcp),
            source: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))),
            source_type: Some(IpType::Private),
            destination: Some(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            destination_type: Some(IpType::Private),
            payload: &[],
        };

        let rendered = format!("{internet}");

        let expected = "\n    Protocol: IPv4\n    Payload Protocol: Tcp\n    Source IP: 192.168.1.10\n    Source IP Type: Privée\n    Destination IP: 8.8.8.8\n    Destination IP Type: Privée\n";

        assert_eq!(rendered, expected);
    }

    #[test]
    fn test_display_internet_with_none_fields() {
        let internet = Internet {
            protocol_name: "IPv4".to_string(),
            payload_protocol: None,
            source: None,
            source_type: None,
            destination: None,
            destination_type: None,
            payload: &[],
        };

        let rendered = format!("{internet}");

        let expected = "\n    Protocol: IPv4\n    Payload Protocol: None\n    Source IP: None\n    Source IP Type: None\n    Destination IP: None\n    Destination IP Type: None\n";

        assert_eq!(rendered, expected);
    }
}
