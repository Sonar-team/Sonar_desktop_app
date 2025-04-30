<template>
    <div class="sidebar">    
      <button class="image-btn">
        <img src="../../assets/images/128x128@2x.png" alt="Sonar Logo" >   
      </button>
      <button class="image-btn" 
        @click="toggleComponent">
        <img src="../../assets/images/graph.png" alt="Quitter" >

      </button> <!-- Toggle Button -->

      <button @click="quit" class="image-btn">
        <img src="../../assets/images/quit.png" alt="Quitter" >
        
      </button>
      <button @click="reset" class="image-btn">
        <img src="../../assets/images/reset.png" alt="Réinitialiser" >
      </button>
      <button @click="triggerSave" class="image-btn">
        <img src="../../assets/images/save.png" alt="Sauvegarder" >
        
      </button>
      <button @click="return_to_home" class="image-btn">
        <img src="../../assets/images/return.png" alt="Retour" >
        
      </button>

  </div>
</template>
  
<script>
import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core'
import { desktopDir } from '@tauri-apps/api/path';
import { exit } from '@tauri-apps/plugin-process';
import { info, error } from '@tauri-apps/plugin-log';


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
      info("trigger save")
      this.SaveAsCsv();
      this.SaveAsXlsx();
      
    },
    async SaveAsCsv() {
      info("Save as csv")
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
            error("save error: ",response))
            )
    },
    async SaveAsXlsx() {
      try {
        info("Début de la sauvegarde en xlsx");
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
          info("Sauvegarde terminée:", saveResponse);
          return saveResponse; // Retourner la réponse pour confirmer que c'est terminé
        } else {
          info("Aucun chemin de fichier sélectionné");
          throw new Error("Sauvegarde annulée ou chemin non sélectionné");
        }
      } catch (error) {
        error("Erreur lors de la sauvegarde en xlsx:", error);
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
        info("close resquested")
        await exit(1); // Appeler exit après la sauvegarde
      } catch (error) {
        error("Erreur lors de la sauvegarde ou de la fermeture de l'application:", error);
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
      info("current date: ",formattedDate)
      return formattedDate;
    },
    padZero(value) {
      // Fonction pour ajouter un zéro en cas de chiffre unique (par exemple, 5 -> 05)
      return value < 10 ? `0${value}` : value;
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
  position: relative; /* ou juste supprime la ligne */
  top: unset;
  left: unset;
  width: 100%;
  background-color: #2A2A2A;
  color: #ECF0F1;
  padding: 10px;
  display: flex;
  flex-direction: row;
  gap: 5px;
  z-index: 9999;
}

.image-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
}

.image-btn img {
  width: 50px;
  height: 50px;
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





</style>
