<template>
  <div class="container">
    
    <div class="content">

    </div>
  </div>
</template>

<script>


import { invoke } from '@tauri-apps/api/core';
import { info } from '@tauri-apps/plugin-log';

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
    };
  },

  mounted() {
    this.localPcapList = [...this.pcapList]; // Copie de la prop dans la donnée locale
    info(`localPcapList: ${this.localPcapList}`); // Affiche this.localPcapList ${this.localPcapList;

    invoke('convert_from_pcap_list', { pcaps: this.localPcapList })
      .then(response => {
        this.totalPackets = response; // Définit totalPackets avec la réponse renvoyée
        info(`Total packets read: ${this.totalPackets}`);
      })
      .catch(error => console.error(error));

    this.$bus.on('toggle', this.toggleComponent);
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
