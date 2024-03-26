// src/plugins/vuetify.js
import { createVuetify } from 'vuetify';
import 'vuetify/styles';
import * as components from 'vuetify/components';
import 'vuetify/dist/vuetify.min.css' // Ensure you are using css-loader

const vuetify = createVuetify({
    components,
  // Configuration de Vuetify (thèmes, icônes, etc.)
});

export default vuetify;
console.log('Vuetify is being set up.');