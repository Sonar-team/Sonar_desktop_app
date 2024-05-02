<template>
  <div class="container">
    <Sidebar
      :netInterface="$route.params.netInterface"
      :confidentialite="$route.params.confidentialite"
      :installationName="$route.params.installationName"
      :time="$route.params.time"
      :currentTime="$route.params.currentTime"
    />
    <div class="content">
      <h3 class="titre">Matrice de flux : {{ getCurrentDate()+ '_' + niveauConfidentialite  + '_' + installationName }}</h3>
      <!--button @click="togglePause">pause</button-->
      <Matrice v-if="showMatrice" /> <!-- Show Matrice when showMatrice is true -->
      <NetworkGraphComponent v-else /> <!-- Show NetworkGraphComponent otherwise -->
      <BottomLong  />
        
    </div>
  </div>
</template>

<script>
import Sidebar from '../components/NavBar/SideBar.vue';
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';
import NetworkGraphComponent from '../components/AnalyseVue/GraphVue/NetworkGraphComponent.vue'; // Import the other component

import { invoke } from '@tauri-apps/api/tauri'

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
    Sidebar,
    NetworkGraphComponent
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

    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${now.getFullYear()}${this.padZero(now.getDate())}${this.padZero(now.getMonth() + 1)}`;
      return formattedDate;
    },

    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
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

}

.content {
  overflow:auto; /* Ajoute un défilement si le contenu dépasse la hauteur de la fenêtre */
  flex: 1; /* Allow content to expand and fill available space */

}

.titre {
  text-align: center;
  color: aliceblue;
  margin: 1px 0; /* Reduce top and bottom margin */

}

.button {
  background-color: #0b1b25; /* Couleur de fond du bouton */
  color: #fff; /* Couleur du texte du bouton */
  padding: 10px 20px; /* Espacement intérieur du bouton */
  border: none; /* Supprimer la bordure du bouton */
  border-radius: 5px; /* Ajouter une bordure arrondie au bouton */
  cursor: pointer; /* Curseur de type pointeur au survol */
}

</style>
