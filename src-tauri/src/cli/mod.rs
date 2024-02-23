// Utilise la bibliothèque colored pour ajouter de la couleur au texte dans la console.
use colored::Colorize;

/// Affiche une bannière en ASCII art.
///
/// Cette fonction crée une bannière ASCII avec le nom de l'application ou un message de bienvenue.
/// La bannière est ensuite colorée en vert et retournée en tant que `String`.
///
/// # Retourne
///
/// Retourne un `String` contenant la bannière ASCII colorée en vert.
///
/// # Exemples
///
/// ```
/// let banner = print_banner();
/// println!("{}", banner);
/// ```
///
/// Cela affichera la bannière suivante en vert dans la console :
/// ```text
///   _________                           
///  /   _____/ ____   ____ _____ _______
///  \_____  \ /  _ \ /    \\__  \\_  __ \
///  /        (  <_> )   |  \/ __ \|  | \/
/// /_______  /\____/|___|  (____  /__|   
///         \/            \/     \/          
/// ```

pub fn print_banner() -> String {
    // ASCII art banner
    let banner = r"
    _________                           
   /   _____/ ____   ____ _____ _______ 
   \_____  \ /  _ \ /    \\__  \\_  __ \
   /        (  <_> )   |  \/ __ \|  | \/
  /_______  /\____/|___|  (____  /__|   
          \/            \/     \/          
   ";

    // La bannière est colorée en vert avant d'être retournée.
    banner.green().to_string()
}

// Le module tests contient les tests unitaires pour le code.
#[cfg(test)]
mod tests {
    // Importe tout ce qui est nécessaire pour les tests depuis le module parent.
    use super::*;

    /// Teste la fonction print_banner.
    ///
    /// Ce test vérifie que la bannière retournée n'est pas vide et contient les éléments attendus de l'art ASCII.
    #[test]
    fn test_print_banner() {
        // Appelle la fonction print_banner pour récupérer la bannière.
        let banner = print_banner();

        // Vérifie que la chaîne retournée n'est pas vide.
        assert!(!banner.is_empty());

        // Vérifie que la chaîne retournée contient le texte du banner ASCII.
        assert!(banner.contains("_______"));
        assert!(banner.contains("/   _____/"));
        assert!(banner.contains("\\_____  \\"));
        assert!(banner.contains("/        ("));
        assert!(banner.contains("/_______  /"));
        assert!(banner.contains("          \\/"));
    }
}
