<template>
  <div class="container">
    <div class="center-container">
      <ConflictDialog v-if="showConflictDialog" 
      :same_ip_diff_mac="sameIpDiffMac" 
      :same_ip_diff_label="sameIpDiffLabel" 
      :invalid_mac="invalidMac"
      :invalid_ip="invalidIp"
      :invalid_lines="invalidLines"
      @showConflictDialog="showConflictDialog = false"/>

        <!-- Overlay de chargement -->
      <div class="overlay" v-if="isConverting">
        <div class="spinner"></div>
        <p class="overlay-text">Conversion en cours…</p>
      </div>
      <button class="btn image-btn cross" @click.prevent="windowClosed" :disabled="isConverting">❌</button>
      
      <div v-if="mode === 'csv'" class="csv-group">
          <button class="btn btn-add text" @click="addCsvFiles" :disabled="isConverting">
            Ajouter un fichier
          </button>
          <p v-show="labelRows.length == 0" class="text">Aucun label importé pour le moment</p>
          <div v-show="labelRows.length > 0" class="table-header-row">
            <h2 class="text table-title">Contenu importé</h2>
            <div class="search-group">
              <input class="input-search" v-model="searchInput" @input="listFilter"/>
              <button class="btn image-btn icon-lg" @click.prevent="clearLabelStore()" title="Réinitialiser le contenu">🔄</button>
            </div>
          </div>
          <div v-show="labelRows.length > 0" class="data-table">
            <div class="data-table-header">
              <div class="col text">Adresse MAC</div>
              <div class="col text">Adresse IP</div>
              <div class="col text">Label</div>
            </div>
            <div class="separator"></div>
            <div class="data-table-body">
              <div v-for="([mac, ip, label], index) in filteredlabelRows" :key="index" class="data-table-row">
                <div class="col text">{{ mac || "-" }}</div>
                <div class="col text">{{ ip || "-" }}</div>
                <div class="col text">{{ label || "-" }}</div>
              </div>
            </div>
          </div>
          <p class="text hint">Format : <code>mac, ip, label</code> — tous les champs peuvent être vides</p>
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
import { displayCaptureError, CaptureStateErrorKind, LabelErrorKind } from '../../../errors/capture';
import ConflictDialog from './ConflictDialog.vue'


