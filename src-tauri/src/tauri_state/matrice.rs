//! Module pour gérer l'état de Sonar.
//!
//! Ce module fournit les structures nécessaires pour maintenir l'état
//! actuel de l'application Sonar, en particulier pour suivre les trames réseau.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rust_xlsxwriter::Worksheet;
use csv::Writer;
use log::{error, info, debug};
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};

use crate::commandes::get_graph_data::GraphBuilder;
use crate::errors::export::ExportError;

use super::capture::capture_handle::layer_2_infos::layer_3_infos::ip_type::IpType;
use super::capture::capture_handle::layer_2_infos::layer_3_infos::Layer3Infos;
use super::capture::capture_handle::layer_2_infos::PacketInfos;

#[derive(Debug, Serialize, Deserialize)]
pub struct PacketInfoEntry {
    pub infos: PacketKey,
    pub stats: PacketStats,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct PacketKey {
    pub mac_address_source: String,
    pub mac_address_destination: String,
    pub interface: String,
    pub l_3_protocol: String,
    pub layer_3_infos: Layer3Infos,
}

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

impl From<&PacketInfos> for PacketKey {
    fn from(info: &PacketInfos) -> Self {
        PacketKey {
            mac_address_source: info.mac_address_source.clone(),
            mac_address_destination: info.mac_address_destination.clone(),
            interface: info.interface.clone(),
            l_3_protocol: info.l_3_protocol.clone(),
            layer_3_infos: info.layer_3_infos.clone(),
        }
    }
}

pub struct SonarState {
    pub matrice: HashMap<PacketKey, PacketStats>,
}

impl SonarState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(SonarState {
            matrice: HashMap::new(),
        }))
    }

    pub fn reset(&mut self) {
        self.matrice.clear();
    }

    pub fn update_matrice_with_packet(&mut self, new_packet: &PacketInfos) {
        let packet_size = new_packet.packet_size as u32;
        let key = PacketKey::from(new_packet);

        let existed = self.matrice.contains_key(&key);
        if existed {
            debug!("Mise à jour d'un paquet existant dans la matrice : {:?}", key);
        } else {
            debug!("Ajout d'un nouveau paquet dans la matrice : {:?}", key);
        }

        self.matrice
            .entry(key)
            .and_modify(|stats| stats.update(packet_size))
            .or_insert(PacketStats::new(packet_size));
    }

    pub fn cmd_save_packets_to_csv(&self, file_path: String) -> Result<(), ExportError> {
        info!("Début de l'export CSV vers {}", file_path);
        let mut wtr = Writer::from_path(&file_path).map_err(|e| {
            error!("Erreur lors de l'ouverture du fichier CSV {} : {}", file_path, e);
            ExportError::Io(e.to_string())
        })?;

        let total = self.matrice.len();
        info!("Nombre total de lignes à écrire : {}", total);

        for (i, (packet_key, stats)) in self.matrice.iter().enumerate() {
            let packet_csv = PacketInfosFlaten::from_packet_key_and_stats(packet_key, stats);
            wtr.serialize(packet_csv).map_err(|e| {
                error!("Erreur de sérialisation à la ligne {} : {:?}", i + 1, e);
                ExportError::Csv(e.to_string())
            })?;
        }

        wtr.flush().map_err(|e| {
            error!("Erreur lors du flush du fichier CSV : {}", e);
            ExportError::Io(e.to_string())
        })?;

        info!("Export CSV terminé avec succès : {}", file_path);
        Ok(())
    }

    pub fn cmd_save_packets_to_excel(&self, file_path: String) -> Result<(), ExportError> {
        info!("Début de l'export Excel vers {}", file_path);

        let write_cell = |sheet: &mut Worksheet, row: u32, col: u16, value: &str| -> Result<(), ExportError> {
            sheet.write_string(row, col, value)
                .map(|_| ())
                .map_err(|e| ExportError::Xlsx(e.to_string()))
        };
        let write_number = |sheet: &mut Worksheet, row: u32, col: u16, value: f64| -> Result<(), ExportError> {
            sheet.write_number(row, col, value)
                .map(|_| ())
                .map_err(|e| ExportError::Xlsx(e.to_string()))
        };

        let data = &self.matrice;
        let total = data.len();
        info!("Nombre total de lignes à écrire : {}", total);

        let mut workbook = Workbook::new();
        let mut sheet = workbook.add_worksheet();

        let headers = [
            "MAC Source", "MAC Destination", "Interface", "L3 Protocol",
            "IP Source", "IP Source Type", "IP Destination", "IP Destination Type",
            "L4 Protocol", "Source Port", "Destination Port", "L7 Protocol",
            "Taille des packets", "Count",
        ];
        for (i, header) in headers.iter().enumerate() {
            write_cell(&mut sheet, 0, i as u16, header)?;
        }

        for (i, (packet_key, stats)) in data.iter().enumerate() {
            let packet_csv = PacketInfosFlaten::from_packet_key_and_stats(packet_key, stats);
            let row = i as u32 + 1;

            if i % 100 == 0 {
                debug!("Écriture de la ligne {}/{}", i + 1, total);
            }

            write_cell(&mut sheet, row, 0, &packet_csv.mac_address_source)?;
            write_cell(&mut sheet, row, 1, &packet_csv.mac_address_destination)?;
            write_cell(&mut sheet, row, 2, &packet_csv.interface)?;
            write_cell(&mut sheet, row, 3, &packet_csv.l_3_protocol)?;
            write_cell(&mut sheet, row, 4, packet_csv.ip_source.as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 5, packet_csv.ip_source_type.as_ref().map(|v| v.to_string()).as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 6, packet_csv.ip_destination.as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 7, packet_csv.ip_destination_type.as_ref().map(|v| v.to_string()).as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 8, packet_csv.l_4_protocol.as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 9, packet_csv.port_source.as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 10, packet_csv.port_destination.as_deref().unwrap_or(""))?;
            write_cell(&mut sheet, row, 11, packet_csv.l_7_protocol.as_deref().unwrap_or(""))?;
            write_number(&mut sheet, row, 12, packet_csv.packet_size as f64)?;
            write_number(&mut sheet, row, 13, packet_csv.count as f64)?;
        }

        workbook.save(&file_path).map_err(|e| {
            error!("Erreur lors de la sauvegarde du fichier Excel : {}", e);
            ExportError::Xlsx(e.to_string())
        })?;

        info!("Export Excel terminé avec succès : {}", file_path);
        Ok(())
    }

    pub fn get_matrice_data(&self) -> Result<String, String> {
        let data: &HashMap<PacketKey, PacketStats> = &self.matrice;

        let entries: Vec<PacketInfoEntry> = data
            .iter()
            .take(30)
            .map(|(info, stats)| PacketInfoEntry {
                infos: info.clone(),
                stats: stats.clone(),
            })
            .collect();

        match serde_json::to_string(&entries) {
            Ok(serialized_data) => Ok(serialized_data),
            Err(e) => {
                let err_msg = format!("Erreur de sérialisation : {}", e);
                error!("{}", err_msg);
                Err(err_msg)
            }
        }
    }

    pub fn get_matrice_len(&self) -> usize {
        self.matrice.len()
    }

    pub fn get_graph_data(&self) -> Result<String, String> {
        let data = self.matrice.clone();
        let mut graph_builder = GraphBuilder::new();

        let total = data.len();

        for (i, (packet, _)) in data.iter().enumerate() {
            if i % 100 == 0 {
                debug!("Ajout de l'arête {}/{}", i + 1, total);
            }
            graph_builder.add_edge(packet);
        }

        let graph_data = graph_builder.build_graph_data();

        serde_json::to_string(&graph_data).map_err(|e| {
            error!("Erreur de sérialisation du graphe : {}", e);
            format!("Serialization error: {}", e)
        })
    }
}

