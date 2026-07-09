use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, path::BaseDirectory};

use crate::{
    errors::CaptureStateError,
    state::{flow_matrix::FlowMatrix, labels_list::PcInfoLabel},
};

pub fn read_labels(app: &AppHandle) -> Result<(), tauri::Error> {
    let resource_path = app
        .path()
        .resolve("resources/labels.csv", BaseDirectory::Resource)?;
    println!("resource_path: {:?}", resource_path);
    // read in file and display :
    let csv_data = std::fs::read_to_string(resource_path.clone())?;
    println!("{}", csv_data);
    Ok(())
}

pub fn create_labels_from_network_interfaces(
    interfaces: Vec<netdev::Interface>,
    app: &AppHandle,
) -> Result<(), CaptureStateError> {
    let state_label = app.state::<Arc<Mutex<PcInfoLabel>>>();
    let mut pcinfo = state_label.lock()?;
    const LABEL_NAME: &str = "pc sonar";

    for interface in interfaces {
        let Some(mac_addr) = interface.mac_addr else {
            continue;
        };

        let mac_addr = mac_addr.to_string();

        for ipv4 in interface.ipv4_addrs() {
            pcinfo.push(format!("{mac_addr},{ipv4},{LABEL_NAME}"));
        }

        for ipv6 in interface.ipv6_addrs() {
            pcinfo.push(format!("{mac_addr},{ipv6},{LABEL_NAME}"));
        }
    }

    Ok(())
}

pub fn update_labels_in_state(
    app: &AppHandle,
    state_label: &mut FlowMatrix,
) -> Result<(), CaptureStateError> {
    let pcinfo = app.state::<Arc<Mutex<PcInfoLabel>>>();
    let pcinfo = pcinfo.lock()?;

    for label in pcinfo.get_label() {
        let Some((mac, ip, label_name)) = parse_label_row(label) else {
            continue;
        };

        state_label.add_label(mac.to_string(), ip, label_name);
    }

    Ok(())
}

pub fn parse_label_row(row: &str) -> Option<(String, String, String)> {
    parse_label_fields(row.split(','))
}

pub fn parse_label_fields<I, S>(fields: I) -> Option<(String, String, String)>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let parts: Vec<String> = fields
        .into_iter()
        .map(|value| clean_csv_field(value.as_ref()).to_string())
        .collect();

    if parts.len() < 3 {
        return None;
    }

    let mac = &parts[0];
    let ip = &parts[1];
    let label_parts = &parts[2..];

    // Une ligne doit porter au moins une adresse (MAC ou IP) pour être exploitable.
    if mac.is_empty() && ip.is_empty() {
        return None;
    }

    // Une IP en notation CIDR ("192.168.1.0/24") est ramenée à sa seule adresse.
    let ip = ip.split('/').next().unwrap_or(ip);
    let label = label_parts
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let label = if label.is_empty() {
        "Label?".to_string()
    } else {
        label
    };

    Some((mac.to_string(), ip.to_string(), label))
}

pub fn clean_csv_field(value: &str) -> &str {
    value.trim().trim_matches('"')
}

#[cfg(test)]
mod tests {
    use super::parse_label_row;
    use netdev::Interface;
    use netdev::ipnet::{Ipv4Net, Ipv6Net};
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn build_label_rows(interfaces: Vec<netdev::Interface>) -> Vec<String> {
        const LABEL_NAME: &str = "pc sonar";
        let mut rows = Vec::new();

        for interface in interfaces {
            let Some(mac_addr) = interface.mac_addr else {
                continue;
            };
            let mac_addr = mac_addr.to_string();

            for ipv4 in interface.ipv4_addrs() {
                rows.push(format!("{mac_addr},{ipv4},{LABEL_NAME}"));
            }
            for ipv6 in interface.ipv6_addrs() {
                rows.push(format!("{mac_addr},{ipv6},{LABEL_NAME}"));
            }
        }

        rows
    }

    #[test]
    fn creates_one_row_per_ip_address() {
        let mut interface = Interface::dummy();
        interface.mac_addr = Some("aa:bb:cc:dd:ee:ff".parse().unwrap());
        interface.ipv4 = vec![Ipv4Net::new(Ipv4Addr::new(192, 168, 1, 10), 24).unwrap()];
        interface.ipv6 =
            vec![Ipv6Net::new("2001:db8::10".parse::<Ipv6Addr>().unwrap(), 64).unwrap()];

        let labels = build_label_rows(vec![interface]);

        assert_eq!(
            labels,
            vec![
                "aa:bb:cc:dd:ee:ff,192.168.1.10,pc sonar".to_string(),
                "aa:bb:cc:dd:ee:ff,2001:db8::10,pc sonar".to_string(),
            ]
        );
    }

    #[test]
    fn skips_interfaces_without_mac_address() {
        let mut interface = Interface::dummy();
        interface.ipv4 = vec![Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 5), 24).unwrap()];

        let labels = build_label_rows(vec![interface]);

        assert!(labels.is_empty());
    }

    #[test]
    fn parses_label_row_into_mac_ip_and_label() {
        let parsed = parse_label_row("aa:bb:cc:dd:ee:ff,192.168.1.10,pc sonar");

        assert_eq!(
            parsed,
            Some((
                "aa:bb:cc:dd:ee:ff".to_string(),
                "192.168.1.10".to_string(),
                "pc sonar".to_string(),
            ))
        );
    }

    #[test]
    fn rejects_invalid_label_rows() {
        assert_eq!(parse_label_row("aa:bb:cc:dd:ee:ff,192.168.1.10"), None);
        assert_eq!(parse_label_row(",,pc sonar"), None);
    }

    #[test]
    fn parses_ip_only_label_row() {
        let parsed = parse_label_row(",8.8.8.8,google.com");

        assert_eq!(
            parsed,
            Some((
                String::new(),
                "8.8.8.8".to_string(),
                "google.com".to_string(),
            ))
        );
    }

    #[test]
    fn parses_quoted_ip_only_label_row() {
        let parsed = parse_label_row(",\"8.8.8.8\", \"google.com\"");

        assert_eq!(
            parsed,
            Some((
                String::new(),
                "8.8.8.8".to_string(),
                "google.com".to_string(),
            ))
        );
    }

    #[test]
    fn parses_trailing_empty_column() {
        let parsed = parse_label_row("aa:bb:cc:dd:ee:ff,192.168.1.10,pc sonar,");

        assert_eq!(
            parsed,
            Some((
                "aa:bb:cc:dd:ee:ff".to_string(),
                "192.168.1.10".to_string(),
                "pc sonar".to_string(),
            ))
        );
    }

    #[test]
    fn merges_extra_columns_into_label() {
        let parsed = parse_label_row(",8.8.8.8,google,public dns");

        assert_eq!(
            parsed,
            Some((
                String::new(),
                "8.8.8.8".to_string(),
                "google public dns".to_string(),
            ))
        );
    }
}
