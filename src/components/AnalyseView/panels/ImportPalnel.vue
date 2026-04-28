<script setup lang="ts">
  const checkedNames = ref(['Jack'])
</script>


<template>
  <div class="container">
    <div class="center-container">

      <div class="left-panel">
        <input type="checkbox" id="jack" value="Jack" v-model="checkedNames">
        <label for="jack">Jack</label>
        <input type="checkbox" id="john" value='John' v-model="checkedNames">
        <label for="john">John</label>
        <input type="checkbox" id="mike" value="Mike" v-model="checkedNames">
        <label for="mike">Mike</label>
        <p>Checked names: {{ checkedNames }}</p>

      </div>
      <div class="separateur"></div>
      <div class="right-panel">
        <!-- Overlay de chargement -->
        <div class="overlay" v-if="isConverting">
          <div class="spinner"></div>
          <p class="overlay-text">Conversion en cours…</p>
        </div>

        <div class="file-group">
          <label for="packetFiles"></label>
          <div v-if="mode === 'csv'" class="file-group">
            <button class="btn" @click="addCsvFiles" :disabled="isConverting">
            Ajouter des fichiers de label
            </button>
            <button class="btn btn-clear" @click="clearFiles" :disabled="isConverting">
              Effacer
            </button>
          </div>
          <div v-else-if="mode === 'pcap'" class="file-group">
            <button class="btn" @click="addPcapFiles" :disabled="isConverting">
              Ajouter des fichiers
            </button>
            <button class="btn btn-clear" @click="clearFiles" :disabled="isConverting">
              Effacer
            </button>
          </div>
        </div>
          
        <ul class="file-list" v-if="packetFiles.length > 0">
          <li v-for="(file, index) in packetFiles" :key="index">
            {{ file }}
          </li>
        </ul>

        <button v-show="mode === 'pcap'"
          @click="convertPcap"
          class="btn btn-open"
          :disabled="isConverting || packetFiles.length === 0"
        >
          Ouvrir
        </button>
        <button v-show="mode === 'csv'"
          @click="convertCsv"
          class="btn btn-open"
          :disabled="isConverting || packetFiles.length === 0"
        >
          Ouvrir
        </button>
      </div>

    </div>
  </div>
</template>

<script lang="ts">
import { ref } from 'vue'
import { defineComponent } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke, Channel } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';
import { useCaptureStore } from '../../../store/capture';
import { CaptureEvent } from '../../../types/capture';
import { displayCaptureError } from '../../../errors/capture';


export default defineComponent({
  name: 'ImportPanel',
  emits: ['update:visible','toggle-pcap'],
  props: {
    mode: {
      type: String,
      default: 'pcap'
    }
  },
  data() {
    return {
      packetFiles: [] as string[],
      isConverting: false,
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
    addPcapFiles() {
      return this.addFiles('Capture File', ['pcap', 'pcapng', 'cap']);
    },

    addCsvFiles() {
      return this.addFiles('Label File', ['csv']);
    },

    async addFiles(name : string, extensions : string[]) {
      const files = await open({
        multiple: true,
        filters: [{ name: name, extensions: extensions }]
      });

      if (files) {
        const list = Array.isArray(files) ? files : [files];
        this.packetFiles.push(...list);
      }
    },

    clearFiles() {
      this.packetFiles = [];
    },

    exit(){
      this.$emit('update:visible', false)
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
    },

    async convertCsv() {
      if (this.packetFiles.length === 0) return;

      info('import_label_files: ' + this.packetFiles);

      this.isConverting = true;

      try {
        await invoke('import_label_files', { csvPaths: this.packetFiles });
        info('réponse invoke');
        this.$emit('update:visible', false);
      } catch (err) {
        displayCaptureError(err);
      } finally {
        this.isConverting = false;
      }
    },
  },

  mounted() {
    this.captureStore.onStarted(() => {
      info("started hearded");
      this.captureStore.updateStatus({ is_running: true });
    });

    this.captureStore.onFinished(() => {
      info("finished hearded");
      this.captureStore.updateStatus({ is_running: false });
    });
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
  align-items: center;
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 2rem;
  width: 90%;
  max-width: 600px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.left-panel {
  width: 30%;
  padding: 1rem;
}

.separateur {
  width: 2px;
  background-color: #ccc;
  cursor: col-resize;
}

.right-panel {
  flex: 1;
  padding: 1rem;
}

.file-group {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-clear {
  background-color: #f56565;
  color: white;
}

.btn-open {
  background-color: #48bb78;
  color: white;
}

.file-list {
  width: 100%;
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
  background-color: #2d3748;
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
