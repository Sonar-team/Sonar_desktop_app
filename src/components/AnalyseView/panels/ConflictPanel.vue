<template>
  <div class="container">
    <div class="center-container">
      <h3 v-if="localFiles.length > 1" class="text">Ces fichiers sont déjà enregistrés, voulez-vous les remplacer ?</h3>
      <h3 v-else class="text">Ce fichier est déjà enregistré, voulez-vous le remplacer ?</h3> 
          <ul class="file-list">
            <li v-for="([file_name, file_path], index) in localFiles" :key="index">
              <label :for="String(index)">
                <span class="text">{{ file_name }}</span>
                <div style="margin-left: auto;"></div>
                <button class="btn btn-clear text" @click="forceCopy(file_path)" title="Supprimer">Oui</button>
                <button class="btn btn-add text" @click="cancelCopy(file_path)" title="Annuler">Non</button>
              </label>
            </li>
          </ul>
      </div>
    </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';
import { displayCaptureError } from '../../../errors/capture';


export default defineComponent({
  name: 'ConflictPanel',
  emits: ['showConflictPanel'],
  props: {
    files: {
      type: Array as PropType<[string, string][]>,
      required: true,
    },
  },
  data() {
    return {
      conflictualFiles: [] as [string, string][],
      localFiles: [...this.files]
    };
  },

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

  methods: {
    windowClosed() {
      this.$emit('showConflictPanel', false);
    },

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
  align-items: center;
  background-color: #1e1e2e;
  border-radius: 8px;
  padding: 2rem;
  width: 100%;
  max-width: 800px;
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

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
