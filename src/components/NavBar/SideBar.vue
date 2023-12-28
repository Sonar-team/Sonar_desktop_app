<template>
    <div class="sidebar">        
        <p>Heure de départ: {{ heureDepart }}</p>
        <p>Heure de fin: {{ heureFin }}</p>
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
      tempsReleve: '',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName:'',
      heureDepart:'',
      heureFin:'',
    };
  },
  methods: {
    formatTime(date) {
      const hours = this.padZero(date.getHours());
      const minutes = this.padZero(date.getMinutes());
      const seconds = this.padZero(date.getSeconds());
      return `${hours}:${minutes}:${seconds}`;
    },

    calculateEndTime() {
      if (!this.heureDepart) {
        console.warn("heureDepart is empty. Skipping calculation of endTime.");
        return;
      }

      try {
        const startTime = new Date(this.heureDepart);
        if (isNaN(startTime.getTime())) {
          throw new Error('Invalid start time');
        }

        const [hours, minutes, seconds] = this.tempsReleve.split(':').map(Number);
        const durationInSeconds = hours * 3600 + minutes * 60 + seconds;
        const endTime = new Date(startTime.getTime() + durationInSeconds * 1000);

        // Format heureDepart and heureFin
        this.heureDepart = this.formatTime(startTime);
        this.heureFin = this.formatTime(endTime);
      } catch (error) {
        console.error("Error in calculateEndTime:", error);
      }
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
    } else if (minutes > 0) {
      minutes--;
      seconds = 59;
    } else if (hours > 0) {
      hours--;
      minutes = 59;
      seconds = 59;
    } else {
      // Le temps est écoulé, arrêter le timer ici si nécessaire
    }

    this.tempsReleve = `${this.padZero(hours)}:${this.padZero(minutes)}:${this.padZero(seconds)}`;
  }, 1000); // Mise à jour chaque seconde (1000 millisecondes)
},

  },
  mounted() {
    console.log("analyse mounted");

    this.heureDepart = this.$route.params.currentTime;
    this.tempsReleve = this.$route.params.time;
    this.calculateEndTime(); // Calculate the end time when the component is mounted
    
    this.$bus.on('increment-event', this.incrementTramesRecues);
    this.$bus.on('update-packet-count', this.incrementMatriceCount);
    this.updateTempsReleve();

    this.netInterface = this.$route.params.netInterface;
    this.installationName = this.$route.params.installationName;
    this.niveauConfidentialite = this.$route.params.confidentialite;
  },
  beforeUnmount() {
    this.$bus.off('update-packet-count', this.incrementMatriceCount);
    this.$bus.off('increment-event', this.incrementTramesRecues);
  },
};
  </script>

<style scoped>
.sidebar-logo {
  width: 100%; /* Adjust the width as necessary */
  max-width: 128px; /* Adjust the max width as necessary */
  height: auto; /* Maintain aspect ratio */
  margin-bottom: 20px; /* Add some space below the logo */
}

.sidebar {
  width: 300px; /* Largeur ajustée */
  background-color: #0b1118; /* Couleur de fond */
  color: #ECF0F1; /* Couleur du texte */
  padding: 20px;
  border-radius: 5px; /* Bordures arrondies */
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* Ombre */
  display: flex;
  flex-direction: column; /* Organisation verticale */
  gap: 10px; /* Espacement entre les éléments */
}

.sidebar p {
  margin: 0;
  padding: 5px 0;
}

.sidebar button {
  padding: 10px 15px;
  background-color: #183244; /* Couleur du bouton */
  color: white;
  border: none;
  border-radius: 3px;
  cursor: pointer;
  transition: background-color 0.3s ease; /* Transition pour le survol */
}

.sidebar button:hover {
  background-color: #0b1b25; /* Couleur au survol */
}
</style>
