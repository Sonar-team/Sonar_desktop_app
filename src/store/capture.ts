// src/stores/capture.ts

import { defineStore } from "pinia";

export type CaptureConfig = {
  device_name: string;
  buffer_size: number;
  timeout: number;
};
export const useCaptureStore = defineStore("capture", {
  state: () => ({
    isRunning: false,
  }),
  actions: {
    updateStatus(status: { is_running: boolean }) {
      this.isRunning = status.is_running;
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
