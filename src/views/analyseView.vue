<template>
  <div class="page-container">
    <div class="top-container">
      <TopBar @toggle-config="toggleConfig" @toggle-pcap="togglePcap" @toggle-filter="toggleFilter"/>
      <div class="panels">
        <ConfigPanel v-if="showConfig" @update:ConfigPanel-visible="(val: any) => showConfig = val" />
        <ImportPanel v-if="showPcap" @update:visible="(val: any) => showPcap = val"/>
        <Filter v-if="showFilter" @update:visible="(val: any) => showFilter = val"/>
      </div>
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
import Filter from '../components/AnalyseView/panels/Filter.vue';

export default defineComponent({
  name: 'MainView',
  components: {
    TopBar,
    ImportPanel,
    ConfigPanel,
    NetworkGraphComponent,
    Matrice,
    BottomLong,
    StatusBar,
    Filter
  },
  data() {
    return {
      showConfig: false,
      showPcap: false,
      showFilter: false,
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
    },
    toggleFilter() {
      this.showFilter = !this.showFilter;
    }
  }
});
</script>

<style scoped>
.page-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.top-container {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.content-container {
  margin-top: 40px;
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.status-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  z-index: 10;
}
</style>
