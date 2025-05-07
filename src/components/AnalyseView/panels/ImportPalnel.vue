<template>
    <div class="container">

      <div class="center-container">
        <div class="file-group">
          <label for="packetFiles"></label>
          <button class="btn" @click="addFiles">Ajouter des fichiers</button>
          <button class="btn btn-clear" @click="clearFiles">Effacer</button>
        </div>
        
        <ul class="file-list">
          <li v-for="(file, index) in packetFiles" :key="index">{{ file }}</li>
        </ul>
        <button @click="convert" class="btn btn-open">Ouvrir</button>
      </div>
    </div>
    <router-view></router-view>
  </template>

<script>
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';

export default {
  data() {
    return {
      packetFiles: []
    };
  },
  methods: {
    async addFiles() {
      const files = await open({
        multiple: true,
        filters: [{ name: 'Capture File', extensions: ['pcap', 'pcapng', 'cap'] }]
      });
      if (files) {
        this.packetFiles.push(...files);
      }
    },
    clearFiles() {
      this.packetFiles = [];
    },
    convert() {
        invoke('convert_from_pcap_list', { pcaps: this.packetFiles })
            .then(response => {
                this.totalPackets = response; // Définit totalPackets avec la réponse renvoyée
                info(`Total packets read: ${this.totalPackets}`);
            })
            .catch(error => console.error(error));
      }
  }
};
</script>

<style scoped>

.container {
    position: fixed; /* rend la div flottante au-dessus du reste */

    top: 50%;
  left: 50%;
  transform: translate(-50%, -50%); /* centre la div parfaitement */
  z-index: 1000; /* s'assure qu'elle passe au-dessus des autres */

  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 75vh;
}

.image-container {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 20px;
}

.center-container {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  border-radius: 15px;
  padding: 20px;
  background-color: #1a1a1a;
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  color: #FFF;
  border: 2px solid #3a3a3a;
  margin: auto;
}

.file-group {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
}

.btn {
  padding: 8px 12px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  background-color: #007bff;
  color: white;
}

.btn-clear {
  background-color: #dc3545;
}

.btn-open {
  background-color: #218621;
}

.file-list {
  margin-top: 20px;
  list-style-type: none;
  padding: 0;
  max-height: 200px;
  overflow-y: auto;
}

.file-list li {
  padding: 5px 10px;
  background-color: #16181a;
  margin-bottom: 5px;
  border-radius: 5px;
}
</style>