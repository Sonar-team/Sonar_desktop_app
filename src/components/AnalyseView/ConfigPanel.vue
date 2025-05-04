<template>
  <div class="config-panel">
    <h2>Configuration Capture</h2>

    <select v-model="deviceName">
      <option v-for="netInterface in netInterfaces" :key="netInterface" :value="netInterface">
        {{ netInterface }}
      </option>
    </select>

    <div class="config-item">
      <label>Taille du buffer :</label>
      <input type="number" v-model.number="bufferSize" />
    </div>

    <div class="config-item">
      <label>Timeout (ms) :</label>
      <input type="number" v-model.number="timeout" />
    </div>

    <div class="actions">
      <button @click="save">Sauvegarder</button>
      <button @click="close">Fermer</button>
    </div>
  </div>
</template>

<script>
import { info } from '@tauri-apps/plugin-log';
import { invoke } from '@tauri-apps/api/core';
import { displayCaptureError } from '../../errors/capture';
import { useCaptureConfigStore } from '../../store/capture';

export default {
  name: "ConfigPanel",

  emits: ['update:visible'],

  data() {
    return {
      netInterfaces: [],
      selectedNetInterface: '',

      deviceName: '',
      bufferSize: '',
      timeout: '',
    };
  },

  computed: {
    configStore() {
      return useCaptureConfigStore();
    }
  },

  methods: {
    async getConfig() {
      try {
        const config = await invoke('get_config_capture');
        info("[ConfigPanel] invoke response =", config);

        // Cast si besoin
        this.deviceName = config.device_name;
        this.bufferSize = config.buffer_size;
        this.timeout = config.timeout;

        this.configStore.updateConfig(config);
        info("[ConfigPanel] configStore =", this.configStore);
      } catch (err) {
        console.error("[ConfigPanel] erreur get_config_capture :", err);
      }
    },

    async save() {
      info("Configuration sauvegardée : " + JSON.stringify({
        deviceName: this.deviceName,
        bufferSize: this.bufferSize,
        timeout: this.timeout,
      }));
      try {
        const config = await invoke('config_capture', { device_name: this.deviceName, buffer_size: this.bufferSize, timeout: this.timeout }); // await invoke('config_capture', this.deviceName, this.bufferSize, this.timeout);
                // Cast si besoin
        this.deviceName = config.device_name;
        this.bufferSize = config.buffer_size;
        this.timeout = config.timeout;

        this.configStore.updateConfig(config);
      } catch (err) {
        console.error("[ConfigPanel] erreur get_config_capture :", err);
      }
      this.close();
    },

    close() {
      this.$emit('update:visible', false);
    }
  },

  async mounted() {
    info("[ConfigPanel] Monté avec visible =", this.visible);
    this.getConfig();

    invoke('get_devices_list').then((interfaces) => {
      this.netInterfaces = interfaces;
      if (interfaces.length > 0) {
        this.selectedNetInterface = interfaces[interfaces.length - 1]; // Set the last item as default
      }
    }).catch(error => {
      console.error("Failed to load interfaces:", error);
    });
  },
  watch: {
    visible(newVal) {
      info("[ConfigPanel] Changement de visible :", newVal);
      if (newVal) {
        this.getConfig();
      }
    }
  }
};
</script>

<style scoped>
.config-panel {
  position: fixed;
  top: 40px;
  width: 300px;
  background-color: #2e2a36;
  color: white;
  padding: 20px;
  border-left: 1px solid #252526;
  box-shadow: -2px 0 5px rgba(0, 0, 0, 0.5);
}

.config-item {
  margin-bottom: 15px;
}

.config-item label {
  display: block;
  margin-bottom: 5px;
}

.config-item input {
  width: 100%;
  padding: 5px;
  border: none;
  border-radius: 4px;
}

.actions {
  display: flex;
  justify-content: space-between;
  margin-top: 20px;
}

.actions button {
  background-color: #ffffff;
  border: none;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.actions button:hover {
  background-color: #5a6274;
}
</style>
