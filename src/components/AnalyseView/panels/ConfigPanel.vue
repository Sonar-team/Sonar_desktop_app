<template>
  <div class="config-panel">
    <h2>Configuration Capture</h2>

    <InterfaceSelector 
      v-model="selectedInterface"
      :net-interfaces="netInterfaces"
      class="config-item"
    />

    <div class="config-item">
      <label>Taille du buffer :</label>
      <input type="number" v-model.number="bufferSize" min="65536" max="536870912" step="1024" />
    </div>

    <div class="config-item">
      <label>Nombre de buffers:</label>
      <input type="number" v-model.number="chanCapacity" min="1" max="1000000" />
    </div>

    <div class="config-item">
      <label>Timeout (ms) :</label>
      <input type="number" v-model.number="timeout" min="1" max="10000" />
    </div>

    <div class="config-item">
      <label>Taille du snaplen :</label>
      <input type="number" v-model.number="snaplen" min="64" max="262144" />
    </div>

    <div class="actions">
      <button @click="save" :disabled="!selectedInterface">Sauvegarder</button>
      <button @click="close">Fermer</button>
    </div>
  </div>
</template>

<script>
import { info } from "@tauri-apps/plugin-log";
import { invoke } from "@tauri-apps/api/core";
import { displayCaptureError } from "../../../errors/capture";
import { useCaptureConfigStore } from "../../../store/capture";
import { DEFAULT_CAPTURE_CONFIG } from "../../../types/config";
import InterfaceSelector from "./CustomSelector/interfaceSelector.vue";

export default {
  name: "ConfigPanel",
  components: { InterfaceSelector },
  emits: ["update:ConfigPanel-visible"],

  data() {
    return {
      netInterfaces: [],
      selectedInterface: null,
      deviceName: "",
      bufferSize: DEFAULT_CAPTURE_CONFIG.buffer_size,
      chanCapacity: DEFAULT_CAPTURE_CONFIG.chan_capacity,
      timeout: DEFAULT_CAPTURE_CONFIG.timeout,
      snaplen: DEFAULT_CAPTURE_CONFIG.snaplen,
    };
  },

  computed: {
    configStore() {
      return useCaptureConfigStore();
    },
  },

  methods: {
    syncSelectedInterface() {
      if (!this.deviceName || !this.netInterfaces.length) return;
      this.selectedInterface = this.netInterfaces.find((iface) => iface.name === this.deviceName) || null;
    },

    async getConfig() {
      try {
        const config = await invoke("get_config_capture");
        info(`[ConfigPanel] config=${JSON.stringify(config)}`);

        this.deviceName = config.device_name || "";
        this.bufferSize = config.buffer_size;
        this.chanCapacity = config.chan_capacity;
        this.timeout = config.timeout;
        this.snaplen = config.snaplen;

        this.configStore.updateConfig(config);
        this.syncSelectedInterface();
      } catch (err) {
        await displayCaptureError(err);
      }
    },

    async loadInterfaces() {
      try {
        this.netInterfaces = await invoke("get_devices_list");
        this.syncSelectedInterface();
      } catch (err) {
        await displayCaptureError(err);
      }
    },

    async save() {
      if (!this.selectedInterface?.name) {
        await displayCaptureError({
          kind: "capture",
          message: { kind: "invalidConfig", message: "interface réseau obligatoire" },
        });
        return;
      }

      try {
        const config = await invoke("config_capture", {
          device_name: this.selectedInterface.name,
          buffer_size: Number(this.bufferSize),
          chan_capacity: Number(this.chanCapacity),
          timeout: Number(this.timeout),
          snaplen: Number(this.snaplen),
        });

        this.deviceName = config.device_name || "";
        this.bufferSize = config.buffer_size;
        this.chanCapacity = config.chan_capacity;
        this.timeout = config.timeout;
        this.snaplen = config.snaplen;
        this.configStore.updateConfig(config);
        this.syncSelectedInterface();
        this.close();
      } catch (err) {
        await displayCaptureError(err);
      }
    },

    close() {
      this.$emit("update:ConfigPanel-visible", false);
    },
  },

  mounted() {
    this.loadInterfaces();
    this.getConfig();
  },

  watch: {
    selectedInterface(nextInterface) {
      this.deviceName = nextInterface?.name || "";
    },
    netInterfaces() {
      this.syncSelectedInterface();
    },
  },
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
  align-items: stretch;
  width: 400px;
  background-color: #1a1a1a;
  color: #fff;
  padding: 20px;
  border-radius: 15px;
  border: 2px solid #3a3a3a;
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  font-family: sans-serif;
  box-sizing: border-box;
}

.config-panel h2 {
  margin-bottom: 20px;
  font-size: 20px;
  text-align: center;
}

.config-item {
  width: 100%;
  margin-bottom: 15px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.config-item label {
  display: block;
  font-weight: bold;
  width: 100%;
  text-align: left;
}

.config-item input,
.config-panel select {
  width: 100%;
  padding: 10px;
  background-color: #2e2a36;
  border: 1px solid #666;
  color: #fff;
  border-radius: 6px;
  font-size: 14px;
  transition: all 0.2s ease;
  box-sizing: border-box;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
  background-image: url("data:image/svg+xml;charset=US-ASCII,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22292.4%22%20height%3D%22292.4%22%3E%3Cpath%20fill%3D%22%23ffffff%22%20d%3D%22M287%2069.4a17.6%2017.6%200%200%200-13-5.4H18.4c-5%200-9.3%201.8-12.9%205.4A17.6%2017.6%200%200%200%200%2082.2c0%205%201.8%209.3%205.4%2012.9l128%20127.9c3.6%203.6%207.8%205.4%2012.8%205.4s9.2-1.8%2012.8-5.4L287%2095c3.5-3.5%205.4-7.8%205.4-12.8%200-5-1.9-9.2-5.5-12.8z%22%2F%3E%3C%2Fsvg%3E");
  background-repeat: no-repeat;
  background-position: right 0.7em top 50%;
  background-size: 0.65em auto;
  padding-right: 2.5em;
  cursor: pointer;
}

.config-item input:focus {
  outline: none;
  border-color: #c53d3d;
  box-shadow: 0 0 0 2px rgba(58, 142, 230, 0.3);
}

select {
  background-color: #2e2a36;
  color: #ffffff;
  border: 1px solid #555;
}

.config-panel select option {
  background-color: #2e2a36 !important;
  color: #ffffff !important;
  padding: 10px;
  border: none;
}

/* Style pour les options au survol */
.config-panel select option:hover,
.config-panel select option:focus,
.config-panel select option:checked {
  background-color: #c53d3d !important;
  color: white !important;
}

select:focus {
  border-color: #c53d3d;
  box-shadow: 0 0 0 2px rgba(197, 61, 61, 0.3);
  outline: none;
}

.actions {
  display: flex;
  justify-content: space-between;
  width: 100%;
  margin-top: 10px;
  gap: 10px;
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

.actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

</style>
