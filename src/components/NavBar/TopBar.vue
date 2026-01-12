<template>
  <div class="top-bar">
    <button class="image-btn" @click="start" title="Démarrer" :disabled="isRunning">
      <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="stop" title="Arrêter" :disabled="!isRunning">
      🛑
    </button>
    <button class="image-btn" @click="reset" title="Réinitialiser">🔄</button>
    <button class="image-btn"  title="Config" :disabled="isRunning" @click="handleConfigClick">
      <img src="/src/assets/mdi--gear.svg" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="triggerSave" title="Sauvegarder ctrl+s">💾</button>
    <button class="image-btn" @click="displayPcapOpener" title="Ouvrir">📄</button>
    <button class="image-btn" @click="quit" title="Quitter">❌</button>
    <button class="image-btn" @click="export_logs" title="Logs">📒</button>
    <button class="image-btn" @click="handleFilterClick" title="Filtrer">
      <img src="/src/assets/filter-solid-full.svg" alt="Flux" class="icon-img" />
    </button>
  </div>
</template>

<script lang="ts">
import { Channel, invoke } from '@tauri-apps/api/core';
import { exit } from '@tauri-apps/plugin-process';
import { info, error } from '@tauri-apps/plugin-log';
import { save } from '@tauri-apps/plugin-dialog';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
// when using `"withGlobalTauri": true`, you may use
// const { register } = window.__TAURI__.globalShortcut;



import { displayCaptureError } from '../../errors/capture'; // Gestion des erreurs propre
import { getCurrentDate } from '../../utils/time';
import { useCaptureStore } from '../../store/capture';
import { CaptureEvent } from '../../types/capture';

export default {
  name: "TopBar",
  emits: ['toggle-config','toggle-pcap','toggle-filter'],

  computed: {
    buttonText(): string {
      return this.captureStore.showMatrice ? 'Graphique' : 'Matrice';
    },
    captureStore() {
      return useCaptureStore();
    },

    isRunning(): boolean {
      return this.captureStore.isRunning;
    },
  },
  data() {
    return {
      showMatrice: true // Toggle state (true for Matrice, false for NetworkGraphComponent)
    };
  },
  mounted() {
    register('CommandOrControl+S', (event) => {
    if (event.state === 'Released') {
          this.SaveAsCsv();
        }
    });
    register('CommandOrControl+R', (event) => {
    if (event.state === 'Released') {
          this.reset();
        }
    });
    
  },
  beforeUnmount() {
  // recommandé en dev/hot reload
    unregister('CommandOrControl+S');
    unregister('CommandOrControl+R');
  },
  methods: {
    async export_logs() {
      info("export logs")
      const response = await save({
        filters: [{
          name: '.log',
          extensions: ['log']
        }],
        title: 'Sauvegarder les logs',
        defaultPath: 'sonar.log'
      });

      if (response) {
        // Attendez que l'invocation d'API pour sauvegarder soit terminée
        const saveResponse = await invoke('export_logs', { destination: response });
        info("Sauvegarde terminée:", saveResponse);
        return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
      } else {
        info("Aucun chemin de fichier sélectionné");
        throw new Error("Sauvegarde annulée ou chemin non sélectionné");
      }
    },

    async SaveAsCsv() {
      info("Save as csv")
      save({
        filters: [{
          name: '.csv',
          extensions: ['csv']
        }],
        title: 'Sauvegarder la matrice de flux',
        defaultPath: getCurrentDate()+ '_DR_Matrice.csv' // Set the default file name here
      
      }).then((response) => 
        invoke('export_csv', { path: response })
          .then((response: any) => 
            info("response: ", response))
          .catch((error: any) => 
            error("error: ", error))
      )
    },
    async SaveAsXlsx() {
      try {
        info("Début de la sauvegarde en xlsx");
        const response = await save({
          filters: [{
            name: '.xlsx',
            extensions: ['xlsx']
          }],
          title: 'Sauvegarder la matrice de flux',
          defaultPath: getCurrentDate() + '_DR_Matrice' + '.xlsx'
        });

        if (response) {
          // Attendez que l'invocation d'API pour sauvegarder soit terminée
          const saveResponse = await invoke('save_packets_to_excel', { file_path: response });
          info("Sauvegarde terminée:", saveResponse);
          return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
        } else {
          info("Aucun chemin de fichier sélectionné");
          throw new Error("Sauvegarde annulée ou chemin non sélectionné");
        }
      } catch (error) {
        error("Erreur lors de la sauvegarde en xlsx:", error);
        throw error; // Relancer l'erreur pour la gestion dans quit()
      }
    },
    async triggerSave() {
      info("trigger save")
      this.SaveAsCsv();
      
    },
    async reset() {
      info("reset")
      await invoke('reset_capture');
      this.$bus.emit('reset');
    },


    handleConfigClick() {
      info("[TopBar] Bouton config cliqué");
      this.$emit('toggle-config');
    },
    displayPcapOpener() {
      info("[TopBar] Bouton open cliqué");
      this.$emit('toggle-pcap');
    },
    handleFilterClick() {
      info("[TopBar] Bouton filter cliqué");
      this.$emit('toggle-filter');
    },
    async start() {
      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent); // 🟢 rendre le Channel accessible

      await invoke('start_capture', { onEvent })
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          info('Capture démarrée : ' + this.captureStore.isRunning);
        })
        .catch(displayCaptureError);
    },
    async stop() {
      const onEvent = this.captureStore.getChannel();
      await invoke('stop_capture',{ onEvent })
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          info('Capture arrêtée : ' + this.captureStore.isRunning);
        })
        .catch(displayCaptureError);
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
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  transform: translateZ(0);
  backface-visibility: hidden;
  -webkit-font-smoothing: subpixel-antialiased;
}

.image-btn:hover {
  background-color: #3f4758;
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.image-btn:active {
  transform: translateY(1px) scale(0.99) translateZ(0);
  transition: transform 0.1s ease, background-color 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}
.image-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  background-color: transparent;
  transform: none !important;
  box-shadow: none !important;
}
.icon-img {
  height: 30;
  width: 30px;
  vertical-align: middle;
}
</style>