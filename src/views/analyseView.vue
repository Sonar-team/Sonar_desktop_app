<template>
  <div class="container">
    <div class="sidebar">
      <p>Temps de relevé: {{ tempsReleve }}</p>
      <p>Trames reçues: {{ tramesRecues }} / {{ tramesEnregistrees }}</p>
      <p>Niveau de confidentialité: {{ niveauConfidentialite }}</p>
    </div>
    <div class="content">
      <h1 class="titre-relevé">{{ getCurrentDate()+ '_' + niveauConfidentialite  + '_' + installationName }}</h1>
      <h2 class="titre">Matrice de flux</h2>
      <Matrice @incrementedMat="incrementMatriceCount"/>
      <h2 class="titre">Trames sniffées</h2>
      <BottomLong @incremented="incrementTramesRecues" />
      <button @click="goToNextPage">Vue graphique</button>
      <button>Stop</button>
    </div>
  </div>
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';

//okok

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
    Matrice,
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
    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
    },
    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${this.padZero(now.getDate())}/${this.padZero(now.getMonth() + 1)}/${now.getFullYear()}`;
      return formattedDate;
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

.titre-relevé,
.titre {
  color: aliceblue;
}

.sidebar {
  width: 20%; /* Largeur de la barre latérale */
  background-color: #444444;
  padding: 20px;
  color: aliceblue;
}

.content {
  flex-grow: 1; /* Prend le reste de l'espace disponible */
  padding: 20px;
  overflow: auto; /* Ajoute un défilement si le contenu dépasse la hauteur de la fenêtre */
}
</style>
