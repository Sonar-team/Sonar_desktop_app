<!--
  Racine de l'application : fond, overlay bloquant pendant les imports,
  barre du haut (TopBar), vue routée et barre de statut. Gère aussi la
  confirmation de fermeture de la fenêtre.
-->
<template>

  <!-- Overlay bloquant -->
  <div v-if="captureStore.isImporting" class="input-blocker"/> 
  
  <div class="bg"></div>
  <router-view></router-view>
</template>

<style>
:root {
  height: 100vh;
}

html, body {
  height: 100%;
  margin: 0;
  padding: 0;
  background-color: #2a2a2a;
}

#app {
  height: 100%;
}

.bg {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: #1a1a1a;
  z-index: -1;
}

.input-blocker {
  position: fixed;
  inset: 0;
  z-index: 10000;
  cursor: wait;
  background: transparent;
}
</style>

<script>
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Channel, invoke } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import { error, info } from '@tauri-apps/plugin-log';
import { useCaptureStore } from './store/capture'
import { requestAppExit } from './utils/appExit';
import { displayCaptureError } from './errors/capture';

const appWindow = getCurrentWebviewWindow()

export default {

  setup() {
    const captureStore = useCaptureStore();
    return { captureStore };
  },
  data() {
    return {
      // Add a data property for the unlisten function
      unlistenCloseEvent: null,
    };
  },

  async mounted() {
    // Set up the close event listener
    this.unlistenCloseEvent = await appWindow.onCloseRequested(async (event) => {
      info("close requested")
      event.preventDefault();
      await requestAppExit();
    });

    // Récupération après crash (#159) : si la session précédente ne s'est
    // pas fermée proprement, le backend propose son dernier autosave.
    try {
      const offer = await invoke('get_recovery_offer');
      if (offer) {
        const restore = await ask(
          "La session précédente ne s'est pas fermée correctement.\n" +
            'Récupérer le dernier relevé sauvegardé automatiquement ?',
          { title: 'SONAR', kind: 'warning' },
        );
        if (restore) {
          const onEvent = new Channel();
          this.captureStore.setChannel(onEvent);
          await invoke('open_project', { path: offer, onEvent });
          await this.captureStore.refreshHasData();
          info('Relevé récupéré depuis l\'autosave');
        }
      }
    } catch (recoveryError) {
      error(`Récupération impossible: ${String(recoveryError)}`);
      await displayCaptureError(recoveryError);
    }
  },

  beforeUnmount() {
    // Call the unlisten function when the component is unmounted
    if (this.unlistenCloseEvent) {
      this.unlistenCloseEvent();
    }
  }
};
</script>
