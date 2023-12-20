<script setup>
import SetInfos from '../components/homeVue/SetInfos.vue';
</script>

<template>
  <div class="container">
    <SetInfos />
    <div class="footer">
      <span>Version du logiciel: 1.0.1</span>
    </div>
  </div>
</template>

<style scoped>
.container {
  margin: 40px;
}
.footer {
  padding: 10px;
  text-align: left;
  color: rgb(255, 255, 255);
}
</style>
