//! Module pour la gestion de graphes basés sur des matrices de flux au format v-network-graph.
//!
//! Ce module offre des structures et des fonctions pour manipuler des données structurées
//! destinées à la représentation graphique de réseaux. Il permet la création et la gestion de graphes
//! où les nœuds représentent des entités réseau (par exemple, des adresses MAC) et les arêtes
//! représentent des connexions entre ces entités, enrichies d'informations de protocole.
//!
//! # Exemple
//!
//! ```rust
//! use votre_crate::module_graph;
//! let mut graph_builder = module_graph::GraphBuilder::new();
//! graph_builder.add_node("00:1B:44:11:3A:B7".to_string());
//! graph_builder.add_edge("00:1B:44:11:3A:B7".to_string(), "00:1B:44:11:3A:B8".to_string(), "TCP".to_string());
//! let graph_data = graph_builder.build_graph_data();
//! println!("{:?}", graph_data);
//! ```
//!
//! # Fonctionnalités
//! - Construction et gestion de graphes de réseau.
//! - Ajout et gestion de nœuds et d'arêtes avec gestion des doublons.
//! - Sérialisation des données du graphe en format JSON pour une intégration facile avec des systèmes frontaux.

use std::{collections::HashMap, fmt};

use log::error;
use serde::Serialize;
use tauri::State;

use crate::tauri_state::SonarState;

/// Représente les données d'un graphe, incluant nœuds et arêtes.
///
/// Chaque `GraphData` contient une collection de nœuds (`nodes`) et d'arêtes (`edges`),
/// organisées dans des `HashMaps` pour un accès rapide par nom.
#[derive(Serialize)]
struct GraphData {
    /// Les nœuds du graphe, stockés dans une HashMap où la clé est le nom du nœud et la valeur est le nœud lui-même.
    nodes: HashMap<String, Node>,
    /// Les arêtes du graphe, stockées dans une HashMap où la clé est le nom de l'arête et la valeur est l'arête elle-même.
    edges: HashMap<String, Edge>,
}

/// Représente un nœud dans un graphe, identifié par son nom.
///
/// Un `Node` est typiquement utilisé pour représenter une entité réseau, telle qu'une adresse MAC,
/// dans le graphe.
#[derive(Serialize, Clone, Hash, Eq, PartialEq)]
struct Node {
    /// L'adresse MAC du nœud.
    mac: String,
    /// L'adresse IP du nœud.
    ip: String,
    /// Le port du nœud.
    port: String,
}

/// Représente une arête dans un graphe, définie par une source, une cible et un label.
///
/// Les `Edge`s servent à représenter les relations entre les nœuds, avec des informations supplémentaires
/// telles que le protocole utilisé pour la communication.
#[derive(Serialize, Clone)]
struct Edge {
    /// Le nœud source de l'arête.
    source: String,
    /// Le nœud cible de l'arête.
    target: String,
    /// Le label de l'arête, représentant le protocole de la couche 3.
    label: String,
}

/// Permet la construction de graphes à partir de données de flux.
///
/// `GraphBuilder` est utilisé pour ajouter progressivement des nœuds et des arêtes
/// et construire les données du graphe.
struct GraphBuilder {
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
}

