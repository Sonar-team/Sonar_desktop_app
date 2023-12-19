<template>
  <div class="container">
    <div class="sidebar">
      <p>Temps restant: {{ tempsReleve }}</p>
      <p>Trames reçues: {{ tramesRecues }} / {{ tramesEnregistrees }}</p>
      <p>Niveau de confidentialité: {{ niveauConfidentialite }}</p>
      <button @click="stopAndSave">Stop</button>
      <button @click="goToNextPage">Vue graphique</button>
    </div>
    <div class="content">
      <h1 class="titre-relevé">{{ getCurrentDate()+ '_' + niveauConfidentialite  + '_' + installationName }}</h1>
        <h2 class="titre">Matrice de flux</h2>
          <Matrice @incrementedMat="incrementMatriceCount"/>
        <h2 class="titre">Trames sniffées</h2>
          <BottomLong @incremented="incrementTramesRecues" />
    </div>
  </div>
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';

import { save } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api'

export default {
  data() {
    return {
      tempsReleve: '01:00:00',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName:''
    };
  },
  components: {
    BottomLong,
    Matrice
  },
  methods: {
    goToNextPage() {
      this.$router.push("/graph");
    },
    incrementTramesRecues() {
      this.tramesRecues++;
    },
    incrementMatriceCount() {
      this.tramesEnregistrees++;
    },
    async stopAndSave() {
      console.log("stop and save")
      save({
        filters: [{
          name: 'Image',
          extensions: ['csv']
        }]
      }).then((response) => 
        invoke('save_packets_to_csv', { file_path: response })
          .then((response) => 
            console.log("save error: ",response))
            )
    },

    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${this.padZero(now.getDate())}/${this.padZero(now.getMonth() + 1)}/${now.getFullYear()}`;
      return formattedDate;
    },

    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
    },

    updateTempsReleve() {
      // Fonction pour mettre à jour tempsReleve toutes les secondes
      setInterval(() => {
        const timeParts = this.tempsReleve.split(':');
        let hours = parseInt(timeParts[0]);
        let minutes = parseInt(timeParts[1]);
        let seconds = parseInt(timeParts[2]);

        if (seconds > 0) {
          seconds--;
        } else {
          if (minutes > 0) {
            minutes--;
            seconds = 59;
          } else {
            if (hours > 0) {
              hours--;
              minutes = 59;
              seconds = 59;
            } else {
              // Le temps est écoulé, arrêter le timer ici si nécessaire
            }
          }
        }
        this.tempsReleve = `${this.padZero(hours)}:${this.padZero(minutes)}:${this.padZero(seconds)}`;
      }, 1000); // Mise à jour chaque seconde (1000 millisecondes)
    },
  },
  mounted() {
    console.log("analyse mounted");
    this.updateTempsReleve();

    this.netInterface = this.$route.params.netInterface;
    this.installationName = this.$route.params.installationName;
    this.tempsReleve = this.$route.params.time;
    this.niveauConfidentialite = this.$route.params.confidentialite;
  }
};
</script>

<style scoped>
.container {
  display: flex;
  height: 100vh; /* Remplit toute la hauteur de la fenêtre */
}
.sidebar {
  width: 20%; /* Largeur de la barre latérale */
  background-color: #444444;
  padding: 20px;
  color: aliceblue;
}
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

.content {
  flex-grow: 1; /* Prend le reste de l'espace disponible */
  padding: 20px;
  overflow: auto; /* Ajoute un défilement si le contenu dépasse la hauteur de la fenêtre */
}

.titre-relevé,
.titre {
  text-align: center;
  color: aliceblue;
}

</style>
