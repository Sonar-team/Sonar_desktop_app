// src/main.js
import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import vuetify from './plugins/vuetify'; // Assurez-vous que le chemin d'importation est correct

const eventBus = {
  emit(event, data) {
    document.dispatchEvent(new CustomEvent(event, { detail: data }));
  },
  on(event, callback) {
    document.addEventListener(event, (e) => callback(e.detail));
  },
  off(event, callback) {
    document.removeEventListener(event, callback);
  },
};

const app = createApp(App);
app.config.globalProperties.$bus = eventBus;
app.use(router);
app.use(vuetify); // Utilisez l'instance Vuetify ici
console.log('Mounting Vue app with Vuetify.');
app.mount('#app');