impl GraphBuilder {
    /// Crée une nouvelle instance de `GraphBuilder`.
    ///
    /// Initialise des collections vides pour les nœuds et les arêtes, et prépare le compteur d'arêtes.
    fn new() -> Self {
        GraphBuilder {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Ajoute un nœud au graphe en utilisant une adresse MAC comme identifiant.
    ///
    /// # Arguments
    ///
    /// * `mac_address` - Une chaîne de caractères représentant l'adresse MAC du nœud.
    ///
    /// # Exemples
    ///
    /// ```
    /// graph_builder.add_node("00:1B:44:11:3A:B7".to_string());
    /// ```
    ///
    /// Si le nœud existe déjà, cette fonction ne modifie pas le graphe.
    fn add_node(&mut self, mac: String, ip: String, port: String) {
        let node_key = format!("{}-{}-{}", mac, ip, port);
        if !self.nodes.contains_key(&node_key) {
            self.nodes.insert(
                node_key.clone(),
                Node {
                    mac,
                    ip,
                    port,
                },
            );
        }
    }
    

    /// Ajoute une arête au graphe, créant les nœuds source et cible si nécessaire.
    ///
    /// # Arguments
    ///
    /// * `source` - Le nœud source de l'arête.
    /// * `target` - Le nœud cible de l'arête.
    /// * `label` - Le label de l'arête, représentant le protocole de la couche 3.
    ///
    /// # Exemples
    ///
    /// ```
    /// graph_builder.add_edge("00:1B:44:11:3A:B7".to_string(), "00:1B:44:11:3A:B8".to_string(), "TCP".to_string());
    /// ```
    ///
    /// Cette méthode assure l'unicité des arêtes dans le graphe.
    fn add_edge(&mut self, source_mac: String, source_ip: String, source_port: String, target_mac: String, target_ip: String, target_port: String, label: String) {
        self.add_node(source_mac.clone(), source_ip.clone(), source_port.clone());
        self.add_node(target_mac.clone(), target_ip.clone(), target_port.clone());
    
        let edge_key = format!("{}-{}:{}->{}-{}:{}", source_mac, source_ip, source_port, target_mac, target_ip, target_port);
        if !self.edges.contains_key(&edge_key) {
            self.edges.insert(
                edge_key,
                Edge {
                    source: format!("{}-{}-{}", source_mac, source_ip, source_port),
                    target: format!("{}-{}-{}", target_mac, target_ip, target_port),
                    label,
                },
            );
        }
    }
    

    /// Construit et retourne les données du graphe à partir des éléments ajoutés.
    ///
    /// Cette méthode assemble les nœuds et les arêtes ajoutés en un objet `GraphData`.
    ///
    /// # Retour
    ///
    /// Retourne une instance de `GraphData` contenant tous les nœuds et arêtes ajoutés.
    fn build_graph_data(&self) -> GraphData {
        GraphData {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

impl fmt::Debug for GraphData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "graphData {{ nodes {{")?;
        for (key, value) in &self.nodes {
            writeln!(f, "    {}: {{ mac: \"{}\", ip: \"{}\", port: : \"{}\"}} ", key, value.mac, value.ip, value.port)?;
        }
        writeln!(f, "  }}, edges  {{")?;
        for (key, value) in &self.edges {
            writeln!(
                f,
                "    {}: {{ source: \"{}\", target: \"{}\", label: \"{}\" }},",
                key, value.source, value.target, value.label
            )?;
        }
        writeln!(f, "  }} }}")
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Node {{ mac: \"{}\" }}", self.mac)
    }
}

impl fmt::Debug for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Edge {{ source: \"{}\", target: \"{}\" }}",
            self.source, self.target
        )
    }
}

/// Récupère les données du graphe à partir de l'état partagé.
///
/// Cette fonction prend une référence à l'état partagé contenant les données de paquet et
/// construit un graphe à partir de ces données. Les nœuds du graphe sont les adresses MAC des
/// sources et destinations des paquets, et les arêtes représentent les connexions entre ces nœuds,
/// avec le label de l'arête représentant le protocole de la couche 3.
pub fn get_graph_data(shared_vec_infopackets: State<SonarState>) -> Result<String, String> {
    // Tentative de verrouillage du mutex sur l'état partagé
    match shared_vec_infopackets.0.lock() {
        Ok(matrice) => {
            let mut graph_builder = GraphBuilder::new();

            // Traitez vos données de paquet ici pour peupler les nœuds et les arêtes
            for packet in matrice.iter() {
                let source_mac = packet.0.mac_address_source.clone();
                let target_mac = packet.0.mac_address_destination.clone();
                let l3_protocol_label = packet.0.l_3_protocol.clone(); 
                let source_ip = packet.0.layer_3_infos.ip_source.clone().unwrap_or_default();
                let target_ip = packet.0.layer_3_infos.ip_destination.clone().unwrap_or_default();
                let source_port = packet.0.layer_3_infos.layer_4_infos.port_source.clone().unwrap_or_default();
                let target_port = packet.0.layer_3_infos.layer_4_infos.port_destination.clone().unwrap_or_default();
        
                graph_builder.add_edge(source_mac, source_ip, source_port, target_mac, target_ip, target_port, l3_protocol_label);
            }

            let graph_data = graph_builder.build_graph_data();

            // Sérialisez les données du graphe en une chaîne JSON
            let json_data = serde_json::to_string(&graph_data).map_err(|e| {
                let err_msg = format!("Erreur de sérialisation : {}", e);
                error!("{}", err_msg);
                err_msg
            })?;
            println!("{:?}", graph_data);

            Ok(json_data)
        }
        Err(_) => {
            let err_msg = "Impossible de verrouiller le mutex".to_string();
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph_builder = GraphBuilder::new();
        let mac_address = "00:1B:44:11:3A:B7".to_string();
        let ip_address = "192.168.1.1".to_string();
        let port = "8080".to_string();
        graph_builder.add_node(mac_address.clone(), ip_address.clone(), port.clone());

        let node_key = format!("{}-{}-{}", mac_address, ip_address, port);
        assert!(graph_builder.nodes.contains_key(&node_key));
        let node = graph_builder.nodes.get(&node_key).unwrap();
        assert_eq!(node.mac, node_key);
        assert_eq!(node.ip, ip_address);
        assert_eq!(node.port, port);
    }

    #[test]
    fn test_add_edge() {
        let mut graph_builder = GraphBuilder::new();
        let source_mac = "00:1B:44:11:3A:B7".to_string();
        let target_mac = "00:1B:44:11:3A:B8".to_string();
        let source_ip = "192.168.1.1".to_string();
        let target_ip = "192.168.1.2".to_string();
        let source_port = "8080".to_string();
        let target_port = "8081".to_string();
        let label = "TCP".to_string();

        graph_builder.add_edge(source_mac.clone(), source_ip.clone(), source_port.clone(), target_mac.clone(), target_ip.clone(), target_port.clone(), label.clone());

        let edge_key = format!("{}-{}:{}->{}-{}:{}", source_mac, source_ip, source_port, target_mac, target_ip, target_port);
        assert!(graph_builder.edges.contains_key(&edge_key));
        let edge = graph_builder.edges.get(&edge_key).unwrap();
        assert_eq!(edge.source, format!("{}-{}-{}", source_mac, source_ip, source_port));
        assert_eq!(edge.target, format!("{}-{}-{}", target_mac, target_ip, target_port));
        assert_eq!(edge.label, label);
    }

    #[test]
    fn test_build_graph_data() {
        let mut graph_builder = GraphBuilder::new();
        graph_builder.add_node(
            "00:1B:44:11:3A:B7".to_string(),
            "192.168.1.1".to_string(),
            "8080".to_string()
        );
        graph_builder.add_node(
            "00:1B:44:11:3A:B8".to_string(),
            "192.168.1.2".to_string(),
            "8081".to_string()
        );
        graph_builder.add_edge(
            "00:1B:44:11:3A:B7".to_string(), "192.168.1.1".to_string(), "8080".to_string(),
            "00:1B:44:11:3A:B8".to_string(), "192.168.1.2".to_string(), "8081".to_string(),
            "TCP".to_string()
        );

        let graph_data = graph_builder.build_graph_data();

        // Vérifiez que les données du graphe contiennent bien les éléments ajoutés
        assert_eq!(graph_data.nodes.len(), 2);
        assert_eq!(graph_data.edges.len(), 1);
    }

    #[test]
    fn test_duplicate_nodes_are_not_added() {
        let mut graph_builder = GraphBuilder::new();
        let mac_address = "00:1B:44:11:3A:B7".to_string();
        let ip_address = "192.168.1.1".to_string();
        let port = "8080".to_string();
        // Ajoutez le même nœud deux fois.
        graph_builder.add_node(mac_address.clone(), ip_address.clone(), port.clone());
        graph_builder.add_node(mac_address.clone(), ip_address.clone(), port.clone());
    
        // Vérifiez qu'un seul nœud a été ajouté.
        assert_eq!(graph_builder.nodes.len(), 1);
    }
    
    #[test]
    fn test_duplicate_edges_are_not_added() {
        let mut graph_builder = GraphBuilder::new();
        // Ajoutez la même arête deux fois avec les informations complètes.
        graph_builder.add_edge(
            "00:1B:44:11:3A:B7".to_string(), "192.168.1.1".to_string(), "8080".to_string(),
            "00:1B:44:11:3A:B8".to_string(), "192.168.1.2".to_string(), "8081".to_string(),
            "TCP".to_string()
        );
        graph_builder.add_edge(
            "00:1B:44:11:3A:B7".to_string(), "192.168.1.1".to_string(), "8080".to_string(),
            "00:1B:44:11:3A:B8".to_string(), "192.168.1.2".to_string(), "8081".to_string(),
            "TCP".to_string()
        );
    
        // Vérifiez qu'une seule arête a été ajoutée.
        assert_eq!(graph_builder.edges.len(), 1);
    }
    
}
