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
import { displayCaptureError } from '../../../errors/capture';
import { useCaptureConfigStore } from '../../../store/capture';

export default {
  name: "ConfigPanel",

  emits: ['update:ConfigPanel-visible'],

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
      this.$emit('update:ConfigPanel-visible', false);
    }
  },

  async mounted() {
    info("[ConfigPanel] Monté avec visible =", this.visible);
    this.getConfig();

    invoke('get_interfaces_tab').then((interfaces) => {
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
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 1000;

  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;

  width: 400px;
  background-color: #1a1a1a;
  color: #fff;
  padding: 20px;
  border-radius: 15px;
  border: 2px solid #3a3a3a;
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  font-family: sans-serif;
}

.config-panel h2 {
  margin-bottom: 20px;
  font-size: 20px;
  text-align: center;
}

.config-item {
  width: 100%;
  margin-bottom: 15px;
}

.config-item label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.config-item input,
select {
  width: 100%;
  padding: 8px;
  background-color: #16181a;
  border: 1px solid #555;
  color: white;
  border-radius: 6px;
  font-size: 14px;
  transition: border 0.2s, box-shadow 0.2s;
}

.config-item input:focus,
select:focus {
  outline: none;
  border-color: #c53d3d;
  box-shadow: 0 0 0 2px rgba(58, 142, 230, 0.3);
}

select option {
  background-color: #2e2a36;
  color: white;
}

.actions {
  display: flex;
  justify-content: space-between;
  width: 100%;
  margin-top: 20px;
}

.actions button {
  padding: 8px 12px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  background-color: #007bff;
  color: white;
  font-weight: bold;
  transition: background-color 0.2s;
}

.actions button:nth-child(2) {
  background-color: #dc3545;
}

.actions button:hover {
  opacity: 0.9;
}

</style>
