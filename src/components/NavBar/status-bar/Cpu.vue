<template>
    <div class="cpu">
      <!-- chip icon -->
      <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
        <rect x="3" y="3" width="6" height="6" rx="0.8"/>
        <line x1="5" y1="1.5" x2="5" y2="3"/><line x1="7" y1="1.5" x2="7" y2="3"/>
        <line x1="5" y1="9" x2="5" y2="10.5"/><line x1="7" y1="9" x2="7" y2="10.5"/>
        <line x1="1.5" y1="5" x2="3" y2="5"/><line x1="1.5" y1="7" x2="3" y2="7"/>
        <line x1="9" y1="5" x2="10.5" y2="5"/><line x1="9" y1="7" x2="10.5" y2="7"/>
      </svg>
      <span class="cpu-val">{{ cpuUsage.toFixed(1) }}%</span>
    </div>
  </template>
  
  <script lang="ts">
  import { defineComponent } from 'vue';
  import { listen } from '@tauri-apps/api/event';
  import { info, warn, error } from '@tauri-apps/plugin-log';
  
  type SystemInfo = {
    cpu_usage: number;
  };
  
  export default defineComponent({
    name: 'Cpu',

    data() {
      return {
        cpuUsage: 0,
      };
    },
    mounted() {
      listen<SystemInfo>('cpu_usage_update', (event) => {
        if (!event || !event.payload) {
          warn('[CPU.vue] Event or payload is undefined');
          return;
        }

        const { cpu_usage } = event.payload;
  
        if (typeof cpu_usage === 'number') {
          this.cpuUsage = cpu_usage;
        } else {
          warn('[CPU.vue] Invalid cpu_usage:', cpu_usage);
        }
      }).then(unlisten => {
        info('[CPU.vue] Listener registered');
      }).catch(err => {
        error('[CPU.vue] Failed to register listener', err);
      });
    },
  });
  </script>
  
  <style scoped>
  .cpu {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .stat-icon {
    width: 11px;
    height: 11px;
    flex-shrink: 0;
    color: #585870;
  }

  .cpu-val {
    font-family: monospace;
    font-size: 11px;
    color: #6a6a80;
    min-width: 38px;
  }
  </style>
  