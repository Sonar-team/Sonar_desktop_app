<template>
    <div class="sidebar">    
      <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" width="150" height="150">   
      <button class="button" 
        @click="toggleComponent">
        {{ buttonText }}
      </button> <!-- Toggle Button -->

      <button @click="quit">
        Quitter
      </button>
      <button @click="reset">
        Réinitialiser
      </button>
      <button @click="triggerSave">
        Sauvegarder
      </button>
      <button @click="return_to_home">
        Retour
      </button>

  </div>
</template>
  
<script>
import { save, message } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core'
import { desktopDir } from '@tauri-apps/api/path';
import { exit, relaunch } from '@tauri-apps/plugin-process';

export default {
  data() {
    return {
      selectedFormat: 'xlsx',

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
      this.SaveAsCsv();
      this.SaveAsXlsx();
      
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
      this.tramesRecues = 0
      invoke('reset')    
    },
    async return_to_home() {
      this.tramesRecues = 0
      invoke('reset')    
    },


getCurrentDate() {
      // Fonction pour obtenir la date actuelle
      const now = new Date();
      // Formattez la date en DD/MM/YYYY
      const formattedDate = `${now.getFullYear()}${this.padZero(now.getMonth() + 1)}${this.padZero(now.getDate())}`;
      return formattedDate;
    },

  },
  mounted() {
    console.log("analyse mounted");
    this.getDesktopDirPath();

    this.netInterface = this.$route.params.netInterface;
    this.installationName = this.$route.params.installationName;
    this.niveauConfidentialite = this.$route.params.confidentialite;
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
  flex-direction:row; /* Organisation verticale */
  gap: 10px; /* Espacement entre les éléments */
}


.sidebar button {
  padding: 0px 15px;
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



</style>
