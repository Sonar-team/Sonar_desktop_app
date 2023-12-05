<!-- YourVueComponent.vue -->
<script setup lang="ts">
  import { VNetworkGraph } from "v-network-graph"
  import "v-network-graph/lib/style.css"
  import { useRoute, useRouter } from 'vue-router';

    // Define nodes and edges as before
    // ...

    const router = useRouter(); // Use the useRouter hook to programmatically navigate

    function handleClick() {
    // Any additional logic for when the button is clicked
    goToAnalysePage();
    }

    function goToAnalysePage() {
    router.push('/analyse'); // Navigate to the graph page
    }

  const nodes = {
  node1: { name: "Source\nMAC: 00:00:00:00:00:00\nIP: 127.0.0.1\nPort: 17664" },
  node2: { name: "L2 Interface: lo" },
  node3: { name: "Destination\nMAC: 00:00:00:00:00:00\nIP: 127.0.0.1\nPort: 53" },
  node4: { name: "Destination\nMAC: 00:00:00:00:00:00\nIP: 127.0.0.1\nPort: 52" },
  // Add more nodes if needed
  node5: { name: "Additional Node\nMAC: AA:BB:CC:DD:EE:FF\nIP: 127.0.0.2\nPort: 80" },
}

    const edges = {
    edge1: { source: "node1", target: "node2", protocol: "IPv4/TCP" },
    edge2: { source: "node2", target: "node3", protocol: "IPv4/TCP" },
    edge3: { source: "node2", target: "node4", protocol: "IPv4/TCP" },
    // Add more edges if needed
    edge4: { source: "node2", target: "node5", protocol: "IPv4/HTTP" },
    }

    const layouts = {
    nodes: {
        node1: { x: 0, y: 0 },
        node2: { x: 50, y: 50 },
        node3: { x: 100, y: 0 },
        node4: { x: 150, y: 50 },
    },
}



</script>

<template>
    <div class="button-container">
        <button @click="goToAnalysePage">Vue analyse</button>
    </div>
    <v-network-graph
      class="graph"
      :nodes="nodes"
      :edges="edges"
      :layouts="layouts"
    />

  </template>

<style>
.graph {
  width: 100%; /* Use 100% to be more responsive */
  max-width: 800px; /* Maximum width to maintain design integrity */
  height: auto; /* Responsive height */
  aspect-ratio: 4 / 3; /* Maintain aspect ratio */
  border: none; /* Remove border or set to a style that fits the theme */
  background-color: transparent; /* Match with dark theme or use a dark color */
  color: #ffffff; /* Text color for dark theme */
  /* Add any additional styles for the graph here */
}

/* Additional styles for nodes and edges if needed */
.v-network-graph-node {
  fill: #2f9c5b; /* Node fill color for dark theme */
  stroke: #325c3f; /* Node stroke color for contrast */
}

.v-network-graph-edge {
  stroke: #ffffff; /* Edge color for dark theme */
}

/* You can add more custom styles targeting specific classes within the v-network-graph library */
</style>

