<template>
  <div class="top-bar">
    <button class="image-btn" @click="start" title="Démarrer" :disabled="isRunning">
      <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="stop" title="Arrêter" :disabled="!isRunning">
      🛑
    </button>

    <button class="image-btn"  title="Config" :disabled="isRunning" @click="handleConfigClick">
      <img src="/src/assets/mdi--gear.svg" alt="Flux" class="icon-img" />
    </button>

    <button class="image-btn" @click="reset" title="Réinitialiser">🔄</button>
    <button class="image-btn" @click="triggerSave" title="Sauvegarder">💾</button>
    <button class="image-btn" @click="displayPcapOpener" title="Ouvrir">📄</button>
    <button class="image-btn" @click="quit" title="Quitter">❌</button>
    <button class="image-btn" @click="export_logs" title="Logs">📒</button>
    <button class="image-btn" @click="handleFilterClick" title="Filtrer">🔍</button>
  </div>
</template>

<script lang="ts">
import { Channel, invoke } from '@tauri-apps/api/core';
import { exit } from '@tauri-apps/plugin-process';
import { info, error } from '@tauri-apps/plugin-log';
import { save } from '@tauri-apps/plugin-dialog';

import { displayCaptureError } from '../../errors/capture'; // Gestion des erreurs propre

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
    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${now.getFullYear()}${this.padZero(now.getMonth() + 1)}${this.padZero(now.getDate())}`;
      
      return formattedDate;
    },
    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
    },
    async SaveAsCsv() {
      info("Save as csv")
      save({
        filters: [{
          name: '.csv',
          extensions: ['csv']
        }],
        title: 'Sauvegarder la matrice de flux',
        defaultPath: this.getCurrentDate()+ 'Matrice' + '.csv' // Set the default file name here
      
      }).then((response) => 
        invoke('export_csv', { path: response })
          .then((response) => 
            error("save error: ",response))
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
          defaultPath: this.getCurrentDate() + 'Matrice' + '.xlsx'
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