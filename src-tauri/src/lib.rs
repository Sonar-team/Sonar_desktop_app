//! # Sonar - Outil de surveillance réseau
//!
//! `sonar` est une bibliothèque et une application permettant la surveillance et l'analyse du trafic réseau.
//! Elle offre des fonctionnalités pour capturer, analyser et enregistrer les paquets réseau sur différentes interfaces.

/// Gestion de l'interface de ligne de commande.
///
/// Ce module fournit les fonctionnalités nécessaires pour interpréter les commandes de l'utilisateur,
/// traiter les arguments de ligne de commande et contrôler le comportement de l'application en fonction de ces entrées.
pub mod cli;

/// Récupération des interfaces réseau.
///
/// Le module `get_interfaces` permet d'identifier et de lister les interfaces réseau disponibles sur la machine.
/// Il est essentiel pour permettre à l'utilisateur de sélectionner l'interface sur laquelle écouter le trafic réseau.
pub mod get_interfaces;

/// Construction et gestion de la matrice des paquets.
///
/// Ce module, `get_matrice`, est responsable de la création et de la gestion d'une structure de données
/// pour organiser et stocker les informations sur les paquets capturés, facilitant leur analyse ultérieure.
pub mod get_matrice;

/// Sauvegarde des paquets capturés.
///
/// Le module `save_packets` offre des fonctionnalités pour enregistrer les paquets réseau capturés.
/// Il permet la persistance des données pour une analyse postérieure ou pour la documentation.
pub mod save_packets;

/// Capture et analyse des paquets réseau.
///
/// `sniff` est le cœur de l'application, responsable de la capture des paquets réseau sur une interface spécifiée,
/// et de l'analyse de ces paquets pour en extraire des informations utiles.
pub mod sniff;

/// Gestion de l'état pour l'intégration avec Tauri.
///
/// Ce module, `tauri_state`, permet de gérer l'état partagé entre les différentes composantes de l'application,
/// en particulier lors de l'utilisation de Tauri pour créer une interface graphique.
pub mod tauri_state;

pub mod get_hostname;

// Les tests unitaires pour valider la fonctionnalité de chaque composant de `sonar`.
mod tests_unitaires;
