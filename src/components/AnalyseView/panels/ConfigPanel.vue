<!--
  Panneau de configuration de la capture : interface, taille de buffer,
  capacité de canal, timeout et snaplen, persistés côté backend
  (commande config_capture).
-->
<template>
  <div
    class="config-panel"
    role="dialog"
    aria-modal="true"
    aria-label="Configuration Capture"
    ref="panel"
    tabindex="-1"
    @keydown.esc="close"
  >
    <h2>Configuration Capture</h2>

    <InterfaceSelector
      v-model="selectedInterface"
      :net-interfaces="netInterfaces"
      class="config-item"
    />

    <div class="config-item">
      <label for="config-buffer-size">
        Taille du buffer :
        <button
          type="button"
          class="help-tooltip"
          aria-label="Taille du buffer kernel pcap, en octets. Plus il est grand, plus SONAR absorbe les pics de trafic sans perte côté système."
          data-tooltip="Buffer kernel pcap, en octets. Plus il est grand, plus SONAR absorbe les pics de trafic sans perte côté système."
        >?</button>
      </label>
      <input id="config-buffer-size" type="number" v-model.number="bufferSize" min="65536" max="536870912" step="1024" />
    </div>

    <div class="config-item">
      <label for="config-chan-capacity">
        Nombre de buffers:
        <button
          type="button"
          class="help-tooltip"
          aria-label="Capacité du canal interne entre capture et traitement. Trop petit, l'application peut perdre des paquets sous charge; trop grand, elle consomme plus de mémoire."
          data-tooltip="Capacité du canal interne entre capture et traitement. Trop petit, l'application peut perdre des paquets sous charge; trop grand, elle consomme plus de mémoire."
        >?</button>
      </label>
      <input id="config-chan-capacity" type="number" v-model.number="chanCapacity" min="1" max="1000000" />
    </div>

    <div class="config-item">
      <label for="config-timeout">
        Timeout (ms) :
        <button
          type="button"
          class="help-tooltip"
          aria-label="Délai pcap avant livraison des paquets. Petit, il réduit la latence; plus grand, il favorise le batching."
          data-tooltip="Délai pcap avant livraison des paquets. Petit, il réduit la latence; plus grand, il favorise le batching."
        >?</button>
      </label>
      <input id="config-timeout" type="number" v-model.number="timeout" min="1" max="10000" />
    </div>

    <div class="config-item">
      <label for="config-snaplen">
        Taille du snaplen :
        <button
          type="button"
          class="help-tooltip"
          aria-label="Nombre maximum d'octets capturés par paquet. Grand, il garde les paquets complets; petit, il réduit CPU et mémoire mais peut tronquer le décodage."
          data-tooltip="Nombre maximum d'octets capturés par paquet. Grand, il garde les paquets complets; petit, il réduit CPU et mémoire mais peut tronquer le décodage."
        >?</button>
      </label>
      <input id="config-snaplen" type="number" v-model.number="snaplen" min="64" max="262144" />
    </div>

    <div class="actions">
      <button type="button" @click="save" :disabled="!selectedInterface">Sauvegarder</button>
      <button type="button" @click="close">Fermer</button>
    </div>
  </div>
</template>

<script lang="ts">
import { info } from "@tauri-apps/plugin-log";
import { invoke } from "@tauri-apps/api/core";
import { defineComponent } from "vue";
import { displayCaptureError } from "../../../errors/capture";
import { useCaptureConfigStore } from "../../../store/capture";
import { DEFAULT_CAPTURE_CONFIG, type CaptureConfig } from "../../../types/config";
import type { NetDevice } from "../../../types/NetDevice";
import InterfaceSelector from "./CustomSelector/interfaceSelector.vue";

type ConfigPanelState = {
  netInterfaces: NetDevice[];
  selectedInterface: NetDevice | null;
  deviceName: string;
  bufferSize: number;
  chanCapacity: number;
  timeout: number;
  snaplen: number;
};

