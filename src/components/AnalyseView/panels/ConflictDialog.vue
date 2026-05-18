<template>
  <div class="container">
    <div class="center-container">
      <h1 class="dialog-title">Conflits détectés</h1>
      <div class="panels">
        <div class="left-panel">
          <img class="image" src="/src/assets/images/warning-sign.png"/>
        </div>
        <div class="right-panel">
              <ul v-show="same_ip_diff_mac.length > 0" class="file-list">
                <h3 class="text">Conflits adresses IP -> MAC</h3>
                <li v-for="([ip, ref_mac, name_i, mac, name_j], index) in same_ip_diff_mac" :key="index">
                  <label :for="String(index)">
                    <span class="text">IP '{{ ip }}' : MAC '{{ ref_mac }}' ({{ name_i }}) vs '{{ mac }}' ({{ name_j }})</span>
                  </label>
                </li>
              </ul>
              <ul v-show="same_ip_diff_label.length > 0" class="file-list">
                <h3 class="text">Conflits adresses IP -> Label</h3>
                <li v-for="([ip, ref_label, name_i, label, name_j], index) in same_ip_diff_label" :key="index">
                  <label :for="String(index)">
                    <span class="text">'{{ ip }}': '{{ ref_label }}'  ({{ name_i.length > 20 ? name_i.slice(0, 25) + '...' : name_i }})  <-> '{{ label }}'  ({{ name_j.length > 20 ? name_j.slice(0, 20) + '...' : name_j }}) </span>
                  </label>
                </li>
              </ul>
              <ul v-show="same_mac_diff_ip.length > 0" class="file-list">
                <h3 class="text">Conflits adresses MAC -> IP</h3>
                <li v-for="([mac, ref_ip, name_i, ip, name_j], index) in same_ip_diff_mac" :key="index">
                  <label :for="String(index)">
                    <span class="text">MAC '{{ mac }}' : IP '{{ ref_ip }}' ({{ name_i }}) vs '{{ ip }}' ({{ name_j }})</span>
                  </label>
                </li>
              </ul>
              <ul v-show="same_mac_diff_label.length > 0" class="file-list">
                <h3 class="text">Conflits adresses MAC -> Label</h3>
                <li v-for="([mac, ref_label, name_i, label, name_j], index) in same_ip_diff_mac" :key="index">
                  <label :for="String(index)">
                    <span class="text">IP '{{ mac }}' : MAC '{{ ref_label }}' ({{ name_i }}) vs '{{ label }}' ({{ name_j }})</span>
                  </label>
                </li>
              </ul>
          </div>
        <div>
          <button class="btn image-btn" @click.prevent="windowClosed">❌</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';
import { displayCaptureError } from '../../../errors/capture';

type ConflictRow = [string, string, string, string, string]

export default defineComponent({
  name: 'ConflictDialog',
  emits: ['showConflictDialog'],

  props: {
    same_ip_diff_mac: {
      type: Array as PropType<ConflictRow[]>,
      required: true
    },
    same_ip_diff_label: {
      type: Array as PropType<ConflictRow[]>,
      required: true
    },
    same_mac_diff_ip: {
      type: Array as PropType<ConflictRow[]>,
      required: true
    },
    same_mac_diff_label: {
      type: Array as PropType<ConflictRow[]>,
      required: true
    }
  },
  data() {
    return {
      //conflictualFiles: [] as [string, string][],
      //localFiles: [...this.files]
    };
  },

  /*
  watch: {
    localFiles: {
      immediate: true,
      handler(newVal) {
        if (newVal.length === 0) {
          this.windowClosed();
        }
      }
    }
  },
  */

  methods: {
    windowClosed() {
      this.$emit('showConflictDialog', false);
    },
    /*
    async forceCopy(path: string) {

      info('force_import: ' + path);

      try {
        await invoke('force_import', { csvPath: path });
        info('réponse invoke');
        this.localFiles = this.localFiles.filter(([name]) => name !== path);
      } catch (err) {
        displayCaptureError(err);
      }
    },

    async cancelCopy(path: string) {

      info('cancel_copy: ' + path);
        this.localFiles = this.localFiles.filter(([, file_path]) => file_path !== path);
    },

  */
  }
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
  width: 100%;
  max-width: 1200px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.panels {
  display: flex;
  flex-direction: row;
  width: 100%;
  align-items: stretch;
}

.dialog-title {
  color: whitesmoke;
  font-size: 2rem;
  margin: 0 0 1rem 0;
  text-align: center;
}

.left-panel {
  justify-content: center;
  width: 20%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.right-panel {
  width: 75%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}


.csv-group {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.file-group {
  display: flex;
  flex-direction:row;
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
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
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


.image {
  width: 100%;
  max-width: 180px;
  height: auto;
  object-fit: contain;
  margin-right: auto;
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
  border-radius: 4px;
  word-break: break-all;
  font-family: monospace;
  font-size: 0.9rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
