<template>
  <div class="top-bar">
    <button class="image-btn" @click="start" title="Démarrer (ctrl+p)" :disabled="isRunning">
      <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="stop" title="Arrêter (ctrl+shift+p)" :disabled="!isRunning">
      🛑
    </button>
    <button class="image-btn" @click="reset" title="Réinitialiser (ctrl+shift+r)">🔄</button>
    <button class="image-btn"  title="Config (ctrl+,)" :disabled="isRunning" @click="handleConfigClick">
      <img src="/src/assets/mdi--gear.svg" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="triggerSave" title="Sauvegarder (ctrl+s)">💾</button>

    <button class="image-btn" @click="displayPcapOpener" title="Ouvrir un fichier Pcap (ctrl+o)">📄</button>
    <button class="image-btn" @click="displayCsvOpener" title="Ouvrir un fichier csv"><img src="./Pictures/import_csv.png" alt="Ouvrir un fichier csv" /></button>
    
    <button class="image-btn" @click="quit" title="Quitter (ctrl+q)">❌</button>
    <button class="image-btn" @click="export_logs" title="Logs (ctrl+)">📒</button>
    <button class="image-btn" @click="handleFilterClick" title="Filtrer (ctrl+,)">🔍</button>
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
  emits: ['toggle-config','toggle-pcap','toggle-csv','toggle-filter'],

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
      showMatrice: true, // Toggle state (true for Matrice, false for NetworkGraphComponent)
      shortcuts: [] as string[],
      showMatrice: true, // Toggle state (true for Matrice, false for NetworkGraphComponent)
    };
  },
  mounted() {
    // Sauvegardes
    this.bindShortcut('CommandOrControl+S', () => this.SaveAsCsv());
    this.bindShortcut('CommandOrControl+Shift+S', () => this.SaveAsXlsx());

    // Reset
    this.bindShortcut('CommandOrControl+Shift+R', () => this.reset());

    // Start / Stop
    // Choix 1 (classique "Play/Stop")
    this.bindShortcut('CommandOrControl+P', () => this.start());
    this.bindShortcut('CommandOrControl+Shift+P', () => this.stop());

    // Ouvrir (pcap opener)
    this.bindShortcut('CommandOrControl+O', () => this.displayPcapOpener());

    // Config
    this.bindShortcut('CommandOrControl+,', () => this.handleConfigClick());

    // Filtre
    this.bindShortcut('CommandOrControl+F', () => this.handleFilterClick());

    // Logs
    this.bindShortcut('CommandOrControl+L', () => this.export_logs());

    // Quit
    this.bindShortcut('CommandOrControl+Q', () => this.quit());
  },

  async beforeUnmount() {
  // recommandé en dev/hot reload
    await this.unbindAllShortcuts();
  },
  methods: {
    bindShortcut(shortcut: string, handler: () => void) {
      this.shortcuts.push(shortcut);
      register(shortcut, (event) => {
        if (event.state === 'Released') handler();
      });
    },

    async unbindAllShortcuts() {
      // unregister accepte string | string[]
      if (this.shortcuts.length > 0) {
        await unregister(this.shortcuts);
      }
      this.shortcuts = [];
    },
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
    displayCsvOpener() {
      info("[TopBar] Bouton open cliqué");
      this.$emit('toggle-csv');
    },
    handleFilterClick() {
      info("[TopBar] Bouton filter cliqué");
      this.$emit('toggle-filter');
    },
    async start() {
      if (this.captureStore.isRunning) {
        return;
      }
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
      if (!this.captureStore.isRunning) {
        return;
      }
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