<template>
    <div class="sidebar">    
        <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">   
        <p>Heure de départ: {{ heureDepart }}</p>
        <p>Heure de fin: {{ heureFin }}</p>
        <p>Temps restant: {{ tempsReleve }}</p>
        <p>Trames reçues: {{ tramesRecues }} </p>
        <p>Matrice de flux: {{ tramesEnregistrees }}</p>
        <p>Niveau de confidentialité: {{ niveauConfidentialite }}</p>
        <button @click="SaveToSelction">Sauvegarder</button>
     
    </div>
  </template>
  
  <script>
import { save } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api'
import { desktopDir } from '@tauri-apps/api/path';
import { message } from '@tauri-apps/api/dialog';

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
    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${this.padZero(now.getDate())}-${this.padZero(now.getMonth() + 1)}-${now.getFullYear()}`;
      return formattedDate;
    },
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
    async SaveToSelction() {
      console.log("stop and save")
      save({
        filters: [{
          name: 'Relevée CSV',
          extensions: ['csv']
        }],
        title: 'Sauvegarder la matrice de flux',
        defaultPath: this.getCurrentDate()+ '_' + this.niveauConfidentialite  + '_' + this.installationName + '.csv' // Set the default file name here
      }).then((response) => 
        invoke('save_packets_to_csv', { file_path: response })
          .then((response) => 
            console.log("save error: ",response))
            )
    },
    async SaveToDesktop() {
      console.log("save to desktop")
      const dir = await this.getDesktopDirPath();
      const dirPath = dir + this.getCurrentDate()+ '_' + this.niveauConfidentialite  + '_' + this.installationName + '.csv';
      if (dirPath) {
        invoke('save_packets_to_csv', { file_path: dirPath })
      } else {
        console.error("Failed to get desktop directory path");
      }
    },
    async getDesktopDirPath() {
    try {
      const dir = await desktopDir();
      console.log("App Data Directory: ", dir);
      return dir;
    } catch (error) {
      console.error("Error getting app data directory: ", error);
    }
  },

    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
    },

    updateTempsReleve() {
      // Fonction pour mettre à jour tempsReleve toutes les secondes
      setInterval(async () => {
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
              // Time is up, stop the timer here if necessary
              this.SaveToDesktop(); // Call the SaveToSelection method
              await message('Sauvegarde automatique sur le Bureau',
               { 
                title: 'Relevée terminée',
                type: 'info' 
              });

        }

        this.tempsReleve = `${this.padZero(hours)}:${this.padZero(minutes)}:${this.padZero(seconds)}`;
      }, 1000); // Mise à jour chaque seconde (1000 millisecondes)
    },

  },
  mounted() {
    console.log("analyse mounted");
    this.getDesktopDirPath();

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
  background-color: #2A2A2A; /* Couleur de fond */
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
  background-color: #11212c; /* Couleur du bouton */
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
