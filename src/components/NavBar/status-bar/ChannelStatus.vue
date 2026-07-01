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
  import { defineComponent } from 'vue'
  import { useCaptureStore } from '../../../store/capture'
  
  interface ChannelPayload {
    channel_size: number
    current_size: number
  }
  
  export default defineComponent({
    data() {
      return {
        progress: 0,
        unlisten: undefined as undefined | (() => void),
      }
    },
    mounted() {
      const captureStore = useCaptureStore()
      this.unlisten = captureStore.onChannelCapacityPayload((payload: ChannelPayload) => {
        const { channel_size, current_size } = payload

        const computed = channel_size > 0
          ? Math.min(100, (current_size * 100) / channel_size)
          : 0
        this.progress = Math.round(computed)
      })
    },
    beforeUnmount() {
      this.unlisten?.()
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
  
