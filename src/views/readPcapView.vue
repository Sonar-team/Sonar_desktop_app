<template>
  <div class="container">
    <pcapSideBar />
    <div class="content">
      <h3 class="titre">Matrice de flux : {{ getCurrentDate() + '_' + niveauConfidentialite + '_' + installationName }}</h3>
      <p class="info">Nombre total de paquets lus : {{ totalPackets }}</p> <!-- Affichage du nombre total de paquets -->
      <Matrice v-if="showMatrice" /> <!-- Show Matrice when showMatrice is true -->
      <NetworkGraphComponent v-else /> <!-- Show NetworkGraphComponent otherwise -->
    </div>
  </div>
</template>

<script>
import BottomLong from '../components/CaptureVue/BottomLong.vue';
import Matrice from '../components/CaptureVue/Matrice.vue';
import NetworkGraphComponent from '../components/CaptureVue/NetworkGraphComponent.vue';
import pcapSideBar from '../components/NavBar/pcapSideBar.vue';
import { invoke } from '@tauri-apps/api/core';

export default {
  props: {
    pcapList: {
      type: Array,
      default: () => []
    }
  },
  data() {
    return {
      localPcapList: [], // Copie locale pour la manipulation des fichiers pcap
      tempsReleve: '',
      tramesRecues: 0,
      tramesEnregistrees: 0,
      niveauConfidentialite: '',
      installationName: '',
      showMatrice: true, // Toggle state (true for Matrice, false for NetworkGraphComponent)
      totalPackets: 0 // Propriété pour stocker le nombre total de paquets
    };
  },
  components: {
    BottomLong,
    Matrice,
    NetworkGraphComponent,
    pcapSideBar
  },
  methods: {
    toggleComponent() {
      this.showMatrice = !this.showMatrice; // Toggle the state
    },

    getCurrentDate() {
      const now = new Date();
      const formattedDate = `${now.getFullYear()}${this.padZero(now.getDate())}${this.padZero(now.getMonth() + 1)}`;
      return formattedDate;
    },

    padZero(value) {
      return value < 10 ? `0${value}` : value;
    }
  },
  mounted() {
    this.localPcapList = [...this.pcapList]; // Copie de la prop dans la donnée locale
    console.log(this.localPcapList);

    invoke('convert_from_pcap_list', { pcaps: this.localPcapList })
      .then(response => {
        this.totalPackets = response; // Définit totalPackets avec la réponse renvoyée
        console.log(`Total packets read: ${this.totalPackets}`);
      })
      .catch(error => console.error(error));

    this.$bus.on('toggle', this.toggleComponent);
    this.installationName = this.$route.params.installationName;
    this.niveauConfidentialite = this.$route.params.confidentialite;
  },
  beforeUnmount() {
    this.$bus.off('toggle', this.toggleComponent);
  }
};
</script>

<style scoped>
.container {
  display: flex;
}

.content {
  overflow: auto;
  flex: 1;
}

.titre {
  text-align: center;
  color: aliceblue;
  margin: 1px 0;
}
.info {
  text-align: center;
  color: aliceblue;
  margin: 1px 0;
}

.button {
  background-color: #0b1b25;
  color: #fff;
  padding: 10px 20px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
}
</style>
