use std::sync::Mutex;

use log::error;
use tauri::{AppHandle, Manager};

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
/// 
/// # Diagramme d'état
///
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
/// stateDiagram-v2
///     [*] --> AcquireState
///     AcquireState --> LockMutex
///     LockMutex --> LockAcquired
///     LockAcquired --> AccessMatrice : Lock Successful
///     LockAcquired --> HandleLockError : Lock Failed
///     AccessMatrice --> SerializeData
///     SerializeData --> SerializationSuccess : Serialization Successful
///     SerializeData --> SerializationFailure : Serialization Failed
///     SerializationSuccess --> ReturnOk
///     SerializationFailure --> LogSerializationError
///     LogSerializationError --> ReturnErr
///     HandleLockError --> LogLockError
///     LogLockError --> ReturnErr
///     ReturnOk --> [*]
///     ReturnErr --> [*]
/// ```
///

pub fn get_matrice_data(app: AppHandle) -> Result<String, String> {
    let state = app.state::<Mutex<SonarState>>(); // Acquire a lock
    let state_guard = state.lock().unwrap();
    // Directly access the `matrice` field from `SonarState`
    match serde_json::to_string(state_guard.get_matrice()) {
        Ok(serialized_data) => {
            // Successfully serialized the matrice to a JSON string
            Ok(serialized_data)
        }
        Err(e) => {
            // Handle serialization errors
            let err_msg = format!("Erreur de sérialisation : {}", e);
            error!("{}", err_msg);
            Err(err_msg)
        }
    }
}
