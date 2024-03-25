<template>
    <div class="sidebar">    
      <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">   
      <p>Départ: {{ heureDepart }}</p>
      <p>Fin: {{ heureFin }}</p>
      <button class="button-up" @click="augmenterTemps"></button>
      <p>Temps restant: {{ tempsReleve }}</p>
      <button class="button-down" @click="diminuerTemps"></button>
      <p>Temps écoulé: {{ tempsEcoule }}</p>
      <p>Trames reçues: {{ tramesRecues }} </p>
      <p>Matrice de flux: {{ tramesEnregistrees }}</p>
      <p>Choix du format:</p>
      <select v-model="selectedFormat">
        <option value="csv">CSV</option>
        <option value="xlsx">Excel</option>
      </select>

    <button @click="SaveFile">Sauvegarder</button>
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
      selectedFormat: 'xlsx',
      tempsReleve: '',
      tempsEcoule: '',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName:'',
      heureDepart:'',
      heureFin:'',
    };
  },
  methods: {
    SaveFile() {
      if (this.selectedFormat === 'csv') {
        this.SaveAsCsv();
      } else if (this.selectedFormat === 'xlsx') {
        this.SaveAsXlsx();
      }
    },
    augmenterTemps() {
    this.ajusterTemps(1); // Augmenter d'une seconde
    },
    diminuerTemps() {
      this.ajusterTemps(-1); // Diminuer d'une seconde
    },
    ajusterTemps(ajustement) {
      let [heures, minutes, secondes] = this.tempsReleve.split(':').map(Number);
      const tempsTotalEnSecondes = heures * 3600 + minutes * 60 + secondes + ajustement;

      heures = Math.floor(tempsTotalEnSecondes / 3600);
      minutes = Math.floor((tempsTotalEnSecondes % 3600) / 60);
      secondes = tempsTotalEnSecondes % 60;

      this.tempsReleve = `${this.padZero(heures)}:${this.padZero(minutes)}:${this.padZero(secondes)}`;
    },
    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${now.getFullYear()}${this.padZero(now.getMonth() + 1)}${this.padZero(now.getDate())}`;
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
    async SaveAsCsv() {
      console.log("Save as csv")
      save({
        filters: [{
          name: 'Relevée CSV',
          extensions: ['csv']
        }],
        title: 'Sauvegarder la matrice de flux',
        defaultPath: this.getCurrentDate()+ '_' + this.niveauConfidentialite  + '_' + this.installationName+ '.csv' // Set the default file name here
      
      }).then((response) => 
        invoke('save_packets_to_csv', { file_path: response })
          .then((response) => 
            console.log("save error: ",response))
            )
    },
    async SaveAsXlsx() {
      console.log("Save as xlsx")
      save({
        filters: [{
          name: 'Relevée Excel',
          extensions: ['xlsx']
        }],
        title: 'Sauvegarder la matrice de flux',
        defaultPath: this.getCurrentDate()+ '_' + this.niveauConfidentialite  + '_' + this.installationName + '.xlsx'// Set the default file name here
      
      }).then((response) => 
        invoke('save_packets_to_excel', { file_path: response })
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

    updateTempsEcoule() {
  const startTime = new Date();
  
  const intervalId = setInterval(() => {
    const now = new Date();
    let elapsed = new Date(now - startTime);

    // Calcul du temps écoulé
    let hours = elapsed.getUTCHours();
    let minutes = elapsed.getUTCMinutes();
    let seconds = elapsed.getUTCSeconds();

    this.tempsEcoule = `${this.padZero(hours)}:${this.padZero(minutes)}:${this.padZero(seconds)}`;
  }, 1000);
},

    updateTempsReleve() {
  // Stocker l'identifiant de l'intervalle
  const intervalId = setInterval(async () => {
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
      // Temps écoulé, arrêter l'intervalle
      clearInterval(intervalId);

      // Appeler SaveToDesktop et attendre la réponse au dialogue
      this.SaveToDesktop();
      await message('Sauvegarde automatique sur le Bureau', { 
        title: 'Relevée terminée',
        type: 'info'
      });
      return; // Important pour sortir de la fonction
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
    this.updateTempsReleve();
    this.updateTempsEcoule(); // calculate
    this.$bus.on('increment-event', this.incrementTramesRecues);
    this.$bus.on('update-packet-count', this.incrementMatriceCount);
    

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
.sidebar {
  width: 190px; /* Largeur ajustée */
  background-color: #2A2A2A; /* Couleur de fond */
  color: #ECF0F1; /* Couleur du texte */
  padding: 20px;
  border-radius: 5px; /* Bordures arrondies */
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* Ombre */
  display: flex;
  flex-direction: column; /* Organisation verticale */
  gap: 10px; /* Espacement entre les éléments */
}

.sidebar img {
  width: 150px; /* Taille du logo */
  height: auto; /* Maintenir le ratio de l'image */
  margin-bottom: 20px; /* Espacement sous l'image */
}

.sidebar p {
  margin: 0;
  padding: 5px 0;
  font-size: 1.2em; /* Augmenter la taille de la police */
}

.sidebar button {
  padding: 10px 15px;
  background-color: #11212c; /* Couleur du bouton */
  color: white;
  border: none;
  border-radius: 3px;
  cursor: pointer;
  transition: background-color 0.3s ease; /* Transition pour le survol */
  font-size: 1.1em; /* Taille de la police du bouton */
}

.sidebar button:hover {
  background-color: #0b1b25; /* Couleur au survol */
}

/* Responsive Design pour les petits écrans */
@media (max-width: 768px) {
  .sidebar {
    width: 100%; /* Pleine largeur pour les petits écrans */
    box-shadow: none; /* Pas d'ombre pour un look plus simple */
  }
}

.button-up, .button-down {
  display: inline-block;
  background-color: #11212c; /* Couleur de fond du bouton */
  color: #ffffff; /* Couleur du texte du bouton */
  text-align: center;
  padding: 10px;
  margin: 5px;
  border: none;
  border-radius: 3px;
  transition: all 0.3s ease;
  position: relative; /* Position relative pour les pseudo-éléments */
}

/* Style pour le contenu du bouton (la flèche) */
.button-up::before, .button-down::before {
  content: '';
  display: block;
  margin: auto; /* Centre automatiquement la flèche */
  width: 0; 
  height: 0;
  border-style: solid;
}

/* Flèche vers le haut */
.button-up::before {
  border-width: 0 5px 8px 5px; /* Ajuste la taille de la flèche */
  border-color: transparent transparent #ffffff transparent; /* Flèche blanche */
}

/* Flèche vers le bas */
.button-down::before {
  border-width: 8px 5px 0 5px; /* Ajuste la taille de la flèche */
  border-color: #ffffff transparent transparent transparent; /* Flèche blanche */
}

/* Effet de survol pour les boutons */
.button-up:hover, .button-down:hover {
  background-color: #0b1b25; /* Couleur de survol plus foncée */
  cursor: pointer;
}

</style>
