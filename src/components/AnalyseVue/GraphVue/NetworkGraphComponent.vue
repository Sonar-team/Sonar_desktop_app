<!-- YourVueComponent.vue -->
<script >
  import { VNetworkGraph } from "v-network-graph"
  import { invoke } from '@tauri-apps/api/tauri';

  import * as vNG from "v-network-graph"
  import {
    ForceLayout,
  } from "v-network-graph/lib/force-layout"

  export default {
  components: {
    VNetworkGraph,
  },
  data() {
    return {
      graphData: {
        nodes: [],
        edges: [],
      },
      packets: [],
      intervalId: null,

      configs: vNG.defineConfigs({
        view: {
          layoutHandler: new ForceLayout({}),
        },
        node: {
          selectable: true,
          normal: { color: "#E0E0E0" }, // Light grey for visibility on dark background
          label: { 
            color: "#E0E0E0",
            fontSize: 18,
            directionAutoAdjustment: true,
          },   // Same as node color for consistency
        },
        edge: {
          selectable: true,
          normal: { color: "#C0C0C0" }, // Slightly darker grey for distinction but still visible
        }
      })
    };
  },
  computed: {
    processedPackets() {
      return this.processData(this.packets);
    }
  },
  mounted() {
    this.intervalId = setInterval(this.fetchPacketInfos, 1000);
  },
  beforeDestroy() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
  },
  methods: {
    async fetchPacketInfos() {
      try {
        const jsonString = await invoke('get_graph_state', {});
        this.graphData = JSON.parse(jsonString);

      } catch (error) {
        console.error("Error fetching packet infos:", error);
      }
    },
  },
}

  
</script>

<template>

  <v-network-graph
    class="graph"
    :nodes="graphData.nodes"
    :edges="graphData.edges"
    :layouts="graphData.layouts"
    :configs="configs"
  />

</template>

<style scoped>
.graph {
  height: 680px;
  border: 2px solid #3a3a3a; /* Bordure plus sombre */
  border-radius: 10px;
  width: 100%;
  text-align: center;
  color: #FFF; /* Texte en blanc */
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  background-color: #1a1a1a; /* Fond plus sombre */
}

</style>
