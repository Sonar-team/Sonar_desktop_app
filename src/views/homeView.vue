<script setup>
import Capture from '../components/homeVue/Capture.vue';
import Ouvrir from '../components/homeVue/Ouvrir.vue';
import Decouvrir from '../components/homeVue/Decouvrir.vue';
</script>

<template>
  <div class="container">

    <Capture />
    <Ouvrir />
    <Decouvrir />

    <div class="footer">
      <span>Version du logiciel: 1.0.1</span>
    </div>
  </div>
</template>

<style scoped>

.container {
  margin: 40px;
  color: white;
}
.footer {
  padding: 10px;
  text-align: left;
}
</style>
