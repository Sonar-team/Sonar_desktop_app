<template>
  <div class="container">
    <div class="center-container">

      <h1 v-show="same_ip_diff_mac.length > 0 || same_ip_diff_label.length > 0" class="dialog-title">Conflits détectés</h1>
      <h1 v-show="invalid_mac.length > 0 || invalid_ip.length > 0" class="dialog-title">MAC/IP invalide(s) détectée(s)</h1>
      <h1 v-show="conflictual_files.length > 0" class="dialog-title">Fichiers en conflit détectés</h1>

      <div class="panels">
        <div class="left-panel">
          <img class="image" src="/src/assets/images/warning-sign.png"/>
        </div>
        <div class="right-panel">

          <div v-show="same_ip_diff_mac.length > 0 || same_ip_diff_label.length > 0" class="file-list">
              <ul v-show="same_ip_diff_mac.length > 0">
                <h3 class="text">Conflits IP -> MAC</h3>
                <li v-for="([ip, ref_mac, name_i, mac, name_j], index) in same_ip_diff_mac" :key="index">
                  <label>
                    <span class="text">'{{ ip }}'(IP):</span><br>
                    <span class="text indented">MAC: '{{ ref_mac }}' <---- {{ name_i.length > 60 ? name_i.slice(0, 60) + '...' : name_i }}</span>
                    <span class="text indented">MAC: '{{ mac }}' <---- {{ name_j.length > 60 ? name_j.slice(0, 60) + '...' : name_j }}</span>
                  </label>
                </li>
              </ul>
              <ul v-show="same_ip_diff_label.length > 0">
                <h3 class="text">Conflits IP -> Label</h3>
                <li v-for="([ip, ref_label, name_i, label, name_j], index) in same_ip_diff_label.sort()" :key="index">
                  <label>
                    <span class="text">'{{ ip }}'(IP):</span><br>
                    <span class="text indented">Label: '{{ ref_label }}' <---- {{ name_i.length > 60 ? name_i.slice(0, 60) + '...' : name_i }}</span>
                    <span class="text indented">Label: '{{ label }}' <---- {{ name_j.length > 60 ? name_j.slice(0, 60) + '...' : name_j }}</span>
                  </label>
                </li>
              </ul>
          </div>

          <div v-show="invalid_mac.length > 0 || invalid_ip.length > 0" class="file-list">
              <ul v-show="invalid_mac.length > 0">
                <h3 class="text">MAC invalides</h3>
                <li v-for="([name, mac], index) in invalid_mac" :key="index">
                  <label>
                    <span class="text indented">MAC: '{{ mac }}' <---- {{ name.length > 60 ? name.slice(0, 60) + '...' : name }}</span>
                  </label>
                </li>
              </ul>
              <ul v-show="invalid_ip.length > 0">
                <h3 class="text">IP invalides</h3>
                <li v-for="([name, ip], index) in invalid_ip" :key="index">
                  <label>
                    <span class="text indented">IP: '{{ ip }}' <---- {{ name.length > 60 ? name.slice(0, 60) + '...' : name }}</span>
                  </label>
                </li>
              </ul>
          </div>

          <div v-show="conflictual_files.length > 0" class="file-list">
              <ul>
                <h3 class="text">Fichiers non importés</h3>
                <li v-for="(name, index) in conflictual_files" :key="index">
                  <label>
                    <span class="text indented">{{ name.length > 100 ? name.slice(0, 100) + '...' : name }}</span>
                  </label>
                </li>
              </ul>
          </div>
        </div>
      </div>

      <div>
          <button class="btn image-btn" @click.prevent="windowClosed">❌</button>
      </div>
     
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue';

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
    invalid_mac: {
      type: Array as PropType<[string, string][]>,
      required: true
    },
    invalid_ip: {
      type: Array as PropType<[string, string][]>,
      required: true
    },
    conflictual_files: {
      type: Array as PropType<string[]>,
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
  flex-direction: column;
  align-items: flex-start;
}

.text {
  color: whitesmoke
}

.indented {
  display: block;
  padding-left: 2rem; 
}

.file-list {
  width: 100%;
  max-height: 1000px;
  overflow-y: auto;
  background-color: #2d3748;
  border-radius: 4px;
  padding: 0.5rem;
  margin-bottom: 1.5rem;
}

.file-list li {
  list-style: none;
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
