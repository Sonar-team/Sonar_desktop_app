//! # Module de capture de paquets
//!
//! Ce module fournit des fonctionnalités pour capturer le trafic réseau, soit sur toutes les interfaces réseau disponibles,
//! soit sur une interface spécifique choisie par l'utilisateur. L'intégration avec Tauri permet une interaction fluide
//! avec l'interface utilisateur pour démarrer ou arrêter la capture à la demande.

pub mod capture_packet;

use capture_packet::{all_interfaces, one_interface};
use log::info;
use tauri::AppHandle;

/// Démarre la capture de paquets jusqu'à interruption par l'utilisateur.
///
/// Cette fonction détermine si la capture doit être effectuée sur toutes les interfaces réseau ou juste une spécifique,
/// puis démarre le processus de capture approprié. Elle est conçue pour être appelée depuis une interface utilisateur Tauri,
/// permettant aux utilisateurs de contrôler la capture de trafic réseau directement depuis l'interface graphique.
///
/// # Arguments
///
/// * `app` - Handle vers l'application Tauri, utilisé pour interagir avec l'état de l'application et l'interface utilisateur.
/// * `interface` - Le nom de l'interface réseau sur laquelle effectuer la capture, ou une chaîne spéciale pour indiquer toutes les interfaces.
/// * `state` - L'état partagé de l'application, encapsulé dans un objet `SonarState` pour maintenir les données à travers l'application.
pub fn scan_until_interrupt(app: AppHandle, interface: &str) {
    // println!("Démarrage de la capture de paquets sur l'interface : {}", interface);
    match check_interface(interface) {
        true => all_interfaces(app),
        false => one_interface(app, interface),
    }
}

/// Vérifie si l'utilisateur a demandé la capture sur toutes les interfaces réseau.
///
/// Cette fonction interne sert à déterminer si l'argument `interface` correspond à la demande de capture sur toutes les interfaces.
/// Elle renvoie `true` si l'utilisateur souhaite capturer le trafic sur toutes les interfaces réseau, ou `false` sinon.
///
/// # Arguments
///
/// * `interface` - Le nom de l'interface réseau spécifié par l'utilisateur, à vérifier.
fn check_interface(interface: &str) -> bool {
    matches!(interface, "Toutes les interfaces")
}

#[tauri::command(async, rename_all = "snake_case")]
pub fn get_selected_interface(app: AppHandle, interface_name: String) {
    info!("Interface sélectionée: {}", interface_name);
    scan_until_interrupt(app, &interface_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_interface_all() {
        // Vérifie que la fonction renvoie vrai lorsque l'interface est "Toutes les interfaces"
        assert!(check_interface("Toutes les interfaces"));
    }

    #[test]
    fn test_check_interface_specific() {
        // Vérifie que la fonction renvoie faux lorsque l'interface est spécifique
        assert!(!check_interface("en0"));
    }
}