export default defineComponent({
  name: 'ImportPanel',
  emits: ['update:visible','toggle-pcap', 'toggle-warning', 'showConflictDialog'],
  components: {
    ConflictDialog
  },
  props: {
    mode: {
      type: String,
      default: 'pcap'
    }
  },
  data() {
    return {
      packetFiles: [] as string[],
      labelRows: [] as [string, string, string][],
      filteredlabelRows: [] as [string, string, string][],
      searchInput: "",
      isConverting: false,
      showConflictDialog: false,
      sameIpDiffMac: [] as [string, string, string][], 
      sameIpDiffLabel: [] as [string, string, string][],
      invalidMac: [] as string[],
      invalidIp: [] as string[],
      invalidLines: [] as string[]
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

    async addFiles(type: 'pcap' | 'csv', extensions: string[]) {
        const label = type === 'csv' ? 'Label File' : 'Capture File';
        const isPcap = type === 'pcap' ? true : false;
        useCaptureStore().isImporting = true;

      try {
        const files = await open({
          multiple: isPcap,
          filters: [{ name: label, extensions: extensions }],
        });

        if (!files) return;

        
        if (type === 'csv') {
          await this.importLabelFile(files);
        } else {
          const list = Array.isArray(files) ? files : [files];
          this.packetFiles.push(...list);
        }
      } finally {
        useCaptureStore().isImporting = false;
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
        await useCaptureStore().refreshHasData();
        this.isConverting = false;
      }

      this.packetFiles = [];
    },

    listFilter() {
      this.filteredlabelRows = this.labelRows.filter((row) => row.some((field) => field.toLowerCase().includes(this.searchInput.toLowerCase())))
    },

    async importLabelFile(path: string) {
      if (path.length === 0) return;

      info('import_label_file: ' + path);

      this.isConverting = true;
      this.invalidIp = [];
      this.invalidMac = [];
      this.sameIpDiffLabel = [];
      this.sameIpDiffMac = [];

      try {
        await invoke('import_label_file', { incomingFilePath: path });
        info('réponse invoke');
      } catch (err) {
        const error = err as CaptureStateErrorKind;
        if (error.kind === "label") {
          const labelError = error.message as LabelErrorKind;
          if (labelError.kind === "invalidMacIpFormat") {
            const [invalidMac, invalidIp] = labelError.message;
            this.invalidMac = invalidMac;
            this.invalidIp = invalidIp;
            this.showConflictDialog = true;
          } else if (labelError.kind === "labelLinesConflicts") {
            const [sameIpDiffMac, sameIpDiffLabel] = labelError.message;
            this.sameIpDiffMac = sameIpDiffMac;
            this.sameIpDiffLabel = sameIpDiffLabel;
            this.showConflictDialog = true;
          } else if (labelError.kind === "invalidRowsFormat") {
            this.invalidLines = labelError.message;
            this.showConflictDialog = true;
          } else {
            displayCaptureError(err);
          }
        } else {
          displayCaptureError(err);
        }  
      } finally {
        this.labelRows = await invoke('get_label_rows');
        this.filteredlabelRows =this.labelRows;
        this.isConverting = false;    
      }
    },
  

    async clearLabelStore() {
        try {
          await invoke('clear_label_store');
          info('réponse invoke');
          this.labelRows = await invoke('get_label_rows');
          this.filteredlabelRows = this.labelRows;
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

    this.labelRows = await invoke('get_label_rows');
    this.filteredlabelRows = this.labelRows;

  },

})
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

.btn-open:enabled:hover{
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

.cross {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
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

.table-header-row {
  display: flex;
  align-items: center;
  width: 90%;
  margin-top: 20px;
  margin-bottom: 10px;
}

.table-title {
  margin: 0 auto 0 0;
}

.search-group {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.input-search {
  width: 140px;
  border-radius: 8px;
  border: 1px solid whitesmoke;
  padding: 0.2em 0.8em;
  font-size: 1em;
  font-family: inherit;
  color: whitesmoke;
  background-color: #2d3748;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  transition: border-color 0.25s, background-color 0.25s;
}
.input-search::placeholder {
  color: rgba(245, 245, 245, 0.5);
}
.input-search:hover {
  background-color: #313152;
}
.input-search:focus {
  outline: none;
  border-color: #2596be;
  background-color: #313152;
}

.icon-lg {
  font-size: 1.6rem;
  line-height: 1;
}

.text {
  color: whitesmoke
}

.file-list {
  width: 90%;
  max-height: 250px;
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

.file-list label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.hint {
  font-size: 0.8em;
  color: rgba(245, 245, 245, 0.5);
  margin-top: 0.4rem;
  margin-bottom: 0.8rem;
}

.hint code {
  font-family: monospace;
  color: rgba(245, 245, 245, 0.75);
}

.separator {
  height: 1px;
  width: 95%;
  background-color: whitesmoke;
  margin: 0 auto 16px auto;
}

.data-table {
  width: 90%;
  background-color: #2d3748;
  border-radius: 4px;
  margin-bottom: 1.5rem;
  overflow: hidden; 
  text-align: center;
}

.data-table-header {
  display: flex;
  padding: 0.5rem;
  margin-top: 5px;
}

.data-table-header .col {
  font-weight: 600;
}

.data-table-body {
  max-height: 250px;
  overflow-y: auto;
  padding: 0.5rem;
}

.data-table-row {
  display: flex;
  padding-top: 4px;
  padding-bottom: 4px;
}

.col {
  width: 33.33%;
  padding-left: 8px;
  box-sizing: border-box;
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
