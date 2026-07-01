// src/main.js
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { createPinia } from "pinia";

const busListeners = new Map<string, Map<(data: any) => void, EventListener>>();

const eventBus = {
  emit(event: string, data: any) {
    document.dispatchEvent(new CustomEvent(event, { detail: data }));
  },
  on(event: string, callback: (arg0: any) => any) {
    const wrapped: EventListener = (e) => callback((e as CustomEvent).detail);
    let listeners = busListeners.get(event);
    if (!listeners) {
      listeners = new Map();
      busListeners.set(event, listeners);
    }
    listeners.set(callback, wrapped);
    document.addEventListener(event, wrapped);
  },
  off(event: string, callback?: (arg0: any) => any) {
    const listeners = busListeners.get(event);
    if (!listeners) return;

    if (!callback) {
      for (const wrapped of listeners.values()) {
        document.removeEventListener(event, wrapped);
      }
      busListeners.delete(event);
      return;
    }

    const wrapped = listeners.get(callback);
    if (!wrapped) return;

    document.removeEventListener(event, wrapped);
    listeners.delete(callback);
    if (listeners.size === 0) busListeners.delete(event);
  },
};

const pinia = createPinia();
const app = createApp(App);
app.config.globalProperties.$bus = eventBus;
app.use(router);
app.use(pinia);
app.mount("#app");
