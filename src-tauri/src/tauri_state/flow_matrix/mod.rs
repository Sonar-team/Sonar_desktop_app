//! Module pour gérer l'état de Sonar.
//!
//! Ce module fournit les structures nécessaires pour maintenir l'état
//! actuel de l'application Sonar, en particulier pour suivre les trames réseau.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use packet_parser::owned::PacketFlowOwned;
use packet_parser::IpType;
use rust_xlsxwriter::Worksheet;
use csv::Writer;
use log::{error, info, debug};
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};

use crate::errors::export::ExportError;
use crate::tauri_state::capture::capture_handle::messages::capture::PacketOwnedStats;






#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PacketStats {
    pub count: u32,
    pub packet_size_total: u32,
}

impl PacketStats {
    fn new(packet_size: u32) -> Self {
        Self {
            count: 1,
            packet_size_total: packet_size,
        }
    }

    fn update(&mut self, packet_size: u32) {
        self.count += 1;
        self.packet_size_total += packet_size;
    }
}



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

    pub fn reset(&mut self) {
        self.matrix.clear();
    }

    pub fn update_flow(&mut self, pkt: &PacketOwnedStats) {
        let ts = timeval_to_systemtime(pkt.ts_sec, pkt.ts_usec);

        let entry = self.matrix.entry(pkt.flow.clone()).or_insert(FlowStats {
            count: 0,
            total_bytes: pkt.len,
            last_seen: ts,
        });
        entry.count += 1;
        entry.total_bytes += pkt.len;
        entry.last_seen = ts;
    }

    // pub fn cmd_save_packets_to_csv(&self, file_path: String) -> Result<(), ExportError> {
    //     info!("Début de l'export CSV vers {}", file_path);
    //     let mut wtr = Writer::from_path(&file_path).map_err(|e| {
    //         error!("Erreur lors de l'ouverture du fichier CSV {} : {}", file_path, e);
    //         ExportError::Io(e.to_string())
    //     })?;

    //     let total = self.matrice.len();
    //     info!("Nombre total de lignes à écrire : {}", total);

    //     for (i, (packet_key, stats)) in self.matrice.iter().enumerate() {
    //         let packet_csv = PacketInfosFlaten::from_packet_key_and_stats(packet_key, stats);
    //         wtr.serialize(packet_csv).map_err(|e| {
    //             error!("Erreur de sérialisation à la ligne {} : {:?}", i + 1, e);
    //             ExportError::Csv(e.to_string())
    //         })?;
    //     }

    //     wtr.flush().map_err(|e| {
    //         error!("Erreur lors du flush du fichier CSV : {}", e);
    //         ExportError::Io(e.to_string())
    //     })?;

    //     info!("Export CSV terminé avec succès : {}", file_path);
    //     Ok(())
    // }

    // pub fn cmd_save_packets_to_excel(&self, file_path: String) -> Result<(), ExportError> {
    //     info!("Début de l'export Excel vers {}", file_path);

    //     let write_cell = |sheet: &mut Worksheet, row: u32, col: u16, value: &str| -> Result<(), ExportError> {
    //         sheet.write_string(row, col, value)
    //             .map(|_| ())
    //             .map_err(|e| ExportError::Xlsx(e.to_string()))
    //     };
    //     let write_number = |sheet: &mut Worksheet, row: u32, col: u16, value: f64| -> Result<(), ExportError> {
    //         sheet.write_number(row, col, value)
    //             .map(|_| ())
    //             .map_err(|e| ExportError::Xlsx(e.to_string()))
    //     };

    //     let data = &self.matrice;
    //     let total = data.len();
    //     info!("Nombre total de lignes à écrire : {}", total);

    //     let mut workbook = Workbook::new();
    //     let mut sheet = workbook.add_worksheet();

    //     let headers = [
    //         "MAC Source", "MAC Destination", "Interface", "L3 Protocol",
    //         "IP Source", "IP Source Type", "IP Destination", "IP Destination Type",
    //         "L4 Protocol", "Source Port", "Destination Port", "L7 Protocol",
    //         "Taille des packets", "Count",
    //     ];
    //     for (i, header) in headers.iter().enumerate() {
    //         write_cell(&mut sheet, 0, i as u16, header)?;
    //     }

    //     for (i, (packet_key, stats)) in data.iter().enumerate() {
    //         let packet_csv = PacketInfosFlaten::from_packet_key_and_stats(packet_key, stats);
    //         let row = i as u32 + 1;

    //         if i % 100 == 0 {
    //             debug!("Écriture de la ligne {}/{}", i + 1, total);
    //         }

    //         write_cell(&mut sheet, row, 0, &packet_csv.mac_address_source)?;
    //         write_cell(&mut sheet, row, 1, &packet_csv.mac_address_destination)?;
    //         write_cell(&mut sheet, row, 2, &packet_csv.interface)?;
    //         write_cell(&mut sheet, row, 3, &packet_csv.l_3_protocol)?;
    //         write_cell(&mut sheet, row, 4, packet_csv.ip_source.as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 5, packet_csv.ip_source_type.as_ref().map(|v| v.to_string()).as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 6, packet_csv.ip_destination.as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 7, packet_csv.ip_destination_type.as_ref().map(|v| v.to_string()).as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 8, packet_csv.l_4_protocol.as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 9, packet_csv.port_source.as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 10, packet_csv.port_destination.as_deref().unwrap_or(""))?;
    //         write_cell(&mut sheet, row, 11, packet_csv.l_7_protocol.as_deref().unwrap_or(""))?;
    //         write_number(&mut sheet, row, 12, packet_csv.packet_size as f64)?;
    //         write_number(&mut sheet, row, 13, packet_csv.count as f64)?;
    //     }

    //     workbook.save(&file_path).map_err(|e| {
    //         error!("Erreur lors de la sauvegarde du fichier Excel : {}", e);
    //         ExportError::Xlsx(e.to_string())
    //     })?;

    //     info!("Export Excel terminé avec succès : {}", file_path);
    //     Ok(())
    // }

    pub fn get_matrice_len(&self) -> usize {
        self.matrix.len()
    }

    // pub fn get_graph_data(&self) -> Result<String, String> {
    //     let data = self.matrix.clone();
    //     let mut graph_builder = GraphBuilder::new();

    //     let total = data.len();

    //     for (i, (packet, _)) in data.iter().enumerate() {
    //         // if i % 100 == 0 {
    //         //     info!("Ajout de l'arête {}/{}", i + 1, total);
    //         // }
    //         graph_builder.add_edge(packet);
    //     }

    //     let graph_data = graph_builder.build_graph_data();

    //     serde_json::to_string(&graph_data).map_err(|e| {
    //         error!("Erreur de sérialisation du graphe : {}", e);
    //         format!("Serialization error: {}", e)
    //     })
    // }
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

