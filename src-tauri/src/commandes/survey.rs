//! Qualification du relevé : où et par quoi il a été capté (#154, tranche 2).
//!
//! Ces informations n'existent pas dans le paquet. Elles sont déclarées par
//! l'opérateur au moment de l'arrêt de la capture ou de l'enregistrement — à
//! la manière de Wireshark, pas en configuration préalable (arbitrage du
//! 04/08/2026). Elles décrivent le relevé entier et restent **hors de la clé**
//! d'identité des flux et des nœuds : le même paquet peut être vu par
//! plusieurs capteurs, et mettre le capteur dans la clé empêcherait de
//! reconnaître ce recouvrement.
//!
//! Le contexte vit dans `FlowMatrix::context` — c'est une propriété du relevé,
//! pas de la session de capture. De là il part dans le préambule `#SFMS` de
//! tout export CSV et dans le `manifest.json` du projet `.sonar`.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{State, command};
use ts_rs::TS;

use crate::{
    errors::CaptureStateError,
    state::{capture::CaptureState, flow_matrix::FlowMatrix},
};

/// Contexte de relevé traversant l'IPC. Miroir de
/// [`sonar_flows_core::sfms::SurveyContext`], qui ne dérive ni `serde` ni
/// `ts-rs` (le cœur écrit le préambule lui-même). Voir `ManifestSurveyContext`
/// pour le même miroir côté manifest JSON.
/// Exporté par `cargo test export_ipc_bindings` comme le reste du contrat.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, TS)]
#[ts(rename = "SurveyContext")]
pub struct SurveyContextDto {
    #[ts(optional = nullable)]
    pub site: Option<String>,
    #[ts(optional = nullable)]
    pub sensor: Option<String>,
    #[ts(optional = nullable)]
    pub interface: Option<String>,
}

impl From<&sonar_flows_core::sfms::SurveyContext> for SurveyContextDto {
    fn from(context: &sonar_flows_core::sfms::SurveyContext) -> Self {
        // Destructuration exhaustive : un champ ajouté au cœur casse la
        // compilation ici plutôt que de disparaître du dialogue.
        let sonar_flows_core::sfms::SurveyContext {
            site,
            sensor,
            interface,
        } = context;
        Self {
            site: site.clone(),
            sensor: sensor.clone(),
            interface: interface.clone(),
        }
    }
}

/// Contexte courant du relevé, pour préremplir le dialogue de qualification.
#[command(async)]
pub fn get_survey_context(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
) -> Result<SurveyContextDto, CaptureStateError> {
    Ok((&matrice.lock()?.context).into())
}

/// Enregistre le contexte saisi par l'opérateur.
///
/// Les saisies vides ou uniquement blanches valent « non renseigné » — la
/// normalisation est celle du cœur, pour que le préambule écrit et le manifest
/// ne divergent jamais du format publié. Le relevé est marqué modifié : le
/// contexte change ce qu'un enregistrement produirait.
#[command(async)]
pub fn set_survey_context(
    matrice: State<'_, Arc<Mutex<FlowMatrix>>>,
    capture_state: State<'_, Arc<Mutex<CaptureState>>>,
    context: SurveyContextDto,
) -> Result<SurveyContextDto, CaptureStateError> {
    use sonar_flows_core::sfms::SurveyContext;

    let normalized = SurveyContext {
        site: SurveyContext::normalized_field(context.site.as_deref()),
        sensor: SurveyContext::normalized_field(context.sensor.as_deref()),
        interface: SurveyContext::normalized_field(context.interface.as_deref()),
    };

    let changed = {
        let mut matrix = matrice.lock()?;
        let changed = matrix.context != normalized;
        matrix.context = normalized.clone();
        changed
    };
    // Ne salir la session que si le contexte a réellement bougé : rouvrir le
    // dialogue et valider sans rien changer ne doit pas réclamer une
    // sauvegarde.
    if changed {
        capture_state.lock()?.mark_dirty();
    }

    Ok((&normalized).into())
}

#[cfg(test)]
mod tests {
    use sonar_flows_core::sfms::SurveyContext;

    /// La normalisation appliquée par la commande est bien celle du cœur :
    /// blancs de bord retirés, saisie vide ou blanche ramenée à « non
    /// renseigné ». Sans quoi le préambule et le manifest divergeraient.
    #[test]
    fn blank_input_normalizes_to_absent() {
        assert_eq!(SurveyContext::normalized_field(Some("  ")), None);
        assert_eq!(SurveyContext::normalized_field(Some("")), None);
        assert_eq!(SurveyContext::normalized_field(None), None);
        assert_eq!(
            SurveyContext::normalized_field(Some("  Usine Nord  ")),
            Some("Usine Nord".to_string())
        );
    }
}
