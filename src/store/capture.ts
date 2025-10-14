import { defineStore } from "pinia";
import { markRaw } from "vue";
import type { Channel } from "@tauri-apps/api/core";
import type { CaptureEvent, GraphUpdate } from "../types/capture";

// ⚠️ Channel hors réactivité pour éviter le proxy de Vue
let __channel: Channel<CaptureEvent> | undefined;

export const useCaptureStore = defineStore("capture", {
  state: () => ({
    isRunning: false,
    showMatrice: true,

    // Listeners HMR-safe dans le state
    packetListeners: [] as Array<(p: any) => void>,
    statsListeners: [] as Array<(d: any) => void>,
    lenFlowMatrixListeners: [] as Array<(d: any) => void>,
    channelCapacityPayloadListeners: [] as Array<(d: any) => void>,
    graphUpdateListeners: [] as Array<(u: GraphUpdate) => void>,
  }),

  actions: {
    updateStatus(status: { is_running: boolean }) {
      this.isRunning = status.is_running;
    },
    toggleView() {
      this.showMatrice = !this.showMatrice;
    },

    setChannel(channel: Channel<CaptureEvent>) {
      console.log("[CaptureStore] Channel attaché");

      // détacher l’ancien handler si besoin
      if (__channel) (__channel as any).onmessage = undefined;

      __channel = markRaw(channel);

      __channel.onmessage = (msg: any) => {
        // console.log("[CaptureStore] Message reçu :", msg.data)
        switch (msg.event) {
          case "packet":
            for (const cb of this.packetListeners) cb(msg.data.packet);
            break;
          case "stats":
            for (const cb of this.statsListeners) cb(msg.data);
            break;
          case "flowMatrixLen":
            for (const cb of this.lenFlowMatrixListeners) {
              cb(msg.data.flow_matrix_len);
            }
            break;
          case "channelCapacityPayload":
            for (const cb of this.channelCapacityPayloadListeners) cb(msg.data);
            break;
          case "graph": {
            const update = msg.data.update as GraphUpdate;
            for (const cb of this.graphUpdateListeners) cb(update);
            break;
          }
        }
      };
    },

    getChannel(): Channel<CaptureEvent> | undefined {
      return __channel;
    },

    onPacket(cb: (p: any) => void) {
      this.packetListeners.push(cb);
    },
    onStats(cb: (d: any) => void) {
      this.statsListeners.push(cb);
    },
    onFlowMatrixLen(cb: (d: any) => void) {
      this.lenFlowMatrixListeners.push(cb);
    },
    onChannelCapacityPayload(cb: (d: any) => void) {
      this.channelCapacityPayloadListeners.push(cb);
    },
    onGraphUpdate(cb: (u: GraphUpdate) => void) {
      console.log("[CaptureStore] GraphUpdate abonné");
      this.graphUpdateListeners.push(cb);
    },
  },
});

export const useCaptureConfigStore = defineStore("captureConfig", {
  state: () => ({
    interface: "",
    buffer_size: 18000000,
    timeout: 10000,
  }),
  actions: {
    updateConfig(
      config: { device_name: string; buffer_size: number; timeout: number },
    ) {
      this.interface = config.device_name;
      this.buffer_size = config.buffer_size;
      this.timeout = config.timeout;
    },
  },
});
