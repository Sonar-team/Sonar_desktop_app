<!--
  Barre d'actions principale : démarrer/arrêter la capture, reset, exports
  (matrice, labels, logs), ouverture des panneaux (import, filtre, config)
  et raccourcis clavier associés.
-->
<template>
  <div class="top-bar">
    <button class="image-btn" @click="start" title="Démarrer (ctrl+p)" :disabled="isRunning || activePanel !== null">
      <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="stop" title="Arrêter (ctrl+shift+p)" :disabled="!isRunning">🛑</button>
    <button class="image-btn" @click="reset" :disabled="activePanel !== null" title="Réinitialiser (ctrl+shift+r)">🔄</button>
    <button class="image-btn"  title="Config (ctrl+,)" :disabled="isRunning" @click="handleConfigClick">
      <img src="/src/assets/mdi--gear.svg" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="triggerSave" title="Sauvegarder (ctrl+s)">💾</button>
    <button class="image-btn" @click="SaveLabels" title="Exporter les labels">🏷️</button>

    <button class="image-btn" @click="displayPcapOpener" :disabled="isRunning || captureStore.hasData" title="Ouvrir un fichier Pcap (ctrl+o)">📄</button>
    <button class="image-btn" @click="displayCsvOpener" :disabled="isRunning" title="Ouvrir un fichier csv"><img src="/src/assets/images/import_csv.png" alt="Ouvrir un fichier csv" /></button>
    
    <button class="image-btn" @click="quit" title="Quitter (ctrl+q)">​❎</button>
    <button class="image-btn" @click="export_logs" title="Logs (ctrl+l)">📒</button>
    <button class="image-btn" @click="handleFilterClick" :disabled="isRunning" title="Filtrer (ctrl+f)">🔍</button>
  </div>
</template>

<script lang="ts">
import { Channel, invoke } from '@tauri-apps/api/core';
import { info, error } from '@tauri-apps/plugin-log';
import { save } from '@tauri-apps/plugin-dialog';
import { register, unregister } from '@tauri-apps/plugin-global-shortcut';
// when using `"withGlobalTauri": true`, you may use
// const { register } = window.__TAURI__.globalShortcut;



import { displayCaptureError } from '../../errors/capture'; // Gestion des erreurs propre
import { getCurrentDate } from '../../utils/time';
import { useCaptureStore } from '../../store/capture';
import { CaptureEvent } from '../../types/capture';
import { requestAppExit } from '../../utils/appExit';

type Panel = 'config' | 'pcap' | 'csv' | 'filter';

