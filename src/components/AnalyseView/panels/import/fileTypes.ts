// Extensions reconnues par le panneau d'import et routage des fichiers
// déposés (drag & drop) selon le mode du panneau ('csv' | 'pcap').
export const PCAP_EXTENSIONS = ['pcap', 'pcapng', 'cap'];
export const MATRIX_EXTENSIONS = ['csv', 'xlsx'];

function extensionOf(path: string): string {
  return (path.split('.').pop() ?? '').toLowerCase();
}

/** Ajoute des chemins à une liste en écartant ceux déjà présents : un même
 *  fichier resélectionné ou redéposé ne doit apparaître qu'une fois dans la
 *  liste affichée (#161). L'ordre d'arrivée est préservé.
 *
 *  Déduplication par chaîne uniquement : le webview n'a pas accès au système
 *  de fichiers et ne peut donc pas reconnaître deux désignations du même
 *  fichier (chemin relatif, lien symbolique). La garantie de fond — un
 *  fichier n'est jamais compté deux fois dans la matrice — est tenue par
 *  `dedupe_paths_by_identity` côté Rust, qui canonicalise avant l'import. */
export function appendUniquePaths(existing: string[], incoming: string[]): string[] {
  return Array.from(new Set([...existing, ...incoming]));
}

export type DroppedPathsRouting =
  | { kind: 'csv'; path: string }
  | { kind: 'files'; pcaps: string[]; matrices: string[] }
  | { kind: 'ignored' };

/** Répartit les chemins déposés selon le mode du panneau : un seul .csv en
 *  mode labels (le premier trouvé), ou des captures/matrices en mode pcap. */
export function routeDroppedPaths(paths: string[], mode: string): DroppedPathsRouting {
  if (mode === 'csv') {
    const csv = paths.find((p) => extensionOf(p) === 'csv');
    return csv ? { kind: 'csv', path: csv } : { kind: 'ignored' };
  }

  const pcaps = paths.filter((p) => PCAP_EXTENSIONS.includes(extensionOf(p)));
  const matrices = paths.filter((p) => MATRIX_EXTENSIONS.includes(extensionOf(p)));
  if (pcaps.length === 0 && matrices.length === 0) return { kind: 'ignored' };
  return { kind: 'files', pcaps, matrices };
}
