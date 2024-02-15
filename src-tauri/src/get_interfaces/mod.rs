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
pub fn get_interfaces() -> Vec<String> {
    // Récupère une liste de toutes les interfaces réseau via le module datalink de pnet.
    let interfaces = datalink::interfaces();
    // Log l'action de récupération des interfaces réseau.
    info!("Récupération des interfaces réseau");

    // Mappe les interfaces à leurs noms ou adresses MAC, les collectant dans un vecteur.
    let mut names: Vec<String> = interfaces
        .iter()
        .map(|iface| {
            #[cfg(target_os = "linux")]
            {
                iface.name.clone() // Retourne le nom de l'interface sous Linux.
            }
            #[cfg(target_os = "windows")]
            {
                format!("Interface MAC: {}", iface.mac.unwrap_or_default()) // Retourne l'adresse MAC de l'interface sous Windows.
            }
        })
        .collect();
 
    // Ajoute une chaîne représentant l'option de sélection de toutes les interfaces.
    let all = String::from("Toutes les interfaces");
    names.push(all);

    // Retourne le vecteur de noms d'interface.
    names
}

#[cfg(test)]
mod tests {
    // Importe la fonction à tester.
    use super::*;

    #[test]
    fn test_get_interfaces() {
        // Appelle la fonction à tester.
        let interface_names = get_interfaces();

        // Vérifie que le vecteur de noms d'interface n'est pas vide.
        assert!(!interface_names.is_empty());

        #[cfg(target_os = "windows")]
        assert!(interface_names.iter().any(|name| name.starts_with("Interface MAC: ")));

        // Vérifie que le dernier élément du vecteur est "Toutes les interfaces".
        assert_eq!(interface_names.last(), Some(&String::from("Toutes les interfaces")));
    }
}
