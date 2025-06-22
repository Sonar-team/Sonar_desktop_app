// src/main.js
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { createPinia } from "pinia";

const eventBus = {
  emit(event: string, data: any) {
    document.dispatchEvent(new CustomEvent(event, { detail: data }));
  },
  on(event: any, callback: (arg0: any) => any) {
    document.addEventListener(event, (e) => callback(e.detail));
  },
  off(event: any, callback: (this: Document, ev: any) => any) {
    document.removeEventListener(event, callback);
  },
};
const pinia = createPinia();
const app = createApp(App);
app.config.globalProperties.$bus = eventBus;
app.use(router);
app.use(pinia);
app.mount("#app");
