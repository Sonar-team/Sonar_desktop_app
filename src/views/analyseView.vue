<template>
  <TopNavBar @change-component="handleComponentChange" />
  <LeftNavBar />
  <AnalyseView ref="analyseView" />
</template>

<script>
import TopNavBar from '../components/NavBar/TopNavBar.vue';
import LeftNavBar from '../components/NavBar/LeftNavBar.vue';
import MatriceVue from '../components/CaptureVue/matricevue/MatriceVue.vue';

export default {
  components: {
    TopNavBar,
    LeftNavBar,
    MatriceVue
  },
  methods: {
    handleComponentChange(componentName) {
      this.$refs.analyseView.changeComponent(componentName);
    }
  }
}
</script>

<style scoped>


</style>