// impl PacketInfosFlaten {
//     fn from_packet_key_and_stats(key: &PacketKey, stats: &PacketStats) -> Self {
//         PacketInfosFlaten {
//             mac_address_source: key.mac_address_source.clone(),
//             mac_address_destination: key.mac_address_destination.clone(),
//             interface: key.interface.clone(),
//             l_3_protocol: key.l_3_protocol.clone(),
//             ip_source: key.layer_3_infos.ip_source.clone(),
//             ip_source_type: key.layer_3_infos.ip_source_type.clone(),
//             ip_destination: key.layer_3_infos.ip_destination.clone(),
//             ip_destination_type: key.layer_3_infos.ip_destination_type.clone(),
//             l_4_protocol: key.layer_3_infos.l_4_protocol.clone(),
//             port_source: key.layer_3_infos.layer_4_infos.port_source.clone(),
//             port_destination: key.layer_3_infos.layer_4_infos.port_destination.clone(),
//             l_7_protocol: key.layer_3_infos.layer_4_infos.l_7_protocol.clone(),
//             packet_size: stats.packet_size_total as usize,
//             count: stats.count,
//         }
//     }
// }


pub fn timeval_to_systemtime(tv_sec: i64, tv_usec: i64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::new(tv_sec as u64, (tv_usec * 1000) as u32)
}