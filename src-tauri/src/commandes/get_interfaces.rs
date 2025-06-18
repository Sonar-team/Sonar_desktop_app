// Utilise le crate log pour les messages de journalisation.
use log::info;
// Utilise le crate pnet pour les opérations réseau.
use pnet::datalink;

/// Récupère les noms de toutes les interfaces réseau sur le système, avec une entrée supplémentaire
/// pour représenter la sélection de toutes les interfaces.
///
/// Cette fonction se sert de `pnet::datalink::interfaces` pour obtenir une liste
/// de toutes les interfaces réseau disponibles sur le système courant. Pour Linux,
/// elle retourne les noms d'interface et pour Windows, elle retourne les adresses MAC
/// des interfaces. Enfin, elle ajoute une chaîne "Toutes les interfaces" à ce vecteur,
/// permettant de représenter l'option de choisir toutes les interfaces dans une interface
/// utilisateur ou un paramètre de configuration.
///
/// # Retours
///
/// Un `Vec<String>` contenant les noms ou adresses MAC de toutes les interfaces réseau
/// trouvées sur le système, plus une entrée "Toutes les interfaces" indiquant l'option
/// de sélection de toutes les interfaces.
///
/// # Exemples
///
/// Utilisation simple :
///
/// ```
/// use sonar_desktop_app::get_interfaces::get_interfaces;
///
/// let interface_names = get_interfaces();
/// for name in interface_names {
///     println!("{}", name);
/// }
/// ```

#[tauri::command(async, rename_all = "snake_case")]
pub fn get_interfaces_tab() -> Vec<String> {
    // Récupère une liste de toutes les interfaces réseau via le module datalink de pnet.
    let interfaces = datalink::interfaces();
    // Log l'action de récupération des interfaces réseau.
    info!("Récupération des interfaces réseau");

    // Mappe les interfaces à leurs noms ou adresses MAC, les collectant dans un vecteur.
    let names: Vec<String> = interfaces
        .iter()
        .filter(|iface| {
            // Filtrer les interfaces inactives
            iface.is_up() && !iface.is_loopback() && !iface.name.starts_with("awdl")
        })
        .map(|iface| {
            // Retourne le nom de l'interface sous Linux.
            #[cfg(target_os = "linux")]
            {
                iface.name.clone()
            }
            // Retourne le nom de l'interface sous Windows.
            #[cfg(target_os = "windows")]
            {
                iface.name.clone()
            }
            // Pour macOS et autres systèmes, utilise le nom de l'interface
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                // Ajoute une description plus claire pour les interfaces
                if iface.name.starts_with("en") {
                    format!("Ethernet: {}", iface.name)
                } else if iface.name.starts_with("wi") || iface.name.starts_with("en") {
                    format!("Wi-Fi: {}", iface.name)
                } else if iface.name.starts_with("lo") {
                    format!("Loopback: {}", iface.name)
                } else {
                    iface.name.clone()
                }
            }
        })
        .collect();

    // Ajoute "Toutes les interfaces" au début de la liste
    let mut result = vec!["Toutes les interfaces".to_string()];
    result.extend(names);

    // Retourne le vecteur de noms d'interface.
    result
}

#[cfg(test)]
mod tests {
    // Importe la fonction à tester.
    use super::*;

    #[test]
    fn test_get_interfaces() {
        // Appelle la fonction à tester.
        let interface_names = get_interfaces_tab();

        // Vérifie que le vecteur de noms d'interface n'est pas vide.
        assert!(!interface_names.is_empty());

        #[cfg(target_os = "windows")]
        assert!(interface_names
            .iter()
            .any(|name| name.starts_with("Interface MAC: ")));

        // Vérifie que le dernier élément du vecteur est "Toutes les interfaces".
        assert_eq!(
            interface_names.last(),
            Some(&String::from("Toutes les interfaces"))
        );
    }
}
