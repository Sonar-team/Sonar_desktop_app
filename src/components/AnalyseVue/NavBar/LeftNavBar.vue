<template>
  <div class="container">
    <div class="sidebar">
      <p>Temps restant: {{ tempsReleve }}</p>
      <p>Trames reçues: {{ tramesRecues }} / {{ tramesEnregistrees }}</p>
      <p>Niveau de confidentialité: {{ niveauConfidentialite }}</p>
      <button @click="stopAndSave">Stop</button>
    </div>
  </div>
</template>
  
<script>
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
  },
  mounted() {
    console.log("Leftnvabar mounted");
    this.updateTempsReleve();

    this.netInterface = this.$route.params.netInterface;
    this.installationName = this.$route.params.installationName;
    this.tempsReleve = this.$route.params.time;
    this.niveauConfidentialite = this.$route.params.confidentialite;
  }
}
</script>

  
  <style scoped>
    .container {
      display: flex; /* Utilise flexbox pour aligner les enfants côte à côte */
      height: 100vh; /* Remplit toute la hauteur de la fenêtre */
    }
  
    .sidebar {
      width: 20%; /* Largeur de la barre latérale */
      background-color: #444444;
      padding: 10px;
      color: aliceblue;
      width: 200px;
    }
    .nav-link {
      display: block;
      margin: 10px 0;
      color: aliceblue;
      text-decoration: none;
    }
    .nav-link:hover {
      text-decoration: underline;
    }
  
    .content {
      flex-grow: 1; /* Prend le reste de l'espace disponible */
      padding: 20px;
      overflow: auto; /* Ajoute un défilement si le contenu dépasse la hauteur de la fenêtre */
    }
  </style>
  