use log::error;
use tauri::State;

use crate::tauri_state::SonarState;

/// Récupère et sérialise les données de trafic réseau depuis l'état partagé.
///
/// Cette fonction tente d'acquérir un verrou sur l'état partagé contenant les informations des paquets
/// et sérialise ces données en une chaîne JSON. Cela permet une transmission facile des données
/// pour la visualisation ou l'analyse ultérieure.
///
/// # Arguments
///
/// * `shared_vec_infopackets` - Un état partagé (`State<SonarState>`) contenant les données de trafic à sérialiser.
///
/// # Retour
///
/// Cette fonction retourne `Ok(String)` contenant les données sérialisées en cas de succès,
/// ou `Err(String)` avec un message d'erreur en cas d'échec.
///
/// # Erreurs
///
/// Retourne une erreur si :
/// - La tentative d'acquérir le verrou sur l'état partagé échoue.
/// - La sérialisation des données en JSON échoue.
///
/// # Exemples
///
/// Supposons que vous ayez un état partagé `shared_state` initialisé et passé à cette fonction :
///
/// ```ignore
/// let result = get_matrice_data(shared_state);
/// match result {
///     Ok(json_string) => println!("Données sérialisées : {}", json_string),
///     Err(e) => eprintln!("Erreur : {}", e),
/// }
/// ```
pub fn get_matrice_data(shared_vec_infopackets: State<SonarState>) -> Result<String, String> {
    // Tentative d'acquisition du verrou sur l'état partagé
    match shared_vec_infopackets.0.lock() {
        Ok(matrice) => {
            // Sérialise le hash map en une chaîne JSON
            serde_json::to_string(&*matrice).map_err(|e| {
                let err_msg = format!("Erreur de sérialisation : {}", e);
                error!("{}", err_msg);
                err_msg
            })
        }
        Err(_) => {
            let err_msg = "Échec de verrouillage du mutex".to_string();
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}
