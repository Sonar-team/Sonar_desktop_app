<template>
  <div class="panel-backdrop" @click.self="close">
    <div class="panel">

      <!-- Header -->
      <div class="panel-header">
        <div class="panel-title">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
            <circle cx="8" cy="8" r="2"/>
            <path d="M8 1.5v1M8 13.5v1M1.5 8h1M13.5 8h1M3.4 3.4l.7.7M11.9 11.9l.7.7M11.9 4.1l-.7.7M4.8 11.2l-.7.7"/>
          </svg>
          Configuration capture
        </div>
        <button class="close-btn" @click="close" title="Fermer">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="4" y1="4" x2="12" y2="12"/>
            <line x1="12" y1="4" x2="4" y2="12"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="panel-body">

        <!-- Interface -->
        <InterfaceSelector
          v-model="netDevice"
          :net-interfaces="netInterfaces"
        />

        <div class="divider"/>

        <!-- Numeric fields -->
        <div class="field">
          <label class="field-label">Taille du buffer</label>
          <input class="field-input" type="number" v-model.number="bufferSize" />
        </div>

        <div class="field">
          <label class="field-label">Nombre de buffers</label>
          <input class="field-input" type="number" v-model.number="chan_capacity" />
        </div>

        <div class="field">
          <label class="field-label">Timeout (ms)</label>
          <input class="field-input" type="number" v-model.number="timeout" />
        </div>

        <div class="field">
          <label class="field-label">Snaplen</label>
          <input class="field-input" type="number" v-model.number="snaplen" />
        </div>

      </div>

      <!-- Footer -->
      <div class="panel-footer">
        <button class="btn-ghost" @click="close">Annuler</button>
        <button class="btn-primary" @click="save">
          <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 7h10M8 3l4 4-4 4"/>
          </svg>
          Sauvegarder
        </button>
      </div>

    </div>
  </div>
</template>

<script>
import { info } from '@tauri-apps/plugin-log';
import { invoke } from '@tauri-apps/api/core';
import { useCaptureConfigStore } from '../../../store/capture';
import InterfaceSelector from './CustomSelector/interfaceSelector.vue';

export default {
  name: "ConfigPanel",
  components: { InterfaceSelector },
  emits: ['update:ConfigPanel-visible'],

  data() {
    return {
      netInterfaces: [],
      netDevice: '',
      bufferSize: '',
      chan_capacity: '',
      timeout: '',
      snaplen: '',
    };
  },

  computed: {
    configStore() { return useCaptureConfigStore(); }
  },

  methods: {
    async getConfig() {
      try {
        const config = await invoke('get_config_capture');
        this.netDevice = config.device_name;
        this.bufferSize = config.buffer_size;
        this.chan_capacity = config.chan_capacity;
        this.timeout = config.timeout;
        this.snaplen = config.snaplen;
        this.configStore.updateConfig(config);
      } catch (err) {
        console.error("[ConfigPanel] erreur get_config_capture :", err);
      }
    },

    async save() {
      if (!this.netDevice) return;
      try {
        const config = await invoke('config_capture', {
          device_name: this.netDevice.name,
          buffer_size: this.bufferSize,
          chan_capacity: this.chan_capacity,
          timeout: this.timeout,
          snaplen: this.snaplen,
        });
        if (typeof config.device_name === 'string') {
          this.netDevice = this.netInterfaces.find(i => i.name === config.device_name) || null;
        } else {
          this.netDevice = config.device_name;
        }
        this.bufferSize = config.buffer_size;
        this.chan_capacity = config.chan_capacity;
        this.timeout = config.timeout;
        this.snaplen = config.snaplen;
        this.configStore.updateConfig(config);
      } catch (err) {
        console.error("[ConfigPanel] erreur config_capture :", err);
      }
      this.close();
    },

    close() {
      this.$emit('update:ConfigPanel-visible', false);
    }
  },

  async mounted() {
    this.getConfig();
    invoke('get_devices_list').then(interfaces => {
      this.netInterfaces = interfaces;
    }).catch(err => console.error("Failed to load interfaces:", err));
  },

  watch: {
    netInterfaces(ifaces) {
      if (ifaces.length > 0 && typeof this.netDevice === 'string') {
        this.netDevice = ifaces.find(i => i.name === this.netDevice) || null;
      }
    }
  }
};
</script>

<style scoped>
.panel-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}

.panel {
  width: 340px;
  height: 100%;
  background: #363648;
  border-left: 1px solid #4a4a60;
  display: flex;
  flex-direction: column;
  animation: slide-in 0.22s cubic-bezier(0.25, 1, 0.5, 1);
}

@keyframes slide-in {
  from { transform: translateX(18px); opacity: 0; }
  to   { transform: translateX(0);     opacity: 1; }
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 14px 0 16px;
  border-bottom: 1px solid #4a4a60;
  flex-shrink: 0;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 500;
  color: #a8a8be;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}
.panel-title svg {
  width: 14px;
  height: 14px;
  color: #707090;
}

.close-btn {
  width: 26px;
  height: 26px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: #686884;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s;
}
.close-btn svg { width: 12px; height: 12px; }
.close-btn:hover { background: #484860; color: #b0b0c8; }

/* Body */
.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 18px 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.divider {
  height: 1px;
  background: #4a4a60;
  margin: 2px 0;
}

/* Fields */
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 11px;
  font-weight: 500;
  color: #808096;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.field-input {
  width: 100%;
  background: #2c2c3a;
  border: 1px solid #484860;
  border-radius: 4px;
  color: #b0b0c8;
  font-size: 13px;
  padding: 7px 10px;
  box-sizing: border-box;
  transition: border-color 0.15s, color 0.15s;
  appearance: none;
  -moz-appearance: textfield;
}
.field-input:focus {
  outline: none;
  border-color: #5a6a80;
  color: #d0d0e0;
  box-shadow: 0 0 0 2px rgba(90, 106, 128, 0.18);
}
.field-input::-webkit-outer-spin-button,
.field-input::-webkit-inner-spin-button { -webkit-appearance: none; }

/* Footer */
.panel-footer {
  display: flex;
  gap: 8px;
  padding: 14px 16px;
  border-top: 1px solid #4a4a60;
  flex-shrink: 0;
}

.btn-ghost {
  flex: 1;
  padding: 8px 14px;
  background: transparent;
  border: 1px solid #484860;
  border-radius: 5px;
  color: #808096;
  font-size: 12px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-ghost:active { transform: scale(0.97); }
.btn-ghost:hover { border-color: #60607a; color: #a8a8be; }

.btn-primary {
  flex: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 14px;
  background: #304868;
  border: 1px solid #3e5c80;
  border-radius: 5px;
  color: #88b4cc;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.1s ease;
}
.btn-primary:active { transform: scale(0.97); }
.btn-primary svg { width: 13px; height: 13px; }
.btn-primary:hover { background: #3a5878; border-color: #4a6888; color: #aaced0; }
</style>
