<template>
    <div class="sidebar">
      <p>Temps restant: {{ tempsReleve }}</p>
      <p>Trames reçues: {{ tramesRecues }} </p>
      <p>Matrice de flux: {{ tramesEnregistrees }}</p>
      <p>Niveau de confidentialité: {{ niveauConfidentialite }}</p>
      <button @click="stopAndSave">Stop</button>
     
    </div>
  </template>
  
  <script>
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
  methods: {
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

    goToNextPage() {
      this.$router.push("/graph");
    },
    incrementTramesRecues() {
      this.tramesRecues++;
    },
    incrementMatriceCount(packetCount) {
      this.tramesEnregistrees = packetCount;
    },
    async stopAndSave() {
      console.log("stop and save")
      save({
        filters: [{
          name: 'Relevée',
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
    this.$bus.on('increment-event', this.incrementTramesRecues);
    this.$bus.on('update-packet-count', this.incrementMatriceCount);
    this.updateTempsReleve();

    this.netInterface = this.$route.params.netInterface;
    this.installationName = this.$route.params.installationName;
    this.tempsReleve = this.$route.params.time;
    this.niveauConfidentialite = this.$route.params.confidentialite;
  },
  beforeUnmount() {
    this.$bus.off('update-packet-count', this.incrementMatriceCount);
    this.$bus.off('increment-event', this.incrementTramesRecues);
  },
};
  </script>

  <style scoped>
.sidebar {
  width: 100px; /* Largeur de la barre latérale */
  background-color: #444444;
  padding: 20px;
  color: aliceblue;
}
</style>