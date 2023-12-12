<template>
  <div >
    <div >
      <button @click="goToNextPage">Vue graphique</button>
      <button @click="stopAndSave">Stop</button>
    </div>
    <div >
      <Matrice />
      <BottomLong />
      <div class="additional-info">
        <p>Temps de relevé: 00:00:00</p>
        <p>Trames reçues: 100 / 1000</p>
        <p>Niveau de confidentialité: DR</p>
      </div>
    </div>
  </div>
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';


import { invoke } from '@tauri-apps/api'
import { save } from '@tauri-apps/api/dialog';

export default {
  data() {
    return {
      // Initialisation des données
      tempsReleve: '00:00:00', // Exemple de format, à ajuster selon vos données
      tramesRecues: 1000,
      tramesEnregistrees: 100,
      niveauConfidentialite: 'DR', // Exemple, à ajuster selon vos besoins
    };
  },
  components: {
    BottomLong,
    Matrice,
  },

  methods: {
    async handleClick() {
      //console.log(`You clicked on interface: ${netInterface}`);
      goToNextPage();
    },
    goToNextPage() {
      this.$router.push("/graph");
    },
    async stopAndSave() {
 
      console.log("stop and save")
      save({
        filters: [{
          name: 'Image',
          extensions: ['csv']
        }]
      }).then((response) => 
        invoke('stop_and_save', { file_path: response })
          .then((response) => 
            console.log(response))
            )
    }
  },
  mounted() {
    console.log("analyse mounted");
  }
};
</script>

<style>

</style>

