import { ask } from '@tauri-apps/plugin-dialog';
import { exit } from '@tauri-apps/plugin-process';
import { error, info } from '@tauri-apps/plugin-log';

let exitRequestInProgress = false;

export async function requestAppExit(): Promise<void> {
  if (exitRequestInProgress) {
    return;
  }

  exitRequestInProgress = true;

  try {
    info('Fermeture demandee');

    const confirmed = await ask('Êtes-vous sûr de vouloir quitter ?', {
      title: 'SONAR',
      kind: 'warning',
    });

    if (confirmed) {
      info('Fermeture confirmee');
      await exit(0);
      return;
    }

    info('Fermeture annulee');
  } catch (closeError) {
    error(`Erreur lors de la fermeture: ${String(closeError)}`);
  } finally {
    exitRequestInProgress = false;
  }
}