#[derive(Serialize)]
struct PacketInfosFlaten {
    mac_address_source: String,
    mac_address_destination: String,
    interface: String,
    l_3_protocol: String,
    ip_source: Option<String>,
    ip_source_type: Option<IpType>,
    ip_destination: Option<String>,
    ip_destination_type: Option<IpType>,
    l_4_protocol: Option<String>,
    port_source: Option<String>,
    port_destination: Option<String>,
    l_7_protocol: Option<String>,
    packet_size: usize,
    count: u32,
}

impl PacketInfosFlaten {
    fn from_packet_key_and_stats(key: &PacketKey, stats: &PacketStats) -> Self {
        PacketInfosFlaten {
            mac_address_source: key.mac_address_source.clone(),
            mac_address_destination: key.mac_address_destination.clone(),
            interface: key.interface.clone(),
            l_3_protocol: key.l_3_protocol.clone(),
            ip_source: key.layer_3_infos.ip_source.clone(),
            ip_source_type: key.layer_3_infos.ip_source_type.clone(),
            ip_destination: key.layer_3_infos.ip_destination.clone(),
            ip_destination_type: key.layer_3_infos.ip_destination_type.clone(),
            l_4_protocol: key.layer_3_infos.l_4_protocol.clone(),
            port_source: key.layer_3_infos.layer_4_infos.port_source.clone(),
            port_destination: key.layer_3_infos.layer_4_infos.port_destination.clone(),
            l_7_protocol: key.layer_3_infos.layer_4_infos.l_7_protocol.clone(),
            packet_size: stats.packet_size_total as usize,
            count: stats.count,
        }
    }
}
