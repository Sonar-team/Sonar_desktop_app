// Persistance des projets récents et du dernier dossier utilisé via
// tauri-plugin-store (#159). Préférences UI uniquement : jamais bloquant —
// toute erreur est tracée et l'appelant continue. La config de capture,
// elle, reste sur la persistance atomique côté Rust.
import { LazyStore } from '@tauri-apps/plugin-store';
import { warn } from '@tauri-apps/plugin-log';

import { dirnameOf, pushRecent } from './recentProjectsCore';

const STORE_FILE = 'ui-preferences.json';
const KEY_RECENTS = 'recentProjects';

const store = new LazyStore(STORE_FILE);

/** Enregistre un projet ouvert/sauvegardé en tête des récents. */
export async function recordRecentProject(path: string): Promise<void> {
  try {
    const list = (await store.get<string[]>(KEY_RECENTS)) ?? [];
    await store.set(KEY_RECENTS, pushRecent(list, path));
    await store.save();
  } catch (err) {
    warn(`recents non enregistrés: ${String(err)}`);
  }
}

/** Liste des projets récents, du plus récent au plus ancien. */
export async function getRecentProjects(): Promise<string[]> {
  try {
    return (await store.get<string[]>(KEY_RECENTS)) ?? [];
  } catch (err) {
    warn(`recents illisibles: ${String(err)}`);
    return [];
  }
}

/** Dossier du projet le plus récent, pour préremplir les dialogues. */
export async function getLastProjectDir(): Promise<string | undefined> {
  const [latest] = await getRecentProjects();
  return latest ? dirnameOf(latest) : undefined;
}
