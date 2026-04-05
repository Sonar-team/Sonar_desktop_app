<template>
  <div>
    <p 
      class="interface-btn"
      >
        Interface : {{ config }}
    </p>

  </div>
</template>

<script lang="ts">
import { info } from '@tauri-apps/plugin-log';
import { CaptureConfig, useCaptureConfigStore, useCaptureStore } from '../../../store/capture';
import { invoke } from '@tauri-apps/api/core';
import { displayCaptureError } from '../../../errors/capture';

export default {
  name: 'InterfaceStatus',

  data() {
    return {
      devices: [] as string[],
      selectedDevice: '', // valeur sélectionnée
    };
  },

  computed: {
    configStore() {
      return useCaptureConfigStore();
    },
    config() {
      return this.configStore.interface;
    },
    captureStore() {
      return useCaptureStore();
    },

  },

  methods: {
    async getconfig() {
      try {
        const config = await invoke<CaptureConfig>('get_config_capture');
        this.configStore.updateConfig(config);
        this.selectedDevice = config.device_name;
        
      } catch (err) {
        await displayCaptureError(err);
      }
    }
  },
  mounted() { 
    this.getconfig();
    this.captureStore.onFinished((f) => {
      
    });
    this.captureStore.onStarted((f) => {
      
    });
  },
};
</script>

<style scoped>
.interface-btn {
  background-color: transparent;
  color: #6a6a80;
  border: none;
  padding: 3px 6px;
  font-size: 11px;
  display: flex;
  align-items: center;
  cursor: pointer;
  border-radius: 3px;
  user-select: none;
  transition: color 0.2s;
  max-width: 220px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

</style>
