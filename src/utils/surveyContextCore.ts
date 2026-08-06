/**
 * Logique pure de la qualification du relevé (#154, tranche 2), isolée de
 * Vue et de Tauri pour être testable sans DOM ni IPC.
 *
 * Le contexte (site, capteur, interface) n'existe pas dans les paquets :
 * l'opérateur le déclare au moment de l'arrêt ou de l'enregistrement.
 */

import type { SurveyContext } from '../types/generated/SurveyContext';

/** Vrai si `value` porte une information, pas seulement des blancs. */
function isFilled(value: string | null | undefined): boolean {
  return typeof value === 'string' && value.trim().length > 0;
}

/**
 * Vrai si le contexte ne porte aucune information exploitable.
 *
 * Une chaîne vide ou blanche compte comme absente : c'est la même règle que
 * `SurveyContext::normalized_field` côté Rust, qui ramène une saisie blanche
 * à « non renseigné ». Sans cet alignement, un relevé « qualifié » avec trois
 * espaces ne serait plus jamais proposé à la qualification.
 */
export function isSurveyContextEmpty(context: SurveyContext | null | undefined): boolean {
  if (!context) return true;
  return !isFilled(context.site) && !isFilled(context.sensor) &&
    !isFilled(context.interface);
}

/**
 * Faut-il ouvrir le dialogue de qualification ?
 *
 * Seulement si le relevé n'est pas déjà qualifié : arrêter une capture puis
 * l'enregistrer ne doit pas poser deux fois la même question. Une fois saisi,
 * le contexte est réutilisé tel quel.
 */
export function needsQualification(context: SurveyContext | null | undefined): boolean {
  return isSurveyContextEmpty(context);
}
