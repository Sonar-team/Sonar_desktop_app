<template>
  <div class="page-container">
    <TopBar @toggle-config="toggleConfig" />
    <ConfigPanel v-if="showConfig" @update:visible="(val: any) => showConfig = val" />

    <div class="content">
      <NetworkGraphComponent v-if="showMatrice" />
      <Matrice v-else />
    </div>

    <BottomLong />
    <StatusBar />
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import Matrice from '../components/AnalyseView/Matrice.vue';
import NetworkGraphComponent from '../components/AnalyseView/NetworkGraphComponent.vue';
import TopBar from '../components/NavBar/TopBar.vue';
import StatusBar from '../components/NavBar/status-bar/StatusBar.vue';
import ConfigPanel from '../components/AnalyseView/ConfigPanel.vue';
import BottomLong from '../components/AnalyseView/BottomLong.vue';

export default defineComponent({
  name: 'MainView',
  components: {
    TopBar,
    ConfigPanel,
    NetworkGraphComponent,
    Matrice,
    BottomLong,
    StatusBar
  },
  data() {
    return {
      showConfig: false,
      showMatrice: false
    };
  },
  methods: {
    toggleConfig() {
      this.showConfig = !this.showConfig;
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
