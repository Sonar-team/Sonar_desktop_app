<!-- YourVueComponent.vue -->
<script setup lang="ts">
  import { VNetworkGraph } from "v-network-graph"
  import { invoke } from '@tauri-apps/api/tauri';

  import * as vNG from "v-network-graph"
  import data from "./data.js"

  const configs = vNG.defineConfigs({
  node: {
    selectable: true,
    normal: { color: "#E0E0E0" }, // Light grey for visibility on dark background
    label: { 
      color: "#E0E0E0",
      fontSize: 18},   // Same as node color for consistency
  },
  edge: {
    selectable: true,
    normal: { color: "#C0C0C0" }, // Slightly darker grey for distinction but still visible
  }
})

</script>

<template>

  <v-network-graph
    class="graph"
    :nodes="data.nodes"
    :edges="data.edges"
    :layouts="data.layouts"
    :configs="configs"

  />

</template>

<style scoped>
.graph {


  height: 500px;
}

</style>
