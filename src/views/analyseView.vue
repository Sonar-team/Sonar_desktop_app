<template>
  <div class="analyse-container">
    <div class="top-section">
      <Matrice />
      <Stat />
    </div>
    <BottomLong />
  </div>
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';
import Stat from '../components/CaptureVue/Stat.vue';

export default {
  components: {
    BottomLong,
    Matrice,
    Stat
  },
  data() {
    // ...
  },
  methods: {
    // ...
  },
  mounted() {
    console.log("analyse mounted");
  }
};
</script>

<style>
.analyse-container {
  display: flex;
  flex-direction: column;
  height: 100vh; /* Utiliser la hauteur de la fenêtre */
}

.top-section {
  display: flex;
  flex-wrap: wrap; /* Permet aux éléments de passer à la ligne si nécessaire */
  justify-content: space-between;
}

.Matrice, .Stat {
  flex: 1; /* Répartit l'espace également */
  min-width: 300px; /* Une largeur minimale pour éviter que les composants ne deviennent trop petits */
}

.BottomLong {
  flex-basis: 100%; /* Prend toute la largeur disponible */
}
</style>

