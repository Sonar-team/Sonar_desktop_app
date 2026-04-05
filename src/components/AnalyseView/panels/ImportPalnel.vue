<template>
  <div class="panel-backdrop" @click.self="$emit('update:visible', false)">
    <div class="panel">

      <!-- Header -->
      <div class="panel-header">
        <div class="panel-title">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
            <path d="M1.5 4.5h4l1.5 2H14v7H1.5V4.5z"/>
            <path d="M1.5 6.5v-3a1 1 0 0 1 1-1h3"/>
          </svg>
          Importer PCAP
        </div>
        <button class="close-btn" @click="$emit('update:visible', false)" title="Fermer">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="4" y1="4" x2="12" y2="12"/>
            <line x1="12" y1="4" x2="4" y2="12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="panel-body">

        <!-- Drop zone / empty state -->
        <div class="drop-zone" v-if="packetFiles.length === 0" @click="addFiles">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 3v12M8 11l4 4 4-4"/>
            <path d="M3 17v2a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-2"/>
          </svg>
          <span>Cliquer pour sélectionner</span>
          <span class="drop-hint">.pcap · .pcapng · .cap</span>
        </div>

        <!-- File list -->
        <ul class="file-list" v-else>
          <li v-for="(file, index) in packetFiles" :key="index" class="file-item">
            <svg class="file-icon" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 1h5.5L11 3.5V13H3V1z"/>
              <path d="M8.5 1v2.5H11"/>
            </svg>
            <span class="file-name" :title="file">{{ basename(file) }}</span>
            <button class="remove-btn" @click="removeFile(index)" title="Retirer" :disabled="isConverting">
              <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
                <line x1="2" y1="2" x2="10" y2="10"/>
                <line x1="10" y1="2" x2="2" y2="10"/>
              </svg>
            </button>
          </li>
        </ul>

        <!-- Add more when list not empty -->
        <button v-if="packetFiles.length > 0" class="add-more-btn" @click="addFiles" :disabled="isConverting">
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
            <line x1="7" y1="2" x2="7" y2="12"/>
            <line x1="2" y1="7" x2="12" y2="7"/>
          </svg>
          Ajouter des fichiers
        </button>

      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <button class="btn-ghost" @click="clearFiles" :disabled="isConverting || packetFiles.length === 0">
          Effacer
        </button>
        <button class="btn-primary" @click="convert" :disabled="isConverting || packetFiles.length === 0">
          <span v-if="isConverting" class="spinner"></span>
          <svg v-else viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 7h10M8 3l4 4-4 4"/>
          </svg>
          {{ isConverting ? 'Chargement…' : 'Ouvrir' }}
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

export default defineComponent({
  name: 'ImportPanel',
  emits: ['update:visible'],

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
  },

  methods: {
    basename(path: string): string {
      return path.split(/[\\/]/).pop() || path;
    },

    async addFiles() {
      if (this.isConverting) return;
      const files = await open({
        multiple: true,
        filters: [{ name: 'Capture File', extensions: ['pcap', 'pcapng', 'cap'] }],
      });
      if (files) {
        const list = Array.isArray(files) ? files : [files];
        this.packetFiles.push(...list);
      }
    },

    removeFile(index: number) {
      this.packetFiles.splice(index, 1);
    },

    clearFiles() {
      this.packetFiles = [];
    },

    async convert() {
      if (this.packetFiles.length === 0) return;
      const onEvent = new Channel<CaptureEvent>();
      this.captureStore.setChannel(onEvent);
      info('convert_from_pcap_list : ' + this.packetFiles);
      this.isConverting = true;
      try {
        await invoke('convert_from_pcap_list', { pcapPaths: this.packetFiles, onEvent });
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
      this.captureStore.updateStatus({ is_running: true });
    });
    this.captureStore.onFinished(() => {
      this.captureStore.updateStatus({ is_running: false });
    });
  },
});
</script>

<style scoped>
.panel-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}

.panel {
  width: 340px;
  height: 100%;
  background: #363648;
  border-left: 1px solid #4a4a60;
  display: flex;
  flex-direction: column;
  animation: slide-in 0.22s cubic-bezier(0.25, 1, 0.5, 1);
}

@keyframes slide-in {
  from { transform: translateX(18px); opacity: 0; }
  to   { transform: translateX(0);     opacity: 1; }
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 14px 0 16px;
  border-bottom: 1px solid #4a4a60;
  flex-shrink: 0;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 500;
  color: #a8a8be;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.panel-title svg {
  width: 14px;
  height: 14px;
  color: #707090;
}

.close-btn {
  width: 26px;
  height: 26px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: #686884;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s;
}
.close-btn svg { width: 12px; height: 12px; }
.close-btn:hover { background: #484860; color: #b0b0c8; }

/* Body */
.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 18px 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Drop zone */
.drop-zone {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  border: 1px dashed #505068;
  border-radius: 6px;
  padding: 44px 24px;
  color: #686884;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
  min-height: 160px;
}
.drop-zone:hover {
  border-color: #707090;
  color: #a8a8be;
  background: rgba(255,255,255,0.02);
}
.drop-zone svg { width: 36px; height: 36px; opacity: 0.45; }
.drop-zone span { font-size: 13px; }
.drop-hint { font-size: 11px; color: #585870; letter-spacing: 0.05em; }

/* File list */
.file-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: #2c2c3a;
  border: 1px solid #404058;
  border-radius: 5px;
  min-width: 0;
}

.file-icon {
  width: 13px;
  height: 13px;
  flex-shrink: 0;
  color: #686884;
}

.file-name {
  flex: 1;
  font-family: monospace;
  font-size: 12px;
  color: #a8a8be;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.remove-btn {
  width: 20px;
  height: 20px;
  background: transparent;
  border: none;
  border-radius: 3px;
  color: #686884;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}
.remove-btn svg { width: 10px; height: 10px; }
.remove-btn:hover { background: #3a2020; color: #c09090; }
.remove-btn:disabled { opacity: 0.3; cursor: not-allowed; }

.add-more-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 8px;
  background: transparent;
  border: 1px dashed #484860;
  border-radius: 5px;
  color: #686884;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
}
.add-more-btn svg { width: 12px; height: 12px; }
.add-more-btn:hover { border-color: #686884; color: #a8a8be; }
.add-more-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* Footer */
.panel-footer {
  display: flex;
  gap: 8px;
  padding: 14px 18px;
  border-top: 1px solid #4a4a60;
  flex-shrink: 0;
}

.btn-ghost {
  flex: 1;
  padding: 8px 14px;
  background: transparent;
  border: 1px solid #484860;
  border-radius: 5px;
  color: #808096;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-ghost:active { transform: scale(0.97); }
.btn-ghost:hover { border-color: #60607a; color: #a8a8be; }
.btn-ghost:disabled { opacity: 0.35; cursor: not-allowed; }

.btn-primary {
  flex: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 14px;
  background: #304868;
  border: 1px solid #3e5c80;
  border-radius: 5px;
  color: #88b4cc;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-primary:active { transform: scale(0.97); }
.btn-primary svg { width: 13px; height: 13px; }
.btn-primary:hover { background: #3a5878; border-color: #4a6888; color: #aaced0; }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }

/* Spinner inline */
.spinner {
  width: 12px;
  height: 12px;
  border: 1.5px solid rgba(136, 180, 204, 0.2);
  border-left-color: #88b4cc;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
