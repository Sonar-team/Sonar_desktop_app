use std::sync::{Arc, Mutex};

use packet_parser::MacAddress;
use tauri::{AppHandle, Manager, path::BaseDirectory};

use crate::state::flow_matrix::FlowMatrix;

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
) -> Result<Vec<String>, tauri::Error> {
    const LABEL_NAME: &str = "pc sonar";

    let mut labels = Vec::new();

    for interface in interfaces {
        let Some(mac_addr) = interface.mac_addr else {
            continue;
        };

        let mac_addr = mac_addr.to_string();

        for ipv4 in interface.ipv4_addrs() {
            labels.push(format!("{mac_addr},{ipv4},{LABEL_NAME}"));
        }

        for ipv6 in interface.ipv6_addrs() {
            labels.push(format!("{mac_addr},{ipv6},{LABEL_NAME}"));
        }
    }

    Ok(labels)
}

pub fn add_labels_to_file(app: &AppHandle, labels: Vec<String>) -> Result<(), tauri::Error> {
    let resource_path = app
        .path()
        .resolve("resources/labels.csv", BaseDirectory::Resource)?;
    println!("resource_path: {:?}", resource_path);

    let csv_data = if labels.is_empty() {
        String::new()
    } else {
        format!("{}\n", labels.join("\n"))
    };

    std::fs::write(resource_path, csv_data)?;

    Ok(())
}

pub fn update_labels_in_state(app: &AppHandle, labels: Vec<String>) -> Result<(), tauri::Error> {
    let state_label = app.state::<Arc<Mutex<FlowMatrix>>>();
    let mut state_label = state_label.lock().unwrap();

    for label in labels {
        let Some((mac, ip, label_name)) = parse_label_row(&label) else {
            continue;
        };

        state_label.add_label(mac.to_string(), ip, label_name);
    }

    Ok(())
}

fn parse_label_row(row: &str) -> Option<(MacAddress, String, String)> {
    let mut parts = row.splitn(3, ',');
    let mac = parts.next()?.trim();
    let ip = parts.next()?.trim();
    let label = parts.next()?.trim();

    if mac.is_empty() || ip.is_empty() || label.is_empty() {
        return None;
    }
let macmac = MacAddress::try_from(mac.to_string());
    Some((macmac.unwrap(), ip.to_string(), label.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{create_labels_from_network_interfaces, parse_label_row};
    use netdev::Interface;
    use netdev::ipnet::{Ipv4Net, Ipv6Net};
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn creates_one_row_per_ip_address() {
        let mut interface = Interface::dummy();
        interface.mac_addr = Some("aa:bb:cc:dd:ee:ff".parse().unwrap());
        interface.ipv4 = vec![Ipv4Net::new(Ipv4Addr::new(192, 168, 1, 10), 24).unwrap()];
        interface.ipv6 = vec![Ipv6Net::new("2001:db8::10".parse::<Ipv6Addr>().unwrap(), 64).unwrap()];

        let labels = create_labels_from_network_interfaces(vec![interface]).unwrap();

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

        let labels = create_labels_from_network_interfaces(vec![interface]).unwrap();

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
}
