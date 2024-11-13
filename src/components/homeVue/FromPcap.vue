<template>
    <div class="container">
      <div class="image-container">
        <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">
      </div>
  
      <div class="center-container">
        <div class="file-group">
          <label for="packetFiles">Packet File:</label>
          <button class="btn" @click="addFiles">Add File(s)</button>
          <button class="btn btn-clear" @click="clearFiles">Clear</button>
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
  import { open } from '@tauri-apps/api/dialog';
  
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
  