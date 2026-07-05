use std::collections::HashMap;
use std::fs::File;
use std::net::IpAddr;
use std::time::SystemTime;

use log::info;
use packet_parser::IpType;
use packet_parser::owned::{
    ApplicationOwned, DataLinkOwned, InternetOwned, PacketFlowOwned, TransportOwned,
};
use packet_parser::parse::data_link::{ethertype::Ethertype, vlan_tag::VlanTag};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct FlowStats {
    pub count: u64,            // Nombre de paquets vus pour ce flow
    pub total_bytes: u32,      // Total des octets passés dans ce flow
    pub last_seen: SystemTime, // Dernière apparition
}

pub struct FlowMatrix {
    // HashMap avec des clés de type PacketFlow et des valeurs de type FlowStats
    pub matrix: HashMap<PacketFlowOwned, FlowStats>,
    pub label: HashMap<(String, String), String>,
}

impl FlowMatrix {
    pub fn new() -> Self {
        Self {
            matrix: HashMap::new(),
            label: HashMap::new(),
        }
    }

    pub fn update_flow(&mut self, pkt: &PacketOwnedStats) {
        let ts = timeval_to_systemtime(pkt.ts_sec, pkt.ts_usec);

        // Lookup par référence : le flow (et ses ~8 String) n'est cloné
        // qu'au premier paquet du flux, pas à chaque paquet.
        if let Some(entry) = self.matrix.get_mut(&pkt.flow) {
            entry.count += 1;
            entry.total_bytes += pkt.len;
            entry.last_seen = ts;
        } else {
            self.matrix.insert(
                pkt.flow.clone(),
                FlowStats {
                    count: 1,
                    total_bytes: pkt.len,
                    last_seen: ts,
                },
            );
        }
    }

    pub fn clear(&mut self) {
        self.matrix.clear();
        self.label.clear();
    }

    // pub fn print(&self) {
    //     // En-tête
    //     println!(
    //         "{:<4} {:<30} {:<10} {:<12} {:<24}",
    //         "#", "FLOW", "COUNT", "BYTES", "LAST SEEN"
    //     );
    //     println!(
    //         "{:<4} {:<30} {:<10} {:<12} {:<24}",
    //         "-",
    //         "------------------------------",
    //         "----------",
    //         "------------",
    //         "------------------------"
    //     );

    //     let mut count_p = 0;
    //     for (flow, stats) in &self.matrix {
    //         count_p += 1;
    //         // Formatage de la date (timestamp en secondes)
    //         let last_seen = match stats.last_seen.duration_since(std::time::UNIX_EPOCH) {
    //             Ok(dur) => {
    //                 let dt = chrono::NaiveDateTime::from_timestamp_opt(dur.as_secs() as i64, 0)
    //                     .unwrap_or_default();
    //                 dt.format("%Y-%m-%d %H:%M:%S").to_string()
    //             }
    //             Err(_) => "N/A".to_string(),
    //         };

    //         println!(
    //             "{:<4} {:<30} {:<10} {:<12} {:<24}",
    //             count_p,
    //             format!("{}", flow),
    //             stats.count,
    //             stats.total_bytes,
    //             last_seen
    //         );
    //     }
    //     println!("count : {}", count_p);
    // }

    pub fn to_flat_vec(&self) -> Vec<FlowMatrixRow> {
        self.matrix
            .iter()
            .map(|(flow, stats)| {
                let ip_source = flow
                    .internet
                    .as_ref()
                    .and_then(|i| i.source_ip)
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let ip_source_type = flow
                    .internet
                    .as_ref()
                    .and_then(|i| i.ip_source_type.clone())
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let label_source = self.get_label(&flow.data_link.source_mac, &ip_source);

                let ip_destination = flow
                    .internet
                    .as_ref()
                    .and_then(|i| i.destination_ip)
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let ip_destination_type = flow
                    .internet
                    .as_ref()
                    .and_then(|i| i.ip_destination_type.clone())
                    .map(|ip| ip.to_string())
                    .unwrap_or_default();
                let label_destination =
                    self.get_label(&flow.data_link.destination_mac, &ip_destination);

                let last_seen = match stats.last_seen.duration_since(std::time::UNIX_EPOCH) {
                    Ok(dur) => {
                        chrono::DateTime::<chrono::Utc>::from_timestamp(dur.as_secs() as i64, 0)
                            .unwrap_or_else(|| {
                                chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap()
                            })
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    }
                    Err(_) => "N/A".into(),
                };

                FlowMatrixRow {
                    mac_source: flow.data_link.source_mac.clone(),
                    mac_destination: flow.data_link.destination_mac.clone(),
                    vlan_id: flow.data_link.vlan.as_ref().map(|v| v.id),
                    protocol_data_link: flow.data_link.ethertype.clone(),
                    ip_source,
                    ip_source_type,
                    label_source,
                    ip_destination,
                    ip_destination_type,
                    label_destination,
                    port_source: flow.transport.as_ref().and_then(|t| t.source_port),
                    port_destination: flow.transport.as_ref().and_then(|t| t.destination_port),
                    protocol_transport: flow.transport.as_ref().map(|t| t.protocol.clone()),
                    application_protocol: flow.application.as_ref().map(|a| a.protocol.clone()),
                    count: stats.count,
                    total_bytes: stats.total_bytes,
                    last_seen,
                }
            })
            .collect()
    }

