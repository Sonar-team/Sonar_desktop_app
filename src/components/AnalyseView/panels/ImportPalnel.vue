<template>
  <div class="container">
    <div class="center-container">
      <ConflictPanel v-if="showConflictPanel" :files="conflictualFiles" @showConflictPanel="showConflictPanel = false"/>

        <!-- Overlay de chargement -->
      <div class="overlay" v-if="isConverting">
        <div class="spinner"></div>
        <p class="overlay-text">Conversion en cours…</p>
      </div>
      <button class="btn image-btn" @click.prevent="windowClosed">❌</button>
      
      <div v-if="mode === 'csv'" class="csv-group">
          <button class="btn btn-add text" @click="addCsvFiles" :disabled="isConverting">
            Ajouter des fichiers
          </button>
          <ul class="file-list">
            <li v-for="([file,], index) in labelFiles" :key="index">
              <label :for="String(index)">
                <input type="checkbox" v-model="selectedLabelFiles" :value="file" :id="String(index)" class="toggle" @change="addSelectedLabelFilesList">
                <span class="text">{{ file }}</span>
                <button class="image-btn" @click.prevent="RemoveLabelFile(file)" title="Supprimer"><img src="./Pictures/Poubelle.jpg" alt="Supprimer" /></button>
              </label>
            </li>
          </ul>
      </div>

      <div v-else-if="mode === 'pcap'">
        <div class="file-group">
          <button class="btn btn-add text" @click="addPcapFiles" :disabled="isConverting">
            Ajouter des fichiers
          </button>
          <button class="btn btn-clear" @click="clearFiles" :disabled="isConverting">
            Effacer
          </button>
        </div>
        <ul class="file-list" v-if="packetFiles.length > 0">
          <li v-for="(file, index) in packetFiles" :key="index">
            {{ file }}
          </li>
        </ul>
        <button @click="convertPcap" class="btn btn-open" :disabled="isConverting || packetFiles.length === 0">
          Ouvrir
        </button>
      </div>
    </div> 
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke, Channel } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';
import { useCaptureStore } from '../../../store/capture';
import { CaptureEvent } from '../../../types/capture';
import { displayCaptureError } from '../../../errors/capture';
import ConflictPanel from './ConflictPanel.vue'


export default defineComponent({
  name: 'ImportPanel',
  components: {
    ConflictPanel  
  },
  emits: ['update:visible','toggle-pcap', 'toggle-warning'],
  props: {
    mode: {
      type: String,
      default: 'pcap'
    }
  },
  data() {
    return {
      packetFiles: [] as string[],
      labelFiles: [] as [string, boolean][],
      selectedLabelFiles: [] as string[],
      conflictualFiles: [] as [string, string][],
      isConverting: false,
      showConflictPanel: false
    };
  },

  computed: {
    captureStore() {
      return useCaptureStore();
    },
    isRunning(): boolean {
      return this.captureStore.isRunning;
    },
  },

  methods: {
    windowClosed() {
      this.$emit('update:visible', false);
    },

    addPcapFiles() {
      return this.addFiles('pcap', ['pcap', 'pcapng', 'cap']);
    },

    addCsvFiles() {
      return this.addFiles('csv', ['csv']);
    },

    async addFiles(type : 'pcap' | 'csv', extensions : string[]) {

      const label = type === 'csv' ? 'Label File' : 'Capture File';
      
      const files = await open({
        multiple: true,
        filters: [{ name: label, extensions: extensions }]
      });

      if (!files) return;

      const list = Array.isArray(files) ? files : [files];

      if (type === 'csv') {
        let labelFilesNames = list.map(((path): [string, boolean] => [path.split(/[\\/]/).pop() ?? path, true]));
        info('' + labelFilesNames);
        await this.convertLabelFile(list, labelFilesNames)
      } else {
        this.packetFiles.push(...list);
      }
    },

    clearFiles() {
      this.packetFiles = [];
    },

    async convertPcap() {
      if (this.packetFiles.length === 0) return;

      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent);

      info('convert_from_pcap_list : ' + this.packetFiles);

      this.isConverting = true;

      try {
        await invoke('convert_from_pcap_list', { pcapPaths: this.packetFiles, onEvent });
        info('réponse invoke');
        this.$emit('update:visible', false);
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.isConverting = false;
      }

      this.packetFiles = [];
    },

    async convertLabelFile(paths: string[], names: [string, boolean][]) {
      if (paths.length === 0) return;

      info('import_label_files: ' + paths);

      this.isConverting = true;

      try {
        this.conflictualFiles = await invoke<[string, string][]>('import_label_files', { csvPaths: paths });
        info('réponse invoke');
        if (this.conflictualFiles.length > 0 ){
          info('Il y a des fichiers en conflits')
          this.showConflictPanel = true
        }
        this.labelFiles.push(...names.filter(([name]) => !this.labelFiles.some(([existing]) => existing === name)));
        this.labelFiles.sort();
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.isConverting = false;
      }
    },
  

    async RemoveLabelFile(fileRemoved: string) {
        info('fileRemoved : ' + fileRemoved);
        try {
          await invoke('remove_label_file', { csvFile: fileRemoved});
          info('réponse invoke');
          this.labelFiles = this.labelFiles.filter(([name]) => name !== fileRemoved);
          this.selectedLabelFiles = this.selectedLabelFiles.filter((name) => name !== fileRemoved);
        } catch (err) {
          displayCaptureError(err);
        }
      },

    async addSelectedLabelFilesList(){
        try {
          await invoke('add_selected_label_files_list', { selectedFilesNamesList: this.selectedLabelFiles});
          info('réponse invoke');
        } catch (err) {
          displayCaptureError(err);
        }
      },
  }, 

  async mounted() {
    this.captureStore.onStarted(() => {
      info("started hearded");
      this.captureStore.updateStatus({ is_running: true });
    });

    this.captureStore.onFinished(() => {
      info("finished hearded");
      this.captureStore.updateStatus({ is_running: false });
    });

    this.labelFiles = await invoke('read_label_files_list');
    this.selectedLabelFiles = this.labelFiles
            .filter(([_, checked]) => checked)
            .map(([file, _]) => file);
  },
});
</script>

