<template>
  <div class="container">
    <Topbar
      :netInterface="$route.params.netInterface"
      :confidentialite="$route.params.confidentialite"
      :installationName="$route.params.installationName"
      :time="$route.params.time"
      :currentTime="$route.params.currentTime"
    />
    <div class="content">
      <NetworkGraphComponent v-if="showMatrice" /> <!-- Show Matrice when showMatrice is true -->
      <Matrice v-else /> <!-- Show NetworkGraphComponent otherwise -->
    </div>

    <BottomLong  />
    <StatusBar 
      :time="$route.params.time"
      :currentTime="$route.params.currentTime"
      />
  </div>
</template>

<script>
import Topbar from '../components/NavBar/TopBar.vue';
import BottomLong from '../components/AnalyseView/BottomLong.vue';
import Matrice from '../components/AnalyseView/Matrice.vue';
import NetworkGraphComponent from '../components/AnalyseView/NetworkGraphComponent.vue'; // Import the other component
import StatusBar from '../components/NavBar/StatusBar.vue'; // Import du composant

import { invoke } from '@tauri-apps/api/core'

export default {
  data() {
    return {
      tempsReleve: '',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName:'',
      showMatrice: true // Toggle state (true for Matrice, false for NetworkGraphComponent)

    };
  },
  components: {
    BottomLong,
    Matrice,
    Topbar,
    NetworkGraphComponent,
    StatusBar,
    
  },
  methods: {
    toggleComponent() {
      this.showMatrice = !this.showMatrice; // Toggle the state
    },
    togglePause() {
      console.log("toggle pause")
      invoke('toggle_pause')
          .then((message) => {
              console.log("Réponse reçue de 'toggle_pause':", message);
              return message; // S'assure que le message est renvoyé pour une utilisation future
          })
          .catch((error) => {
              console.error("Erreur lors de l'invocation de 'toggle_pause':", error);
              throw error; // Permet de propager l'erreur pour une gestion plus avancée si nécessaire
          });
    },

  },
  mounted() {
    invoke('get_selected_interface', { interface_name: this.$route.params.netInterface })
    this.$bus.on('toggle',this.toggleComponent)
    this.installationName = this.$route.params.installationName;
    this.niveauConfidentialite = this.$route.params.confidentialite;
  },
  beforeMount() {
    this.$bus.off('toggle',this.toggleComponent)
  }
}
</script>

<style scoped>
.container {
  display: flex;
  flex-direction: column;
}

</style>