    /// Exporte la matrice vers un fichier CSV.
    pub fn export_to_csv(&self, path: String) -> std::io::Result<()> {
        let file = File::create(&path)?;
        let mut wtr = csv::Writer::from_writer(file);

        for row in self.to_flat_vec() {
            wtr.serialize(row)?;
        }

        wtr.flush()?;
        info!("✅ Matrice exportée avec succès vers {}", path);
        Ok(())
    }

    pub fn add_label(&mut self, mac: String, ip: String, label: String) {
        self.label.insert((mac, ip), label);
    }

    pub fn get_label(&self, mac: &str, ip: &str) -> Option<String> {
        self.label
            .get(&(mac.to_string(), ip.to_string()))
            .or_else(|| self.label.get(&(String::new(), ip.to_string())))
            .or_else(|| self.label.get(&(mac.to_string(), String::new())))
            .cloned()
    }

    pub fn get_label_list(&self) -> Vec<String> {
        println!("get_label_list");
        println!("{:?}", self.label);
        let debug_label = self.label.values().cloned().collect();
        println!("{:?}", debug_label);
        debug_label
    }

    // pub fn get_all_graph_data(&self) -> Vec<FlowMatrixRow> {
    //     self.to_flat_vec()
    // }
    // pub fn add_label_list(&mut self, list: String) {
    //     self.label.insert((mac, ip), label);
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMatrixRow {
    pub mac_source: String,
    pub mac_destination: String,
    pub vlan_id: Option<u16>,
    pub protocol_data_link: String,
    pub ip_source: String,
    pub ip_source_type: String,
    pub label_source: Option<String>,
    pub ip_destination: String,
    pub ip_destination_type: String,
    pub label_destination: Option<String>,
    pub port_source: Option<u16>,
    pub port_destination: Option<u16>,
    pub protocol_transport: Option<String>,
    pub application_protocol: Option<String>,
    pub count: u64,
    pub total_bytes: u32,
    pub last_seen: String,
}

impl FlowMatrixRow {
    /// Reconstruit le flux et ses statistiques depuis une ligne de CSV exporté
    /// (chemin inverse de `to_flat_vec`). Les types d'IP sont recalculés depuis
    /// les adresses ; le VLAN ne conserve que son id (pcp/dei non exportés).
    pub fn to_flow_and_stats(&self) -> (PacketFlowOwned, FlowStats) {
        let source_ip = self.ip_source.parse::<IpAddr>().ok();
        let destination_ip = self.ip_destination.parse::<IpAddr>().ok();

        let internet = (source_ip.is_some() || destination_ip.is_some()).then(|| InternetOwned {
            source_ip,
            ip_source_type: source_ip.map(|_| IpType::from_ip(&self.ip_source)),
            destination_ip,
            ip_destination_type: destination_ip.map(|_| IpType::from_ip(&self.ip_destination)),
            protocol: self.protocol_data_link.clone(),
        });

        let transport = self
            .protocol_transport
            .as_ref()
            .filter(|p| !p.is_empty())
            .map(|p| TransportOwned {
                source_port: self.port_source,
                destination_port: self.port_destination,
                protocol: p.clone(),
            });

        let application = self
            .application_protocol
            .as_ref()
            .filter(|p| !p.is_empty())
            .map(|p| ApplicationOwned { protocol: p.clone() });

        let vlan = self.vlan_id.map(|id| VlanTag {
            id,
            pcp: 0,
            dei: false,
            inner_ethertype: Ethertype(0),
        });

        let flow = PacketFlowOwned {
            data_link: DataLinkOwned {
                destination_mac: self.mac_destination.clone(),
                source_mac: self.mac_source.clone(),
                ethertype: self.protocol_data_link.clone(),
                vlan,
            },
            internet,
            transport,
            application,
        };

        let last_seen = chrono::NaiveDateTime::parse_from_str(&self.last_seen, "%Y-%m-%d %H:%M:%S")
            .map(|dt| {
                UNIX_EPOCH + std::time::Duration::from_secs(dt.and_utc().timestamp().max(0) as u64)
            })
            .unwrap_or(UNIX_EPOCH);

        let stats = FlowStats {
            count: self.count,
            total_bytes: self.total_bytes,
            last_seen,
        };

        (flow, stats)
    }
}

use std::time::UNIX_EPOCH;

use crate::state::capture::capture_handle::messages::capture::PacketOwnedStats;

pub fn timeval_to_systemtime(tv_sec: impl Into<i64>, tv_usec: impl Into<i64>) -> SystemTime {
    let tv_sec = tv_sec.into();
    let tv_usec = tv_usec.into();

    UNIX_EPOCH + std::time::Duration::new(tv_sec as u64, (tv_usec * 1000) as u32)
}

#[cfg(test)]
mod tests {
    use super::FlowMatrix;
    use crate::state::capture::capture_handle::messages::capture::PacketOwnedStats;
    use packet_parser::owned::{DataLinkOwned, PacketFlowOwned};

