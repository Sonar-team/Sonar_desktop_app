//! Module pour gérer l'état de Sonar.
//!
//! Ce module fournit les structures nécessaires pour maintenir l'état
//! actuel de l'application Sonar, en particulier pour suivre les trames réseau.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use csv::Writer;
use log::error;
use rust_xlsxwriter::Workbook;
use serde::Serialize;
use thiserror::Error;

use crate::get_matrice::get_graph_data::GraphBuilder;
use crate::sniff::capture_packet::layer_2_infos::layer_3_infos::ip_type::IpType;
use crate::sniff::capture_packet::layer_2_infos::PacketInfos;

#[derive(Debug, Serialize)]
struct PacketInfoEntry {
    info: PacketInfos,
    count: u32,
}

/// `SonarState` encapsule l'état global de l'application Sonar.
///
/// Cette structure est conçue pour stocker et gérer les informations sur les trames réseau
/// capturées, y compris le comptage de leurs occurrences.
///
/// # Structure
/// `SonarState` contient un `Arc<Mutex<HashMap<PacketInfos, u32>>>`.
/// - `Arc` permet un accès thread-safe et partagé à l'état.
/// - `Mutex` garantit que l'accès à l'état est mutuellement exclusif,
///   empêchant les conditions de concurrence.
/// - `HashMap<PacketInfos, u32>` stocke les trames réseau (`PacketInfos`) et
///   leur nombre d'occurrences (`u32`).
///
/// # Exemple
/// ```
/// use std::sync::{Mutex, Arc};
/// use std::collections::HashMap;
/// use crate::sniff::capture_packet::layer_2_infos::PacketInfos;
/// use crate::SonarState;
///
/// let state = SonarState::new();
/// // Utilisez `state` ici pour gérer les trames réseau et leur comptage
/// ```

pub struct SonarState {
    // Contient les trames réseau et leur nombre d'occurrences
    pub matrice: HashMap<PacketInfos, u32>,
    // Indique si le filtrage des adresses IPv6 est activé
    pub filter_ipv6: bool,
    pub actif: bool,
}

impl SonarState {
    // Constructeur pour initialiser `SonarState`
    pub fn new() -> Arc<Mutex<Self>> {
        let state = Arc::new(Mutex::new(SonarState {
            matrice: HashMap::new(),
            filter_ipv6: true, // Par défaut, le filtrage IPv6 est activé
            actif: true,
        }));
        state
    }

    // Méthode pour basculer l'état de `actif`
    pub fn toggle_actif(&self) {
        let mut filter_state = self.actif;
        filter_state = !filter_state; // Inverse l'état actuel
        println!("filter_state: {:?}", filter_state);
    }

    // Méthode pour basculer l'état de `filter_ipv6`
    pub fn toggle_filter_ipv6(&self) {
        let mut filter_state = self.filter_ipv6;
        filter_state = !filter_state; // Inverse l'état actuel
        println!("filter_state: {:?}", filter_state);
    }

    // Getter method for matrice
    pub fn get_matrice(&self) -> &HashMap<PacketInfos, u32> {
        &self.matrice
    }

    // Met à jour `matrice` avec un nouveau paquet
    pub fn update_matrice_with_packet(&mut self, new_packet: PacketInfos) {
        //println!("new_packet: {:?}", new_packet);

        //println!("  before update matrice: {:?}", self.matrice);
        let entry = self.matrice.entry(new_packet).or_insert(0);
        *entry += 1;
        //println!("  updated matrice: {:?}", self.matrice);
        //println!("");
    }

