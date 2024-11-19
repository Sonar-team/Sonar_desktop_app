<template>
    <div class="sidebar">    
      <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">   
      <button class="button" @click="toggleComponent">{{ buttonText }}</button> <!-- Toggle Button -->
      <p>Trames reçues: {{ tramesRecues }} </p>
      <p>Matrice de flux: {{ tramesEnregistrees }}</p>
      
      <p>Exporter: </p>
      <select v-model="selectedFormat" 
        @change="triggerSave" 
        style="border: 2px solid #89CFF0; color: aliceblue">
        <option value="csv">Csv</option>
        <option value="xlsx">Excel</option>
      </select>

      <button @click="quit">
        Quitter
      </button>
      <button @click="reset">
        Réinitialiser
      </button>

  </div>
</template>
  
<script>
import { save } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api'
import { exit } from '@tauri-apps/api/process';

export default {
  data() {
    return {
      selectedFormat: 'xlsx',
      tramesRecues: 0,

      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName:'',
      showMatrice: true // Toggle state (true for Matrice, false for NetworkGraphComponent)
    };
  },
  computed: {
    buttonText() {
      // Change le texte du bouton en fonction de la vue actuellement affichée
      return this.showMatrice ? 'Graphique' : 'Matrice';
    }
  },
  methods: {
    toggleComponent() {
      this.$bus.emit('toggle')
      this.showMatrice = !this.showMatrice; // Toggle the state
    },
    triggerSave() {
      if (this.selectedFormat === 'csv') {
        this.SaveAsCsv();
      } else if (this.selectedFormat === 'xlsx') {
        this.SaveAsXlsx();
      }
    },
    incrementTramesRecues() {
      this.tramesRecues++;
    },
    incrementMatriceCount(packetCount) {
      // console.log("incrementMatriceCount", packetCount)
      this.tramesEnregistrees = packetCount;
    },
    async SaveAsCsv() {
      console.log("Save as csv")
      save({
        filters: [{
          name: '.csv',
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
      try {
        console.log("Début de la sauvegarde en xlsx");
        const response = await save({
          filters: [{
            name: '.xlsx',
            extensions: ['xlsx']
          }],
          title: 'Sauvegarder la matrice de flux',
          defaultPath: this.getCurrentDate() + '_' + this.niveauConfidentialite + '_' + this.installationName + '.xlsx'
        });

        if (response) {
          // Attendez que l'invocation d'API pour sauvegarder soit terminée
          const saveResponse = await invoke('save_packets_to_excel', { file_path: response });
          console.log("Sauvegarde terminée:", saveResponse);
          return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
        } else {
          console.log("Aucun chemin de fichier sélectionné");
          throw new Error("Sauvegarde annulée ou chemin non sélectionné");
        }
      } catch (error) {
        console.error("Erreur lors de la sauvegarde en xlsx:", error);
        throw error; // Relancer l'erreur pour la gestion dans quit()
      }
    },
    getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${now.getFullYear()}${this.padZero(now.getMonth() + 1)}${this.padZero(now.getDate())}`;
      return formattedDate;
    },
    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
    },
    async quit() {
      try {
        await this.SaveAsXlsx(); // Attendre que SaveAsXlsx soit terminé
        console.log('Sauvegarde terminée, fermeture de l\'application');
        await exit(1); // Appeler exit après la sauvegarde
      } catch (error) {
        console.error("Erreur lors de la sauvegarde ou de la fermeture de l'application:", error);
      }
    },

    async reset() {
      invoke('reset')    
    },

   

  },


  mounted() {
    console.log("analyse mounted");
   

    // Exemple d'initialisation de heureDepart au format ISO 8601 (YYYY-MM-DDTHH:mm:ss)


    this.$bus.on('increment-event', this.incrementTramesRecues);
    this.$bus.on('update-packet-count', this.incrementMatriceCount);
    

   
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
  padding-top: 32px;
  border-radius: 5px; /* Bordures arrondies */
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* Ombre */
  display: flex;
  flex-direction: column; /* Organisation verticale */
  gap: 10px; /* Espacement entre les éléments */
}

.sidebar img {
  width: 150px; /* Taille du logo */
  height: auto; /* Maintenir le ratio de l'image */
  margin-bottom: 10px; /* Espacement sous l'image */
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

.buttons {
  display: flex;
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