    fn sample_packet(len: u32) -> PacketOwnedStats {
        PacketOwnedStats {
            ts_sec: 1_000,
            ts_usec: 0,
            caplen: len,
            len,
            flow: PacketFlowOwned {
                data_link: DataLinkOwned {
                    destination_mac: "aa:bb:cc:dd:ee:ff".to_string(),
                    source_mac: "11:22:33:44:55:66".to_string(),
                    ethertype: "IPv4".to_string(),
                    vlan: None,
                },
                internet: None,
                transport: None,
                application: None,
            },
        }
    }

    #[test]
    fn update_flow_counts_first_packet_bytes_once() {
        let mut matrix = FlowMatrix::new();
        let pkt = sample_packet(100);

        matrix.update_flow(&pkt);

        let stats = matrix.matrix.get(&pkt.flow).expect("flux inséré");
        assert_eq!(stats.count, 1);
        assert_eq!(stats.total_bytes, 100);
    }

    #[test]
    fn update_flow_accumulates_on_existing_flow() {
        let mut matrix = FlowMatrix::new();
        let pkt = sample_packet(100);

        matrix.update_flow(&pkt);
        matrix.update_flow(&pkt);
        matrix.update_flow(&pkt);

        assert_eq!(matrix.matrix.len(), 1, "un seul flux");
        let stats = matrix.matrix.get(&pkt.flow).expect("flux inséré");
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_bytes, 300);
    }

    #[test]
    fn get_label_falls_back_to_ip_only_label() {
        let mut matrix = FlowMatrix::new();
        matrix.add_label(
            String::new(),
            "8.8.8.8".to_string(),
            "google.com".to_string(),
        );

        assert_eq!(
            matrix.get_label("aa:bb:cc:dd:ee:ff", "8.8.8.8"),
            Some("google.com".to_string())
        );
    }

    #[test]
    fn get_label_falls_back_to_mac_only_label() {
        let mut matrix = FlowMatrix::new();
        matrix.add_label(
            "bc:24:11:ff:3e:15".to_string(),
            String::new(),
            "google.com".to_string(),
        );

        assert_eq!(
            matrix.get_label("bc:24:11:ff:3e:15", "192.168.1.105"),
            Some("google.com".to_string())
        );
    }

    #[test]
    fn get_label_prefers_exact_mac_ip_label() {
        let mut matrix = FlowMatrix::new();
        matrix.add_label(
            String::new(),
            "8.8.8.8".to_string(),
            "google.com".to_string(),
        );
        matrix.add_label(
            "aa:bb:cc:dd:ee:ff".to_string(),
            "8.8.8.8".to_string(),
            "custom dns".to_string(),
        );

        assert_eq!(
            matrix.get_label("aa:bb:cc:dd:ee:ff", "8.8.8.8"),
            Some("custom dns".to_string())
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::time::{SystemTime, Duration};

//     // Dummy PacketFlow pour les tests (adapter selon la vraie signature de PacketFlow)
//     #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//     struct DummyFlow(u8);

//     // Dummy impl PacketFlow<'a> pour les tests (adapter selon la vraie signature de PacketFlow)
//     // Ici, on suppose PacketFlow<'a> = DummyFlow (adapter selon ton code réel)
//     impl<'a> From<&'a DummyFlow> for PacketFlow<'a> {
//         fn from(df: &'a DummyFlow) -> Self {
//             unsafe { std::mem::transmute_copy(df) }
//         }
//     }

//     #[test]
//     fn test_new_flow_matrix() {
//         let matrix: FlowMatrix = FlowMatrix { flows: HashMap::new() };
//         assert_eq!(matrix.flows.len(), 0);
//     }

//     #[test]
//     fn test_update_flow_inserts_and_updates() {
//         let mut matrix = FlowMatrix { flows: HashMap::new() };
//         let now = SystemTime::now();
//         let flow: PacketFlow<'_> = unsafe { std::mem::zeroed() }; // Remplacer par un vrai PacketFlow si possible

//         matrix.update_flow(flow.clone(), 100, timeval_to_systemtime(now));
//         assert_eq!(matrix.flows.len(), 1);
//         let stats = matrix.flows.get(&flow).unwrap();
//         assert_eq!(stats.count, 1);
//         assert_eq!(stats.total_bytes, 100);
//         assert_eq!(stats.last_seen, now);

//         // Update same flow
//         let later = now + Duration::from_secs(10);
//         matrix.update_flow(flow.clone(), 50, timeval_to_systemtime(later));
//         let stats = matrix.flows.get(&flow).unwrap();
//         assert_eq!(stats.count, 2);
//         assert_eq!(stats.total_bytes, 150);
//         assert_eq!(stats.last_seen, later);
//     }
// }
