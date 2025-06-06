//! Module pour gérer l'état de Sonar.
//!
//! Ce module fournit les structures nécessaires pour maintenir l'état
//! actuel de l'application Sonar, en particulier pour suivre les trames réseau.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use csv::Writer;
use log::{error, info};
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

// Clé sans `packet_size` pour éviter de considérer les tailles comme des doublons
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
    pub count: u32,             // Nombre de paquets similaires
    pub packet_size_total: u32, // Taille totale cumulée des paquets
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

// Conversion de `PacketInfos` en `PacketKey` pour utiliser comme clé du `HashMap`
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
    // Contient les trames réseau et leur nombre d'occurrences
    pub matrice: HashMap<PacketKey, PacketStats>,
    // Indique si le filtrage des adresses IPv6 est activé
}

impl SonarState {
    // Constructeur pour initialiser `SonarState`
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(SonarState {
            matrice: HashMap::new(),
        }))
    }
    pub fn reset(&mut self) {
        self.matrice.clear();
    }

    // Getter method for matrice
    // pub fn get_matrice(&self) -> &HashMap<PacketKey, PacketStats> {
    //     &self.matrice
    // }

    // Met à jour `matrice` avec un nouveau paquet
    pub fn update_matrice_with_packet(&mut self, new_packet: &PacketInfos) {
        let packet_size = new_packet.packet_size as u32;
        // Crée une clé à partir de `PacketInfos` en utilisant `PacketKey`
        let key = PacketKey::from(new_packet);
        // Insertion ou mise à jour dans la matrice
        self.matrice
            .entry(key)
            .and_modify(|stats| stats.update(packet_size))
            .or_insert(PacketStats::new(packet_size));
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
    pub fn cmd_save_packets_to_csv(&self, file_path: String) -> Result<(), ExportError> {
        let mut wtr = Writer::from_path(&file_path).map_err(|e| {
            error!(
                "Erreur lors de l'ouverture du fichier CSV {} : {}",
                file_path, e
            );
            ExportError::Io(e.to_string())
        })?;

        for (packet_key, stats) in self.matrice.iter() {
            let packet_count = stats.count;
            println!("Packet count: {}", packet_count);
            let packet_csv = PacketInfosFlaten::from_packet_key_and_stats(packet_key, stats);
            wtr.serialize(packet_csv).map_err(|e| {
                error!("Erreur de sérialisation CSV : {:?}", e);
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
    pub fn cmd_save_packets_to_excel(&self, file_path: String) -> Result<(), ExportError> {
        let data = self.matrice.clone();

        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();

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
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;
        }

        for (i, (packet_key, stats)) in data.iter().enumerate() {
            let packet_csv = PacketInfosFlaten::from_packet_key_and_stats(packet_key, stats);

            sheet
                .write_string(i as u32 + 1, 0, &packet_csv.mac_address_source)
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            sheet
                .write_string(i as u32 + 1, 1, &packet_csv.mac_address_destination)
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            sheet
                .write_string(i as u32 + 1, 2, &packet_csv.interface)
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            sheet
                .write_string(i as u32 + 1, 3, &packet_csv.l_3_protocol)
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;

            if let Some(ip_src) = &packet_csv.ip_source {
                sheet
                    .write_string(i as u32 + 1, 4, ip_src)
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(ip_src_type) = &packet_csv.ip_source_type {
                sheet
                    .write_string(i as u32 + 1, 5, ip_src_type.to_string())
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(ip_dst) = &packet_csv.ip_destination {
                sheet
                    .write_string(i as u32 + 1, 6, ip_dst)
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(ip_dst_type) = &packet_csv.ip_destination_type {
                sheet
                    .write_string(i as u32 + 1, 7, ip_dst_type.to_string())
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(l4_proto) = &packet_csv.l_4_protocol {
                sheet
                    .write_string(i as u32 + 1, 8, l4_proto)
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(port_src) = &packet_csv.port_source {
                sheet
                    .write_string(i as u32 + 1, 9, port_src)
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(port_dst) = &packet_csv.port_destination {
                sheet
                    .write_string(i as u32 + 1, 10, port_dst)
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }
            if let Some(l7_proto) = &packet_csv.l_7_protocol {
                sheet
                    .write_string(i as u32 + 1, 11, l7_proto)
                    .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            }

            sheet
                .write_number(i as u32 + 1, 12, packet_csv.packet_size as f64)
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;
            sheet
                .write_number(i as u32 + 1, 13, packet_csv.count as f64)
                .map_err(|e| ExportError::Xlsx(e.to_string()))?;
        }

        workbook
            .save(file_path)
            .map_err(|e| ExportError::Xlsx(e.to_string()))?;

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

/// Modifie `from_packet_infos` pour accepter `PacketKey` et `PacketStats`
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
            packet_size: stats.packet_size_total as usize, // Utilisation de la taille totale cumulée
            count: stats.count,
        }
    }
}
