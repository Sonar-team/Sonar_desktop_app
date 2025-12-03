<template>
    <div class="cpu">
      <img src="/src/assets/images/TablerCpu.svg" alt="Flux" class="icon-img" />
      <p>{{ cpuUsage.toFixed(2) }}%</p>
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
    gap: 5px;
  }
  
  .icon-img {
    height: 15px;
    width: 15px;
    vertical-align: middle;
  }
  </style>
  