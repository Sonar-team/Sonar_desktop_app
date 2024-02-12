//! # Module Get Matrice
//!
//! Ce module est responsable de la gestion et de la récupération des données structurées
//! nécessaires pour la visualisation du trafic réseau. Il sert de conteneur pour les sous-modules
//! qui traitent spécifiquement les données de matrice et de graphique.

/// Module pour obtenir des données structurées prêtes à être utilisées dans des visualisations graphiques.
///
/// Ce sous-module traite les données capturées pour générer des structures de données
/// qui peuvent être directement utilisées pour alimenter des graphiques ou des visualisations,
/// facilitant ainsi la compréhension des motifs de trafic réseau.
pub mod get_graph_data;

/// Module pour obtenir les données de la matrice de trafic réseau.
///
/// Ce sous-module se concentre sur l'extraction et la structuration des données nécessaires
/// pour construire une "matrice de trafic", qui peut être utilisée pour analyser les interactions
/// entre différents nœuds dans le réseau. Cela inclut la préparation des données pour des analyses
/// de connectivité, de volume de trafic, et d'autres mesures clés.
pub mod get_matrice_data;
