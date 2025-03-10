<template>
  <div class="status-bar">
    <div class="status-content">
      <p>Début: {{ heureDepart }}</p>
      <p>Fin: {{ heureFin }}</p>
      <p>Temps restant: {{ tempsReleve }}</p>
      <p>Temps écoulé: {{ tempsEcoule }}</p>
      <p>Trames reçues: {{ tramesRecues }} </p>
      <p>Matrice de flux: {{ tramesEnregistrees }}</p>
    </div>
  </div>
</template>

<script>
  import { padZero } from '../../utils/time';

  export default {
    data() {
      return {
        tramesRecues: 0,
        tramesEnregistrees: 0,
        tempsReleve: '',
        heureDepart:'',
        };
    },
    methods: {
      incrementTramesRecues() {
        this.tramesRecues++;
      },
      incrementMatriceCount(packetCount) {
        // console.log("incrementMatriceCount", packetCount)
        this.tramesEnregistrees = packetCount;
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

          this.tempsEcoule = `${padZero(hours)}:${padZero(minutes)}:${padZero(seconds)}`;
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

        this.tempsReleve = `${padZero(hours)}:${padZero(minutes)}:${padZero(seconds)}`;
      }, 1000); // Mise à jour chaque seconde (1000 millisecondes)
      },
      calculateEndTime() {
        if (!this.heureDepart) {
          console.warn("heureDepart is empty. Skipping calculation of endTime.");
          return;
        }

        try {
          console.log("heureDepart initial value: ", this.heureDepart);

          let startTime;

          if (typeof this.heureDepart === 'string') {
            // Si heureDepart est sous la forme "HH:mm:ss", ajoutez la date actuelle
            if (this.heureDepart.match(/^\d{2}:\d{2}:\d{2}$/)) {
              const currentDate = new Date().toISOString().split('T')[0]; // Obtenez la date d'aujourd'hui (YYYY-MM-DD)
              this.heureDepart = `${currentDate}T${this.heureDepart}`; // Combinez la date et l'heure
            }
            startTime = new Date(this.heureDepart); // Convertir la chaîne en objet Date
          } else if (this.heureDepart instanceof Date) {
            // Si c'est déjà un objet Date, l'utiliser directement
            startTime = this.heureDepart;
          } else {
            throw new Error('Invalid start time format');
          }

          console.log("Parsed startTime: ", startTime);

          if (isNaN(startTime.getTime())) {
            throw new Error('Invalid start time');
          }

          const [hours, minutes, seconds] = this.tempsReleve.split(':').map(Number);
          const durationInSeconds = hours * 3600 + minutes * 60 + seconds;
          const endTime = new Date(startTime.getTime() + durationInSeconds * 1000);

          // Format heureDepart et heureFin
          this.heureDepart = this.formatTime(startTime);
          this.heureFin = this.formatTime(endTime);

          console.log("Calculated heureFin: ", this.heureFin);


        } catch (error) {
          console.error("Error in calculateEndTime:", error);
        }
      },
      ajusterTemps(ajustement) {
        let [heures, minutes, secondes] = this.tempsReleve.split(':').map(Number);
        let tempsTotalEnSecondes = heures * 3600 + minutes * 60 + secondes + ajustement;

        // S'assurer que le temps ne passe pas en dessous de 0
        if (tempsTotalEnSecondes < 0) {
          tempsTotalEnSecondes = 0;
        }

        heures = Math.floor(tempsTotalEnSecondes / 3600);
        minutes = Math.floor((tempsTotalEnSecondes % 3600) / 60);
        secondes = tempsTotalEnSecondes % 60;

        this.tempsReleve = `${padZero(heures)}:${padZero(minutes)}:${padZero(secondes)}`;
        this.calculateEndTime();
      },
      formatTime(date) {
        const hours = padZero(date.getHours());
        const minutes = padZero(date.getMinutes());
        const seconds = padZero(date.getSeconds());
        return `${hours}:${minutes}:${seconds}`;
      },
    },
    mounted() {
      this.$bus.on('increment-event', this.incrementTramesRecues);
      this.$bus.on('update-packet-count', this.incrementMatriceCount);

      this.heureDepart = this.$route.params.currentTime || new Date().toISOString();
      this.tempsReleve = this.$route.params.time || '00:00:00'; // Valeur par défaut si pas de temps relevé
      
      this.calculateEndTime(); // Calculer l'heure de fin lors du montage
      this.updateTempsReleve();
      this.updateTempsEcoule(); // Calculer le temps écoulé

    },
    beforeUnmount() {
      this.$bus.off('update-packet-count', this.incrementMatriceCount);
      this.$bus.off('increment-event', this.incrementTramesRecues);
    },
  }
</script>
  
<style scoped>
  .status-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    width: 100%;
    background-color: #0b1b25;
    color: white;
    text-align: center;
    padding: 4px;
    font-size: 12px;
  }
  
  .status-content {
  display: flex;
  justify-content: space-around; /* Répartit les éléments équitablement */
  align-items: center; /* Aligne les éléments verticalement */
  flex-wrap: wrap; /* Permet le passage à la ligne si la largeur est insuffisante */
}
</style>