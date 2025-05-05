<template>
  <div class="top-bar">
    <button class="image-btn" @click="start" title="Démarrer" :disabled="isRunning">
      <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="stop" title="Arrêter" :disabled="!isRunning">
      <img src="/src/assets/stop.svg" alt="Stop" class="icon-img" />
    </button>

    <button class="image-btn"  title="Config" @click="handleConfigClick">
      <img src="/src/assets/config.svg" alt="Config" class="icon-img" />
    </button>
    
    <button class="image-btn" @click="reset" title="Réinitialiser">🔄</button>
    <button class="image-btn" @click="triggerSave" title="Sauvegarder">💾</button>
    <button class="image-btn" @click="toggleComponent" :title="buttonText">📊</button>
    <button class="image-btn" @click="quit" title="Quitter">❌</button>
  </div>

</template>

<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { exit } from '@tauri-apps/plugin-process';
import { info } from '@tauri-apps/plugin-log';

import { displayCaptureError } from '../../errors/capture'; // Gestion des erreurs propre

import { useCaptureStore } from '../../store/capture';

export default {
  name: "TopBar",
  emits: ['toggle-config'],

  computed: {
    buttonText(): string {
      return this.captureStore.showMatrice ? 'Graphique' : 'Matrice';
    },

    captureStore() {
      return useCaptureStore();
    },
    isRunning(): boolean {
      return this.captureStore.isRunning;
    }
  },
  data() {
    return {
      showMatrice: true // Toggle state (true for Matrice, false for NetworkGraphComponent)
    };
  },
  methods: {
    async reset() {
      this.tramesRecues = 0
      invoke('reset')    
    },
    toggleComponent() {
      this.captureStore.toggleView();
      this.$bus.emit('toggle'); // Si tu utilises toujours le bus
    },

    handleConfigClick() {
      info("[TopBar] Bouton config cliqué");
      this.$emit('toggle-config');
    },
    async start() {
      await invoke('start_capture')
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          
          info('Capture démarrée : ' + this.captureStore.isRunning);
        })
        .catch(async (err) => {
          await displayCaptureError(err);
        });
    },
    async stop() {
      await invoke('stop_capture')
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          console.log(typedStatus)
          info('Capture arretée : ' + this.captureStore.isRunning);
          
        })
        .catch(async (err) => {
          await displayCaptureError(err);
        });
    },
    async triggerSave() {
      info('Sauvegarde demandée');
    },
    toggleView() {
      info('Vue basculée');
    },
    async quit() {
      info('Fermeture demandée');
      await exit(0);
    },
    toggleConfig() {
      info('Ouverture panneau config'); 
    }
  }
}
</script>

<style scoped>
.top-bar {
  position: fixed;
  top: 0;
  left: 0;
  height: 40px;
  width: 100%;
  background-color: #070416;
  display: flex;
  align-items: center;
  padding: 0 10px;
  gap: 8px;
  border-bottom: 1px solid #252526;
  z-index: 9999;
}

.image-btn {
  background: transparent;
  border: none;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 18px;
  transition: background-color 0.2s;
}

.image-btn:hover {
  background-color: #3f4758;
}
.image-btn:disabled {
  opacity: 0.4; /* rend plus clair */
  cursor: not-allowed; /* curseur interdit */
  background-color: transparent; /* garde transparent au survol même désactivé */
}
.icon-img {
  height: 30;
  width: 30px;
  vertical-align: middle;
}
</style>