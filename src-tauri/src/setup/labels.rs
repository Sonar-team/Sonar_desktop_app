use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use tauri::{AppHandle, Manager, path::BaseDirectory};

use crate::{
    errors::CaptureStateError,
    state::{flow_matrix::FlowMatrix, label_files_list::PcInfoLabel},
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
) -> Result<(), tauri::Error> {
    let state_label = app.state::<Arc<Mutex<PcInfoLabel>>>();
    let mut pcinfo = state_label.lock().unwrap();
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
    let pcinfo = pcinfo.lock().unwrap().get_label().clone();

    for label in pcinfo {
        let Some((mac, ip, label_name)) = parse_label_row(&label) else {
            continue;
        };

        state_label.add_label(mac.to_string(), ip, label_name);
    }

    Ok(())
}

pub fn parse_label_row(row: &str) -> Option<(String, String, String)> {
    let parts: Vec<_> = row.split(',').map(clean_csv_field).collect();
    match parts.as_slice() {
        [mac, ip, label] if is_mac_address(mac) && is_ip_address(ip) && !label.is_empty() => {
            // si tous les arguments sont présents
            // println!("parse_label_row: mac: {0}, ip: {1}, label: {2}", mac, ip, label );
            Some((mac.to_string(), ip.to_string(), label.to_string()))
        }
        [mac, ip, label] if mac.is_empty() && is_ip_address(ip) && !label.is_empty() => {
            // si il manque l'adresse mac
            println!(
                "parse_label_row: mac: {0}, ip: {1}, label: {2}",
                mac, ip, label
            );
            Some((String::new(), ip.to_string(), label.to_string()))
        }
        [mac, ip, label] if is_mac_address(mac) && ip.is_empty() && !label.is_empty() => {
            // si il manque l'adresse IP
            // println!("parse_label_row: mac: {0}, ip: {1}, label: {2}", mac, ip, label );
            Some((mac.to_string(), String::new(), label.to_string()))
        }
        [mac, ip, label] if is_mac_address(mac) && is_ip_address(ip) && label.is_empty() => {
            //si il manque le label
            // println!("parse_label_row: mac: {0}, ip: {1}, label: {2}", mac, ip, label );
            Some((mac.to_string(), ip.to_string(), String::from("Label?")))
        }
        [mac, ip, label] if is_mac_address(mac) && ip.is_empty() && label.is_empty() => {
            //si il manque l'adresse mac ET le label
            // println!("parse_label_row: mac: {0}, ip: {1}, label: {2}", mac, ip, label );
            Some((mac.to_string(), String::new(), String::from("Label?")))
        }
        [mac, ip, label] if mac.is_empty() && is_ip_address(ip) && label.is_empty() => {
            //si il manque l'adresse ip ET le label
            // println!("parse_label_row: mac: {0}, ip: {1}, label: {2}", mac, ip, label );
            Some((String::new(), ip.to_string(), String::from("Label?")))
        }
        _ => None,
    }
}

pub fn clean_csv_field(value: &str) -> &str {
    value.trim().trim_matches('"').split('/').next().unwrap()
}

pub fn is_ip_address(value: &str) -> bool {
    value.parse::<IpAddr>().is_ok()
}

pub fn is_mac_address(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
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
}
