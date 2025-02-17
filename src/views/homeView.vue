<script setup>
import { ref } from 'vue';
import Capture from '../components/homeVue/Capture.vue';
import FromPcap from '../components/homeVue/FromPcap.vue';

const showCapture = ref(true); // état pour savoir quel composant est au-dessus

// Fonction pour basculer entre Capture et FromPcap
const toggleView = () => {
  showCapture.value = !showCapture.value;
};
</script>

<template>
  <div class="container">
    <div class="tab" @click="toggleView">
      <button>{{ showCapture ? "Ouvrir un Pcap" : "Afficher Capture" }}</button>
    </div>

    <transition name="fade">
      <Capture v-if="showCapture" class="overlay" />
      <FromPcap v-else class="overlay" />
    </transition>

    <div class="footer">
      <span>Version 2.0.0 - Bêta:</span>
      <!-- <span>© 2024 A-GPL - MARINE NATIONALE - SSF TOULON - ERDT CYBER - CYPRIEN AVICO</span> -->
    </div>
  </div>
</template>

<style scoped>
.container {
  position: relative;
  margin: 40px;

}

.tab {
  position: absolute;
  top: 0;
  right: 10px;
  padding: 5px 10px;
  background-color: #444;
  color: white;
  cursor: pointer;
  border-radius: 0 0 5px 5px;
}

.overlay {
  position: relative;
  z-index: 1;
}


.footer {
  padding: 10px;
  text-align: left;
  color: rgb(255, 255, 255);
}
</style>
