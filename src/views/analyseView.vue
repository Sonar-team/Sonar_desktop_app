<template>
  <div class="container">
    <div class="sidebar">
      <p>Temps de relevé: {{ tempsReleve }}</p>
      <p>Trames reçues: {{ tramesRecues }} / {{ tramesEnregistrees }}</p>
      <p>Niveau de confidentialité: {{ niveauConfidentialite }}</p>
    </div>
    <div class="content">
      <button @click="goToNextPage">Vue graphique</button>
      <button>Stop</button>
      <Matrice @incrementedMat="incrementMatriceCount"/>
      <BottomLong @incremented="incrementTramesRecues" />
    </div>
  </div>
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';

export default {
  data() {
    return {
      tempsReleve: '00:00:00',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: 'DR'
    };
  },
  components: {
    BottomLong,
    Matrice,
  },
  methods: {
    goToNextPage() {
      this.$router.push("/graph");
    },
    incrementTramesRecues() {
      this.tramesRecues++;
    },
    incrementMatriceCount() {
      this.tramesEnregistrees++;
    }
  },
  mounted() {
    console.log("analyse mounted");
  }
};
</script>

<style>
.container {
  display: flex;
}

.sidebar {
  width: 200px;
  background-color: #f1f1f1;
  padding: 20px;
}

.content {
  flex-grow: 1;
  padding: 20px;
}
</style>
