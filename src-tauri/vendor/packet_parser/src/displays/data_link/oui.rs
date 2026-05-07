use std::fmt::{self, Display};

use crate::parse::data_link::mac_addres::oui::Oui;

/// Implements `Display` for `Oui` so it can be converted into a string.
impl Display for Oui {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Oui::ASUSTek => "ASUSTek",
            Oui::Siemens => "Siemens",
            Oui::SiemensN => "SiemensN",
            Oui::Sagemcom => "Sagemcom",
            Oui::Intel => "Intel",
            Oui::PnMc => "PnMc",
            Oui::SiemensD3 => "SiemensD3",
            Oui::Unknown => "Unknown",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {

    use crate::parse::data_link::mac_addres::oui::Oui;

    #[test]
    fn test_oui_display() {
        let test_cases = vec![
            (Oui::ASUSTek, "ASUSTek"),
            (Oui::Siemens, "Siemens"),
            (Oui::SiemensN, "SiemensN"),
            (Oui::Sagemcom, "Sagemcom"),
            (Oui::Intel, "Intel"),
            (Oui::PnMc, "PnMc"),
            (Oui::SiemensD3, "SiemensD3"),
            (Oui::Unknown, "Unknown"),
        ];

        for (oui_variant, expected_str) in test_cases {
            assert_eq!(
                oui_variant.to_string(),
                expected_str,
                "Failed for variant {:?}",
                oui_variant
            );
        }
    }
}
