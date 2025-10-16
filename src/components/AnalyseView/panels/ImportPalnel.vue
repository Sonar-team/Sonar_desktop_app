<template>
  <div class="container">
    <div class="center-container">
      <div class="file-group">
        <label for="packetFiles"></label>
        <button class="btn" @click="addFiles" >
          Ajouter des fichiers
        </button>
        <button class="btn btn-clear" @click="clearFiles" >
          Effacer
        </button>
      </div>

      <ul class="file-list" v-if="packetFiles.length > 0">
        <li v-for="(file, index) in packetFiles" :key="index">
          {{ file }}
        </li>
      </ul>


      <button
        @click="convert"
        class="btn btn-open"
      >
        Ouvrir
      </button>


    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke, Channel } from '@tauri-apps/api/core';
import { info, error } from '@tauri-apps/plugin-log';
import { useCaptureStore } from '../../../store/capture';
import { CaptureEvent } from '../../../types/capture';
import { displayCaptureError } from '../../../errors/capture';

export default defineComponent({
  name: 'ImportPanel',
  emits: ['toggle-pcap'],
  data() {
    return {
      packetFiles: [] as string[],
      
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
    async addFiles() {
      const files = await open({
        multiple: true,
        filters: [{ name: 'Capture File', extensions: ['pcap', 'pcapng', 'cap'] }]
      });
      if (files) {
        const list = Array.isArray(files) ? files : [files];
        this.packetFiles.push(...list);
      }
    },
    clearFiles() {
      this.packetFiles = [];
    },
    async convert() {
      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent);
      info('convert_from_pcap_list : ' + this.packetFiles);
       await invoke('convert_from_pcap_list', { pcaps: this.packetFiles, onEvent })
        .then(() => {
          
          info('reponse invok ');
        })
        .catch(displayCaptureError);
    },
  },
  mounted() {
    this.captureStore.onStarted(() => {
      info("started hearded")
      this.captureStore.updateStatus({is_running: true});
    });

    this.captureStore.onFinished(() => {
      info("finished hearded")
      this.captureStore.updateStatus({is_running: false});
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

.progress-container {
  width: 100%;
  margin: 1rem 0;
}

.progress-bar {
  height: 8px;
  background-color: #2d3748;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 0.5rem;
}

.progress {
  height: 100%;
  background-color: #4299e1;
  transition: width 0.3s ease;
}

.progress-text {
  text-align: center;
  font-size: 0.875rem;
  color: #a0aec0;
}

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