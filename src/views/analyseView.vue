<template>
  <div class="analyse-container">
    <div class="button-container">
    <button @click="goToNextPage">Vue graphique</button>
    <button @click="stopAndSave">Stop</button>
  </div>
    <div class="top-section">
      <Matrice />
      <Stat />
      <!-- Ajout d'une nouvelle section pour les informations supplémentaires -->
      <div class="additional-info">
        <p>Temps de relevé: 00:00:00</p>
        <p>Trames reçues: 100 / 1000</p>
        <p>Niveau de confidentialité: DR</p>
      </div>
    </div>
    <BottomLong />
  </div>
  
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';
import Stat from '../components/CaptureVue/Stat.vue';

import { invoke } from '@tauri-apps/api'
import { save } from '@tauri-apps/api/dialog';

export default {
  data() {
    return {
      // Initialisation des données
      tempsReleve: '00:00:00', // Exemple de format, à ajuster selon vos données
      tramesRecues: 1000,
      tramesEnregistrees: 100,
      niveauConfidentialite: 'DR', // Exemple, à ajuster selon vos besoins
    };
  },
  components: {
    BottomLong,
    Matrice,
    Stat
  },

  methods: {
    async handleClick() {
      //console.log(`You clicked on interface: ${netInterface}`);
      goToNextPage();
    },
    goToNextPage() {
      this.$router.push("/graph");
    },
    async stopAndSave() {
 
      console.log("stop and save")
      save({
        filters: [{
          name: 'Image',
          extensions: ['csv']
        }]
      }).then((response) => 
        invoke('stop_and_save', { file_path: response })
          .then((response) => 
            console.log(response))
            )

      
    }
  },
  mounted() {
    console.log("analyse mounted");
  }
};
</script>

<style>
.analyse-container {
  display: flex;
  flex-direction: column;
  height: 100vh; /* Full height of the viewport */
  color: #333; /* Default text color for visibility */
}

.top-section {
  display: flex;
  flex-wrap: wrap; /* Wrap items when not enough space */
  justify-content: space-between;
  padding: 20px; /* Spacing around the inner content */
  border-bottom: 1px solid #ddd; /* Separator from the rest of the content */
}

.Matrice, .Stat {
  flex: 1; /* Each takes equal width */
  min-width: 300px; /* Minimum width so they don't get too narrow */
  margin: 10px; /* Spacing between components */
  border: 1px solid #ddd; /* Border for definition */
  border-radius: 4px; /* Slightly rounded corners */
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1); /* Subtle shadow for depth */
}

.BottomLong {
  flex-basis: 100%; /* Takes full width */
  padding: 20px; /* Spacing around the inner content */
  position: right ;
  border-top: 1px solid; /* Separates from the above content */
}

.button-container {
  display: flex;
  justify-content: flex-end; /* Aligns the button to the right */
  padding: 20px; /* Adds some space around the button */
}

.button-container button:hover {
  background-color: #0056b3; /* Darker shade on hover */
}

.additional-info {
  padding: 10px;
  border: 1px solid #ddd;
  margin-top: 10px;
  color: chocolate;
  border-radius: 4px; /* Consistency in design */
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1); /* As with Matrice and Stat */
}

/* Additional styles for the charts if needed */
.chart {
  padding: 15px;
  background: #ffffff;
  border-radius: 4px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Responsive design adjustments */
@media (max-width: 768px) {
  .top-section {
    flex-direction: column; /* Stack the components on smaller screens */
  }

  .Matrice, .Stat {
    flex-basis: 100%; /* Full width on smaller screens */
  }
}

</style>

