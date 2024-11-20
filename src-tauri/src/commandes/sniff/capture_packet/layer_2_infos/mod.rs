//! Ce module fournit des utilitaires pour le traitement des paquets, y compris l'extraction et l'affichage des informations de paquet.
//!
//! ## Structures
//!
//! - [`PacketInfos`](struct.PacketInfos.html): Représente des informations détaillées sur un paquet réseau, y compris les adresses MAC, l'interface, et les données des couches 3 et 4.
//!
//! ## Fonctions
//!
//! - [`PacketInfos::new`](struct.PacketInfos.html#method.new): Construit une nouvelle instance de `PacketInfos` à partir d'un paquet Ethernet et du nom de l'interface.
//!
//! ## Implémentation de Trait
//!
//! - [`Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html): L'implémentation de cette trait permet d'afficher les informations du paquet de manière conviviale.

use std::fmt;

use pnet::packet::{ethernet::EthernetPacket, Packet};

use layer_3_infos::{get_layer_3_infos, Layer3Infos};
use serde::Serialize;
pub mod layer_3_infos;

/// Représente des informations détaillées sur un paquet réseau, y compris les adresses MAC, l'interface, et les données des couches 3 et 4.
#[derive(Debug, Default, Serialize, Clone, Eq, Hash, PartialEq)]
pub struct PacketInfos {
    /// Adresse MAC source du paquet.
    pub mac_address_source: String,
    /// Adresse MAC de destination du paquet.
    pub mac_address_destination: String,
    /// Interface réseau par laquelle le paquet a été reçu ou sera envoyé.
    pub interface: String,
    /// Protocole de couche 3 utilisé dans le paquet.
    pub l_3_protocol: String,
    /// Informations détaillées de la couche 3 (par exemple, adresses IP, protocole).
    pub layer_3_infos: Layer3Infos,
    /// La taille totale du paquet en octets.
    pub packet_size: usize,
}

impl PacketInfos {
    /// Construit une nouvelle instance de `PacketInfos` à partir d'un paquet Ethernet et du nom de l'interface.
    ///
    /// # Arguments
    ///
    /// * `interface_name` - Une chaîne de caractères qui contient le nom de l'interface réseau.
    /// * `ethernet_packet` - Une référence au paquet Ethernet à partir duquel extraire les informations.
    pub fn new(interface_name: &String, ethernet_packet: &EthernetPacket<'_>) -> PacketInfos {
        PacketInfos {
            mac_address_source: ethernet_packet.get_source().to_string(),
            mac_address_destination: ethernet_packet.get_destination().to_string(),
            interface: interface_name.to_string(),
            l_3_protocol: ethernet_packet.get_ethertype().to_string(),
            layer_3_infos: get_layer_3_infos(ethernet_packet),
            packet_size: ethernet_packet.packet().len(), // Initialize packet size with total packet length
        }
    }
}

/// Construit une nouvelle instance de `PacketInfos` à partir d'un paquet Ethernet et du nom de l'interface.
///
/// # Arguments
///
/// * `interface_name` - Une chaîne de caractères qui contient le nom de l'interface réseau.
/// * `ethernet_packet` - Une référence au paquet Ethernet à partir duquel extraire les informations.
///
/// # Exemple
///
/// ```rust
/// use pnet::packet::ethernet::EthernetPacket;
/// use packet_infos::PacketInfos;
///
/// let interface_name = String::from("eth0");
/// let ethernet_packet_data: &[u8] = &[/* données du paquet Ethernet */];
/// if let Some(ethernet_packet) = EthernetPacket::new(ethernet_packet_data) {
///     let packet_infos = PacketInfos::new(&interface_name, &ethernet_packet);
///     println!("Packet Infos: {:?}", packet_infos);
/// }
/// ```

impl fmt::Display for PacketInfos {
    /// Formate les informations du paquet pour l'affichage.
    ///
    /// # Arguments
    ///
    /// * `f` - Une référence mutable à un `Formatter`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Implement the formatting for PacketInfos here, including the new packet size field
        writeln!(f, "MAC Source: {}", self.mac_address_source)?;
        writeln!(f, "MAC Destination: {}", self.mac_address_destination)?;
        writeln!(f, "L2 Interface: {}", self.interface)?;
        writeln!(f, "L3 Protocol: {}", self.l_3_protocol)?;
        writeln!(
            f,
            "IP Source: {}",
            self.layer_3_infos.ip_source.as_deref().unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "IP Destination: {}",
            self.layer_3_infos
                .ip_destination
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "Port Destination: {}",
            self.layer_3_infos
                .layer_4_infos
                .port_destination
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "Port Source: {}",
            self.layer_3_infos
                .layer_4_infos
                .port_source
                .as_deref()
                .unwrap_or("N/A")
        )?;
        writeln!(
            f,
            "L4 Protocol: {}",
            self.layer_3_infos.l_4_protocol.as_deref().unwrap_or("N/A")
        )?;
        writeln!(f, "Packet Size: {} bytes", self.packet_size)?;
        Ok(())
    }
}
