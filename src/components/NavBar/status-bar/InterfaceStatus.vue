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
import { CaptureConfig, useCaptureConfigStore } from '../../../store/capture';
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
  },

  methods: {
    async getconfig() {
      try {
        const config = await invoke<CaptureConfig>('get_config_capture');
        this.configStore.updateConfig(config);
        this.selectedDevice = config.device_name; // <-- Utilise device_name ici !
        
      } catch (err) {
        await displayCaptureError(err);
      }
    }
  },
  mounted() { 
    this.getconfig();
  },
};
</script>

<style scoped>
.interface-btn {
  background-color: #243452;
  color: #ffffff;
  border: none;
  padding: 3px 10px;
  font-size: 12px;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 3px;
  user-select: none;
  transition: background-color 0.2s, opacity 0.2s;
}

</style>