<style scoped>
.container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: rgba(0, 0, 0, 0.7);
  z-index: 1000;
}

.center-container {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: stretch;
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 2rem;
  width: 90%;
  max-width: 600px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.csv-group {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.file-group {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
  justify-content: center;
}

.btn {
  border-radius: 8px;
  border: 1px solid;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: whitesmoke;
  background-color: #181829;
  transition: border-color 0.25s, background-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-clear {
  background-color: #181829;
  border-color: #d8392b;
  color: white;
}

.btn-clear:hover{
  background-color:#313152 ;
}

.btn-clear:active {
  background-color: #d8392b;
}

.btn-open {
  background-color: #181829;
  border-color: #48bb78;
  display: block;
  margin: 0 auto;
}

.btn-open:hover {
    background-color: #313152;
}

.btn-open:active {
  background-color: #48bb78;
}

.btn-add {
  border-color: whitesmoke;
}

.btn-add:hover {
  border-color: #2596be;
  background-color:#313152 ;
}
.btn-add:active {
  border-color: #2596be;
  background-color: #2596be;
}

.image-btn {
  background: none;
  border:none;
  padding: 0;
  cursor: pointer;
  margin-left: auto;
}

.image-btn:hover {
  transform: translateY(-1px) translateZ(0);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.image-btn:active {
  transform: translateY(1px) scale(0.99) translateZ(0);
  transition: transform 0.1s ease, background-color 0.2s;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.toggle {
    /* On définit la hauteur de notre élément */
    --toggle-height: 1.5rem;

    /* On désactive le style par défaut du système d'exploitation */
    appearance: none;

    /* On définit les dimensions de la "piste" (le fond) */
    width: 3rem;
    height: var(--toggle-height);
    border-radius: 99px; /* Un grand border-radius pour l'effet pilule */
    background: #334155; /* Couleur quand c'est inactif (gris bleuté) */
    position: relative;
    cursor: pointer;
    transition: 0.3s;

    /* On crée la "pastille" (le bouton qui glisse) avec un pseudo-élément
    * Le pseudo-élément est intéressant car le rond qui indique l'état n'a pas de sens sémantique,
    * ajouter un élément juste graphique est de la responsabilité de CSS.
    */
    &::after {
        /* On définit la hauteur et le placement de notre pastille */
        --element-top-left: 2px;
        --element-size: calc(var(--toggle-height) - var(--element-top-left) * 2);

        content: '';
        position: absolute;
        left: var(--element-top-left);
        top: var(--element-top-left);
        width: var(--element-size);
        height: var(--element-size);
        background: white;
        border-radius: 50%;
        transition: 0.3s;
    }

    /* On gère l'état "Activé" grâce à la pseudo-classe :checked natif aux checkbox */
    &:checked {
        background: #2596be; /* Couleur quand c'est actif (vert néon) */


        /* On déplace la pastille vers la droite */
        &::after {
            transform: translateX(var(--toggle-height));
        }
    }
}

.file-list label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.text {
  color: whitesmoke
}

.file-list {
  width: 90%;
  max-height: 200px;
  overflow-y: auto;
  background-color: #2d3748;
  border-radius: 4px;
  padding: 0.5rem;
  margin-bottom: 1.5rem;
}

.file-list li {
  padding: 0.5rem;
  margin: 0.25rem 0;
  border-radius: 4px;
  word-break: break-all;
  font-family: monospace;
  font-size: 0.9rem;
}

/* Overlay + Spinner */

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  border-radius: 8px;
  z-index: 2000;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.1);
  border-left-color: #4299e1;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 1rem;
}

.overlay-text {
  color: white;
  font-size: 1.1rem;
  font-weight: 500;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
