use std::fmt;

use crate::parse::internet::Internet;

impl<'a> fmt::Display for Internet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Afficher au plus 16 octets pour éviter la saturation

        write!(
            f,
            "\n    Protocol: {}\n    Source IP: {}\n    Source IP Type: {}\n    Destination IP: {}\n    Destination IP Type: {}\n",
            self.protocol_name,
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
