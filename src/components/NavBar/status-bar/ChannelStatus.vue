<template>
    <div class="channel-status">
      <p :title="progress + '%'">🚨​:</p>
      <p> {{ progress }}%</p>
      <div class="progress-bar-background">
        <div class="progress-bar" 
          :style="{ width: progress + '%' }" 
          >
        </div>
        
      </div>
    </div>
  </template>
  
  <script lang="ts">
  import { listen } from '@tauri-apps/api/event'

  import { defineComponent } from 'vue'
  
  interface ChannelPayload {
    channel_size: number
    current_size: number
  }
  
  export default defineComponent({
    data() {
      return {
        progress: 0
      }
    },
    mounted() {
      listen<ChannelPayload>('channel', (event) => {
        const { channel_size, current_size } = event.payload
  
        const computed = Math.min(100, (current_size * 100) / channel_size)
        this.progress = Math.round(computed)
  
      })
    }
  })
  </script>
  
  
  <style scoped>
  .channel-status {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .progress-bar-background {
    width: 100px; /* Taille fixe de la barre */
    height: 3px;
    background-color: #f7f7f7;
  }
  
  .progress-bar {
    height: 100%;
    background-color: #e79a6e;
 
  }
  </style>
  