export default {
  name: "TopBar",
  emits: ['toggle-config', 'toggle-pcap','toggle-csv', 'toggle-filter', 'toggle-graph'],

  props: {
    configOpen: Boolean,
    filterOpen: Boolean,
    csvOpen: Boolean,
    pcapOpen: Boolean,
  },

  watch: {
  configOpen(val) { if (!val && this.activePanel === 'config') this.activePanel = null; },
  filterOpen(val) { if (!val && this.activePanel === 'filter') this.activePanel = null; },
  csvOpen(val)    { if (!val && this.activePanel === 'csv') this.activePanel = null; },
  pcapOpen(val)   { if (!val && this.activePanel === 'pcap') this.activePanel = null; },
},

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
      localHandler: null as ((e: KeyboardEvent) => void) | null,
      activePanel: null as Panel | null,
    };
  },
  async mounted() {
    // En mode fenêtré, des raccourcis globaux voleraient Ctrl+S, Ctrl+F, etc.
    // à toutes les applications du système : on n'utilise le plugin
    // global-shortcut qu'en headless, et un listener clavier local sinon.
    const headless = await invoke<boolean>('is_headless');

    if (headless) {
      const bindings: [string, () => void][] = [
        ['CommandOrControl+S', () => this.SaveAsCsv()],
        ['CommandOrControl+Shift+S', () => this.SaveAsXlsx()],
        ['CommandOrControl+Shift+R', () => this.reset()],
        ['CommandOrControl+P', () => this.start()],
        ['CommandOrControl+Shift+P', () => this.stop()],
        ['CommandOrControl+O', () => this.displayPcapOpener()],
        ['CommandOrControl+,', () => this.handleConfigClick()],
        ['CommandOrControl+F', () => this.handleFilterClick()],
        ['CommandOrControl+L', () => this.export_logs()],
        ['CommandOrControl+Q', () => this.quit()],
      ];
      for (const [shortcut, handler] of bindings) {
        this.shortcuts.push(shortcut);
        await register(shortcut, (event) => {
          if (event.state === 'Released') handler();
        });
      }
    } else {
      this.localHandler = (e: KeyboardEvent) => {
        const ctrl = e.ctrlKey || e.metaKey;
        if (!ctrl) return;

        const key = e.key.toLowerCase();
        if (key === 's' && !e.shiftKey) { e.preventDefault(); this.SaveAsCsv(); }
        else if (key === 's' && e.shiftKey) { e.preventDefault(); this.SaveAsXlsx(); }
        else if (key === 'r' && e.shiftKey) { e.preventDefault(); this.reset(); }
        else if (key === 'p' && !e.shiftKey) { e.preventDefault(); this.start(); }
        else if (key === 'p' && e.shiftKey) { e.preventDefault(); this.stop(); }
        else if (key === 'o') { e.preventDefault(); this.displayPcapOpener(); }
        else if (key === ',') { e.preventDefault(); this.handleConfigClick(); }
        else if (key === 'f') { e.preventDefault(); this.handleFilterClick(); }
        else if (key === 'l') { e.preventDefault(); this.export_logs(); }
        else if (key === 'q') { e.preventDefault(); this.quit(); }
      };
      window.addEventListener('keydown', this.localHandler);
    }

    useCaptureStore().refreshHasData(); // Vérifie s'il y a déjà des données au montage pour ajuster l'état de hasData
  },

  async beforeUnmount() {
    // recommandé en dev/hot reload
    if (this.shortcuts.length > 0) {
      await unregister(this.shortcuts);
      this.shortcuts = [];
    }
    if (this.localHandler) {
      window.removeEventListener('keydown', this.localHandler);
      this.localHandler = null;
    }
  },
  methods: {
    async export_logs() {
      info("export logs")

      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      if (this.activePanel !== null) {
        this.$emit(`toggle-${this.activePanel}`, false)
      }
      
      useCaptureStore().isImporting = true;

      try{
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
          info(`Sauvegarde terminée: ${JSON.stringify(saveResponse)}`);
          return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
        } else {
          info("Aucun chemin de fichier sélectionné");
          throw new Error("Sauvegarde annulée ou chemin non sélectionné");
        } 
      } finally {
        useCaptureStore().isImporting = false;
    }
  },

    async SaveAsCsv() {
      info("Save as csv");

       if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      if (this.activePanel !== null) {
        this.$emit(`toggle-${this.activePanel}`, false)  // Ferme le panneau ouvert avant de sauvegarder
      }

      useCaptureStore().isImporting = true;

      try {
        const response = await save({
          filters: [{ name: '.csv', extensions: ['csv'] }],
          title: 'Sauvegarder la matrice de flux',
          defaultPath: getCurrentDate() + '_DR_Matrice.csv'
        });

        if (response) {
          const saveResponse = await invoke('export_csv', { path: response });
          info(`response: ${JSON.stringify(saveResponse)}`);
        } else {
          info("Aucun chemin sélectionné");
        }
      } catch (err) {
        error(`Erreur sauvegarde csv: ${JSON.stringify(err)}`);
      } finally {
        useCaptureStore().isImporting = false;
      }
    },
    async SaveLabels() {
      info("Export des labels");

      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      if (this.activePanel !== null) {
        this.$emit(`toggle-${this.activePanel}`, false)  // Ferme le panneau ouvert avant de sauvegarder
      }

      useCaptureStore().isImporting = true;

      try {
        const response = await save({
          filters: [{ name: '.csv', extensions: ['csv'] }],
          title: 'Exporter les labels',
          defaultPath: getCurrentDate() + '_labels.csv'
        });

        if (response) {
          await invoke('export_label_file', { path: response });
          info("Labels exportés");
        } else {
          info("Aucun chemin sélectionné");
        }
      } catch (err) {
        error(`Erreur export labels: ${err}`);
        displayCaptureError(err);
      } finally {
        useCaptureStore().isImporting = false;
      }
    },
    async SaveAsXlsx() {

       if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }
      
      if (this.activePanel !== null) {
        this.$emit(`toggle-${this.activePanel}`, false)
      }

      useCaptureStore().isImporting = true;

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
          info(`Sauvegarde terminée: ${JSON.stringify(saveResponse)}`);
          return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
        } else {
          info("Aucun chemin de fichier sélectionné");
          throw new Error("Sauvegarde annulée ou chemin non sélectionné");
        }
      } catch (err) {
        // `catch (error)` masquait la fonction de log `error` → TypeError.
        error(`Erreur lors de la sauvegarde en xlsx: ${JSON.stringify(err)}`);
        throw err; // Relancer l'erreur pour la gestion dans quit()
      } finally {
        this.activePanel = null;
        useCaptureStore().isImporting = false;
      }
    },
    async triggerSave() {
      info("trigger save")
      this.SaveAsCsv();
      
    },
    async reset() {
      info("reset")

      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      await invoke('reset_capture');
      await useCaptureStore().refreshHasData();
      this.$bus.emit('reset');
    },


    handleConfigClick() {
      info("[TopBar] Bouton config cliqué");

      if (this.captureStore.isRunning) {
        return;
      }
      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }


      if (this.activePanel !== null && this.activePanel !== 'config') {
        this.$emit(`toggle-${this.activePanel}`, false)
      };
      this.activePanel = 'config';
      this.$emit('toggle-config');
    },
    displayPcapOpener() {
      info("[TopBar] Bouton open cliqué");

      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      if (useCaptureStore().hasData || this.captureStore.isRunning) return;
      if (this.activePanel !== null && this.activePanel !== 'pcap') {
        this.$emit(`toggle-${this.activePanel}`, false)
      };
      this.activePanel = 'pcap';
      this.$emit('toggle-pcap');
    },
    displayCsvOpener() {
      info("[TopBar] Bouton open cliqué");

      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      if (this.captureStore.isRunning) return; // Empêche d'ouvrir le panneau d'import CSV si la matrice de flux contient déjà des données ou si une capture est en cours
      if (this.activePanel !== null && this.activePanel !== 'csv') {
        this.$emit(`toggle-${this.activePanel}`, false) // Ferme le panneau ouvert avant d'ouvrir le panneau d'import CSV
      };
      this.activePanel = 'csv';
      this.$emit('toggle-csv');
    },
    handleFilterClick() {

      if (this.captureStore.isRunning) {
        return;
      }

      info("[TopBar] Bouton filter cliqué");

      if (useCaptureStore().isImporting) {
        info("Une opération d'importation ou de sauvegarde est déjà en cours. Veuillez patienter.");
        return;
      }

      if (this.activePanel !== null && this.activePanel !== 'filter') {
        this.$emit(`toggle-${this.activePanel}`, false)
      };
      this.activePanel = 'filter';
      this.$emit('toggle-filter');
    },
    async start() {
      if (this.activePanel !== null || useCaptureStore().isImporting) return;

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
      if (!this.captureStore.isRunning || useCaptureStore().isImporting) {
        return;
      }
      const onEvent = this.captureStore.getChannel();
      await invoke('stop_capture',{ onEvent })
        .then((status) => {
          const typedStatus = status as { is_running: boolean };
          this.captureStore.updateStatus(typedStatus);
          info('Capture arrêtée : ' + this.captureStore.isRunning);
        })
        .catch(displayCaptureError)
        .finally(() =>useCaptureStore().refreshHasData());
    },
    toggleView() {
      info('Vue basculée');
    },

    async quit() {
      info('Fermeture demandée');
      await requestAppExit();
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
