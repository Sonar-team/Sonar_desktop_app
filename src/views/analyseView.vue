<template>
  <div class="page-container">
    <TopBar @toggle-config="toggleConfig" @toggle-pcap="togglePcap"/>
    <ConfigPanel v-if="showConfig" @update:ConfigPanel-visible="(val: any) => showConfig = val" />
    <ImportPanel v-if="showPcap" @update:visible="(val: any) => showPcap = val"/>
    <div class="content">
      <NetworkGraphComponent v-if="!captureStore.showMatrice" />
      <Matrice v-else />
    </div>

    <BottomLong />
    <StatusBar />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { useCaptureStore } from '../store/capture';

import Matrice from '../components/AnalyseView/Matrice.vue';
import NetworkGraphComponent from '../components/AnalyseView/NetworkGraphComponent.vue';
import TopBar from '../components/NavBar/TopBar.vue';
import StatusBar from '../components/NavBar/status-bar/StatusBar.vue';
import ConfigPanel from '../components/AnalyseView/panels/ConfigPanel.vue';
import BottomLong from '../components/AnalyseView/BottomLong.vue';
import ImportPanel from '../components/AnalyseView/panels/ImportPalnel.vue';


export default defineComponent({
  name: 'MainView',
  components: {
    TopBar,
    ImportPanel,
    ConfigPanel,
    NetworkGraphComponent,
    Matrice,
    BottomLong,
    StatusBar
  },
  data() {
    return {
      showConfig: false,
      showPcap: false,
    };
  },
  computed: {
    captureStore() {
      return useCaptureStore();
    }
  },
  methods: {
    toggleConfig() {
      this.showConfig = !this.showConfig;
    },
    togglePcap() {
      this.showPcap = !this.showPcap;
    }
  }
});
</script>

<style scoped>
.page-container {
  display: flex;
  flex-direction: column;
  padding-top: 70px;
  box-sizing: border-box;
  overflow: hidden;
}
</style>
