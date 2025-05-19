<template>
  <div class="page-container">
    <div class="top-container">
      <TopBar @toggle-config="toggleConfig" @toggle-pcap="togglePcap"/>
      <ConfigPanel v-if="showConfig" @update:ConfigPanel-visible="(val: any) => showConfig = val" />
      <ImportPanel v-if="showPcap" @update:visible="(val: any) => showPcap = val"/>
    </div>

    <div class="content-container">
      <NetworkGraphComponent v-if="!captureStore.showMatrice" />
      <Matrice v-else />
    </div>

    <div class="status-bar">
      <BottomLong />
      <StatusBar />
    </div>
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


}

.top-container {
  flex-shrink: 0; /* Ne pas réduire */
  display: flex;
  flex-direction: column;
}

.content-container {
  margin-top: 40px;
  flex: 1;
}

.status-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  z-index: 10;
}
</style>
