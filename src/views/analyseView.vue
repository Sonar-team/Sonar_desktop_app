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
  height: 100%;
}

.top-section {
  display: flex;
  justify-content: space-between;
}

/* Assurez-vous que les composants Matrice et Stat ont des largeurs appropriées */
.Matrice, .Stat {
  flex: 1; /* Répartit l'espace également entre Matrice et Stat */
}
</style>
