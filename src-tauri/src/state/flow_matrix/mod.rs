use std::collections::HashMap;
use std::fs::File;
use std::time::SystemTime;

use log::info;
use packet_parser::owned::PacketFlowOwned;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FlowStats {
    pub count: u64,            // Nombre de paquets vus pour ce flow
    pub total_bytes: u32,      // Total des octets passés dans ce flow
    pub last_seen: SystemTime, // Dernière apparition
}

pub struct FlowMatrix {
    // HashMap avec des clés de type PacketFlow et des valeurs de type FlowStats
    pub matrix: HashMap<PacketFlowOwned, FlowStats>,
}

impl FlowMatrix {
    pub fn new() -> Self {
        Self {
            matrix: HashMap::new(),
        }
    }

    pub fn update_flow(&mut self, pkt: &PacketOwnedStats) -> u64 {
        let ts = timeval_to_systemtime(pkt.ts_sec.into(), pkt.ts_usec.into());

        let entry = self.matrix.entry(pkt.flow.clone()).or_insert(FlowStats {
            count: 0,
            total_bytes: pkt.len,
            last_seen: ts,
        });
        entry.count += 1;
        entry.total_bytes += pkt.len;
        entry.last_seen = ts;

        entry.count
    }

    pub fn clear(&mut self) {
        self.matrix.clear();
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
                let protocol_network = flow
                    .internet
                    .as_ref()
                    .map(|i| i.protocol.clone())
                    .unwrap_or_default();
                let last_seen = match stats.last_seen.duration_since(std::time::UNIX_EPOCH) {
                    Ok(dur) => chrono::DateTime::<chrono::Utc>::from_timestamp(dur.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap())
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                    Err(_) => "N/A".into(),
                };

                FlowMatrixRow {
                    mac_source: flow.data_link.source_mac.clone(),
                    mac_destination: flow.data_link.destination_mac.clone(),
                    protocol_data_link: flow.data_link.ethertype.clone(),
                    ip_source,
                    ip_source_type,
                    ip_destination,
                    ip_destination_type,
                    protocol_network,
                    port_source: flow.transport.as_ref().and_then(|t| t.source_port),
                    port_destination: flow.transport.as_ref().and_then(|t| t.destination_port),
                    protocol_transport: flow.transport.as_ref().map(|t| t.protocol.clone()),
                    application_protocol: flow
                        .application
                        .as_ref()
                        .map(|a| a.application_protocol.clone()),
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
}

#[derive(Debug, Clone, Serialize)]
pub struct FlowMatrixRow {
    pub mac_source: String,
    pub mac_destination: String,
    pub protocol_data_link: String,
    pub ip_source: String,
    pub ip_source_type: String,
    pub ip_destination: String,
    pub ip_destination_type: String,
    pub protocol_network: String,
    pub port_source: Option<u16>,
    pub port_destination: Option<u16>,
    pub protocol_transport: Option<String>,
    pub application_protocol: Option<String>,
    pub count: u64,
    pub total_bytes: u32,
    pub last_seen: String,
}

use std::time::UNIX_EPOCH;

use crate::state::capture::capture_handle::messages::capture::PacketOwnedStats;

pub fn timeval_to_systemtime(tv_sec: i64, tv_usec: i64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::new(tv_sec as u64, (tv_usec * 1000) as u32)
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
