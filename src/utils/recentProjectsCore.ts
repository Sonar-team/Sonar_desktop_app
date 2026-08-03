// Logique pure des projets récents (#159), sans dépendance au plugin
// store : testable sous `deno test` (les modules @tauri-apps/plugin-* ne se
// résolvent pas tous dans cet environnement, cf. forceLayout).

export const MAX_RECENT_PROJECTS = 10;

/** Ajoute `path` en tête de liste, dédupliqué, plafonné à `max` entrées. */
export function pushRecent(
  list: string[],
  path: string,
  max = MAX_RECENT_PROJECTS,
): string[] {
  return [path, ...list.filter((entry) => entry !== path)].slice(0, max);
}

/** Dossier parent d'un chemin, séparateurs / et \ confondus ; `undefined`
 * si le chemin n'a pas de parent exploitable. */
export function dirnameOf(path: string): string | undefined {
  const cut = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return cut > 0 ? path.slice(0, cut) : undefined;
}
