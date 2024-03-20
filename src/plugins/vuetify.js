// src/plugins/vuetify.js
import { createVuetify } from 'vuetify';
import 'vuetify/styles';
import * as components from 'vuetify/components';

const vuetify = createVuetify({
    components,
    theme: {
        defaultTheme: 'dark' // This sets the default theme to dark.
      }
  // Configuration de Vuetify (thèmes, icônes, etc.)
});

export default vuetify;
console.log('Vuetify is being set up.');