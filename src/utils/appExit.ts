// Fermeture de l'application, protégée contre les demandes concurrentes
// (un seul dialogue à la fois). La confirmation n'est demandée que si des
// modifications ne sont pas enregistrées (#159) ; la capture en cours est
// arrêtée proprement (drainage puis jointure, #158) avant de quitter.
import { ask } from '@tauri-apps/plugin-dialog';
import { exit } from '@tauri-apps/plugin-process';
import { error, info } from '@tauri-apps/plugin-log';
import { Channel, invoke } from '@tauri-apps/api/core';

let exitRequestInProgress = false;

export async function requestAppExit(): Promise<void> {
  if (exitRequestInProgress) {
    return;
  }

  exitRequestInProgress = true;

  try {
    info('Fermeture demandee');

    // En cas de doute (IPC en échec), le relevé est considéré comme modifié :
    // mieux vaut une confirmation de trop qu'un travail perdu.
    const dirty = await invoke<boolean>('is_session_dirty').catch(() => true);

    if (dirty) {
      const confirmed = await ask(
        'Des modifications non enregistrées seront perdues.\nQuitter quand même ?',
        {
          title: 'SONAR',
          kind: 'warning',
        },
      );

      if (!confirmed) {
        info('Fermeture annulee');
        return;
      }
    }

    info('Fermeture confirmee');
    // Arrêt gracieux du pipeline (idempotent depuis Idle). Best-effort :
    // un échec d'arrêt n'empêche pas de quitter.
    try {
      await invoke('stop_capture', { onEvent: new Channel() });
    } catch (stopError) {
      error(`Arrêt de la capture avant fermeture: ${String(stopError)}`);
    }
    await exit(0);
  } catch (closeError) {
    error(`Erreur lors de la fermeture: ${String(closeError)}`);
  } finally {
    exitRequestInProgress = false;
  }
}
