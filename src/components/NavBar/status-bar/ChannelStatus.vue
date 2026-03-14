<template>
    <div class="channel-status" title="Remplissage du buffer canal">
      <!-- buffer/layers icon -->
      <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
        <path d="M1.5 4l4.5 2 4.5-2-4.5-2-4.5 2z"/>
        <path d="M1.5 6.5l4.5 2 4.5-2"/>
        <path d="M1.5 9l4.5 2 4.5-2"/>
      </svg>
      <span class="buf-pct">{{ progress }}%</span>
      <div class="progress-bar-background">
        <div class="progress-bar" :style="{ width: progress + '%' }"></div>
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

  .stat-icon {
    width: 11px;
    height: 11px;
    flex-shrink: 0;
    color: #585870;
  }

  .buf-pct {
    font-family: monospace;
    color: #6a6a80;
    font-size: 11px;
    min-width: 28px;
  }

  .progress-bar-background {
    width: 60px;
    height: 2px;
    background-color: #3c3c50;
  }

  .progress-bar {
    height: 100%;
    background-color: #4a6080;
    transition: width 0.3s cubic-bezier(0.25, 1, 0.5, 1);
  }
  </style>
  