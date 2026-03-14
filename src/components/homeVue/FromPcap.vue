<template>
    <div class="container">
      <div class="image-container">
        <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">
      </div>
  
      <div class="center-container">
        <div class="file-group">
          <label for="packetFiles"></label>
          <button class="btn" @click="addFiles">Ajouter des fichiers</button>
          <button class="btn btn-clear" @click="clearFiles">Effacer</button>
        </div>
        
        <ul class="file-list">
          <li v-for="(file, index) in packetFiles" :key="index">{{ file }}</li>
        </ul>
        <button @click="goToReadPage" class="btn btn-open">Ouvrir</button>
      </div>
    </div>
    <router-view></router-view>
  </template>
  
  <script>
  import { open } from '@tauri-apps/plugin-dialog';
  
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
      goToReadPage() {
        this.$router.push({
            name: 'ReadPcap',
            query: {
            pcapList: JSON.stringify(this.packetFiles)
            }
        });
        }
    }
  };
  </script>
  
  <style scoped>
  .container {
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
    height: 50vh;
    border-radius: 12px;
    padding: 20px;
    background-color: #343448;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    color: #d4d4d8;
    border: 1px solid #484858;
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
    border: 1px solid #545465;
    border-radius: 5px;
    cursor: pointer;
    background-color: #2e4a68;
    color: #d4d4d8;
  }

  .btn-clear {
    background-color: #4a2828;
    border-color: #5c3030;
  }

  .btn-open {
    background-color: #284a28;
    border-color: #305c30;
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
  