type CaptureNumericConfig = Omit<CaptureConfig, "device_name">;

function normalizeCaptureConfig(
  selectedInterface: NetDevice | null,
  values: CaptureNumericConfig,
): CaptureConfig | null {
  const deviceName = selectedInterface?.name.trim();
  if (!deviceName) return null;

  return {
    device_name: deviceName,
    buffer_size: Number(values.buffer_size),
    chan_capacity: Number(values.chan_capacity),
    timeout: Number(values.timeout),
    snaplen: Number(values.snaplen),
  };
}

export default defineComponent({
  name: "ConfigPanel",
  components: { InterfaceSelector },
  emits: ["update:ConfigPanel-visible"],

  data(): ConfigPanelState {
    return {
      netInterfaces: [] as NetDevice[],
      selectedInterface: null as NetDevice | null,
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
    syncSelectedInterface(): void {
      if (!this.deviceName || !this.netInterfaces.length) return;
      this.selectedInterface = this.netInterfaces.find((iface) => iface.name === this.deviceName) || null;
    },

    async getConfig(): Promise<void> {
      try {
        const config = await invoke<CaptureConfig>("get_config_capture");
        info(`[ConfigPanel] config=${JSON.stringify(config)}`);

        this.deviceName = config.device_name;
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

    async loadInterfaces(): Promise<void> {
      try {
        this.netInterfaces = await invoke<NetDevice[]>("get_devices_list");
        this.syncSelectedInterface();
      } catch (err) {
        await displayCaptureError(err);
      }
    },

    async save(): Promise<void> {
      const payload = normalizeCaptureConfig(this.selectedInterface, {
        buffer_size: this.bufferSize,
        chan_capacity: this.chanCapacity,
        timeout: this.timeout,
        snaplen: this.snaplen,
      });

      if (!payload) {
        await displayCaptureError({
          kind: "capture",
          message: { kind: "invalidConfig", message: "interface réseau obligatoire" },
        });
        return;
      }

      try {
        const config = await invoke<CaptureConfig>("config_capture", payload);

        this.deviceName = config.device_name;
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

    close(): void {
      this.$emit("update:ConfigPanel-visible", false);
    },
  },

  mounted() {
    this.loadInterfaces();
    this.getConfig();
    (this.$refs.panel as HTMLElement | undefined)?.focus();
  },

  watch: {
    selectedInterface(nextInterface: NetDevice | null) {
      this.deviceName = nextInterface?.name || "";
    },
    netInterfaces() {
      this.syncSelectedInterface();
    },
  },
});
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
  display: flex;
  align-items: center;
  gap: 6px;
  font-weight: bold;
  width: 100%;
  text-align: left;
}

.help-tooltip {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  border: 1px solid #8a8a8a;
  border-radius: 50%;
  color: #ffffff;
  background-color: #303744;
  font-family: inherit;
  font-size: 12px;
  line-height: 1;
  cursor: help;
  flex: 0 0 auto;
}

.help-tooltip::after {
  content: attr(data-tooltip);
  position: absolute;
  left: 24px;
  top: 50%;
  transform: translateY(-50%);
  z-index: 1001;
  width: min(280px, 70vw);
  padding: 8px 10px;
  border: 1px solid #555;
  border-radius: 6px;
  background-color: #111827;
  color: #ffffff;
  font-size: 12px;
  font-weight: 400;
  line-height: 1.35;
  opacity: 0;
  pointer-events: none;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.35);
  transition: opacity 0.15s ease;
}

.help-tooltip:hover::after,
.help-tooltip:focus::after {
  opacity: 1;
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
  background-image: url("../../../assets/select-chevron.svg");
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
  /* #007bff était sous le seuil WCAG AA avec du texte blanc (3.98:1) ;
     même bleu, légèrement assombri pour passer 4.5:1. */
  background-color: #0071eb;
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