    /// Fonction pour enregistrer les paquets vers un fichier CSV.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Chemin du fichier CSV.
    /// * `app` - Application handle de Tauri.
    ///
    /// # Exemple
    ///
    /// ```rust
    /// cmd_save_packets_to_csv(String::from("paquets.csv"), state);
    /// ```
    pub fn cmd_save_packets_to_csv(&self, file_path: String) -> Result<(), MyError> {
        let data = self.matrice.clone(); // Acquire a lock

        // Create a CSV writer
        let mut wtr = Writer::from_path(file_path).map_err(|e| MyError::IoError(e.to_string()))?;

        // Serialize the entire vector to the CSV
        for (packet, count) in data.iter() {
            let packet_csv = PacketInfosFlaten::from_packet_infos(packet, *count);
            wtr.serialize(packet_csv)
                .map_err(|e| MyError::CsvError(e.to_string()))?;
        }

        // Flush to ensure all data is written to the file
        wtr.flush().map_err(|e| MyError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Fonction pour enregistrer les paquets vers un fichier Excel.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Chemin du fichier Excel.
    /// * `state` - État contenant les données des paquets.
    ///
    /// # Exemple
    ///
    /// ```rust
    /// cmd_save_packets_to_excel(String::from("paquets.xlsx"), state);
    /// ```
    pub fn cmd_save_packets_to_excel(&self, file_path: String) -> Result<(), MyError> {
        // Lock the state to access the data
        let data = self.matrice.clone(); // Acquire a lock

        // Create an Excel workbook
        let mut workbook = Workbook::new();

        // Add a worksheet
        let sheet = workbook.add_worksheet();

        // Write header
        let headers = [
            "MAC Source",
            "MAC Destination",
            "Interface",
            "L3 Protocol",
            "IP Source",
            "IP Source Type",
            "IP Destination",
            "IP Destination Type",
            "L4 Protocol",
            "Source Port",
            "Destination Port",
            "L7 Protocol",
            "Taille des packets",
            "Count",
        ];

        for (i, header) in headers.iter().enumerate() {
            sheet
                .write_string(0, i as u16, header.to_string())
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }

        // Serialize the entire vector to the Excel sheet
        for (i, (packet, count)) in data.iter().enumerate() {
            let packet_csv = PacketInfosFlaten::from_packet_infos(packet, *count);

            // Écriture des champs dans chaque colonne
            sheet
                .write_string(i as u32 + 1, 0, &packet_csv.mac_address_source)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
            sheet
                .write_string(i as u32 + 1, 1, &packet_csv.mac_address_destination)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
            sheet
                .write_string(i as u32 + 1, 2, &packet_csv.interface)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
            sheet
                .write_string(i as u32 + 1, 3, &packet_csv.l_3_protocol)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;

            // Les champs optionnels doivent être gérés pour éviter les valeurs null
            if let Some(ip_src) = &packet_csv.ip_source {
                sheet
                    .write_string(i as u32 + 1, 4, ip_src)
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(ip_src_type) = &packet_csv.ip_source_type {
                sheet
                    .write_string(i as u32 + 1, 5, ip_src_type.to_string())
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(ip_dst) = &packet_csv.ip_destination {
                sheet
                    .write_string(i as u32 + 1, 6, ip_dst)
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(ip_dst_type) = &packet_csv.ip_destination_type {
                sheet
                    .write_string(i as u32 + 1, 7, ip_dst_type.to_string())
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(l4_proto) = &packet_csv.l_4_protocol {
                sheet
                    .write_string(i as u32 + 1, 8, l4_proto)
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(port_src) = &packet_csv.port_source {
                sheet
                    .write_string(i as u32 + 1, 9, port_src)
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(port_dst) = &packet_csv.port_destination {
                sheet
                    .write_string(i as u32 + 1, 10, port_dst)
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }
            if let Some(l7_ptoto) = &packet_csv.l_7_protocol {
                sheet
                    .write_string(i as u32 + 1, 11, l7_ptoto)
                    .map_err(|e| MyError::XlsxError(e.to_string()))?;
            }

            // Écriture du champ 'size'
            sheet
                .write_number(i as u32 + 1, 12, packet_csv.packet_size as f64)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;

            // Écriture du champ 'count'
            sheet
                .write_number(i as u32 + 1, 13, packet_csv.count as f64)
                .map_err(|e| MyError::XlsxError(e.to_string()))?;
        }

        // Close the workbook
        workbook
            .save(file_path)
            .map_err(|e| MyError::XlsxError(e.to_string()))?;

        Ok(())
    }

    /// Récupère et sérialise les données de trafic réseau depuis l'état partagé.
    ///
    /// # Retour
    ///
    /// Cette fonction retourne `Ok(String)` contenant les données sérialisées en cas de succès,
    /// ou `Err(String)` avec un message d'erreur en cas d'échec.
    ///
    /// # Exemples
    ///
    /// ```ignore
    /// let result = get_matrice_data(app);
    /// match result {
    ///     Ok(json_string) => println!("Données sérialisées : {}", json_string),
    ///     Err(e) => eprintln!("Erreur : {}", e),
    /// }
    /// ```
    pub fn get_matrice_data(&self) -> Result<String, String> {
        let data: &HashMap<PacketInfos, u32> = &self.matrice;

        let entries: Vec<PacketInfoEntry> = data
            .iter()
            .map(|(info, &count)| PacketInfoEntry {
                info: info.clone(),
                count,
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

    pub fn get_graph_data(&self) -> Result<String, String> {
        let data = self.matrice.clone(); // Acquire a lock

        let mut graph_builder = GraphBuilder::new();

        for (packet, _) in data.iter() {
            graph_builder.add_edge(packet);
        }

        let graph_data = graph_builder.build_graph_data();
        //println!("{:?}", serde_json::to_string(&graph_data).unwrap());

        serde_json::to_string(&graph_data).map_err(|e| {
            error!("Serialization error: {}", e);
            format!("Serialization error: {}", e)
        })
    }
}

/// Enum représentant les différentes erreurs pouvant survenir lors de l'écriture de paquets vers un fichier CSV ou Excel.
#[derive(Debug, Error, serde::Serialize)]
pub enum MyError {
    /// Erreur d'entrée/sortie avec un message explicatif.
    #[error("Erreur d'E/S : {0}")]
    IoError(String),

    /// Erreur lors de la manipulation de fichiers CSV avec un message explicatif.
    #[error("Erreur CSV : {0}")]
    CsvError(String),

    /// Erreur de conversion UTF-8 avec un message explicatif.
    #[error("Erreur de conversion UTF-8 : {0}")]
    Utf8Error(String),

    /// Erreur lors de la manipulation de fichiers Excel avec un message explicatif.
    #[error("Erreur Excel : {0}")]
    XlsxError(String),
}

/// Structure représentant les informations des paquets à sérialiser vers un fichier CSV.
#[derive(Serialize)]
struct PacketInfosFlaten {
    /// Adresse MAC source du paquet.
    mac_address_source: String,
    /// Adresse MAC destination du paquet.
    mac_address_destination: String,
    /// Interface du paquet.
    interface: String,
    /// Protocole de la couche 3 du paquet.
    l_3_protocol: String,
    /// Adresse IP source du paquet (optionnel).
    ip_source: Option<String>,
    ip_source_type: Option<IpType>,
    /// Adresse IP destination du paquet (optionnel).
    ip_destination: Option<String>,
    ip_destination_type: Option<IpType>,
    /// Protocole de la couche 4 du paquet (optionnel).
    l_4_protocol: Option<String>,
    /// Port source du paquet (optionnel).
    port_source: Option<String>,
    /// Port destination du paquet (optionnel).
    port_destination: Option<String>,
    /// Taille du paquet.
    l_7_protocol: Option<String>,
    packet_size: usize,
    /// Nombre de fois que ce paquet a été rencontré.
    count: u32,
}

impl PacketInfosFlaten {
    /// Convertit les informations du paquet en une structure `PacketInfosFlaten`.
    fn from_packet_infos(packet: &PacketInfos, count: u32) -> Self {
        PacketInfosFlaten {
            mac_address_source: packet.mac_address_source.clone(),
            mac_address_destination: packet.mac_address_destination.clone(),
            interface: packet.interface.clone(),
            l_3_protocol: packet.l_3_protocol.clone(), // Populate from PacketInfos
            ip_source: packet.layer_3_infos.ip_source.clone(),
            ip_source_type: packet.layer_3_infos.ip_source_type.clone(),
            ip_destination: packet.layer_3_infos.ip_destination.clone(),
            ip_destination_type: packet.layer_3_infos.ip_destination_type.clone(),
            l_4_protocol: packet.layer_3_infos.l_4_protocol.clone(),
            port_source: packet.layer_3_infos.layer_4_infos.port_source.clone(),
            port_destination: packet.layer_3_infos.layer_4_infos.port_destination.clone(),
            l_7_protocol: packet.layer_3_infos.layer_4_infos.l_7_protocol.clone(),
            packet_size: packet.packet_size,
            count,
        }
    }
}
