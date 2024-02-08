<!-- YourVueComponent.vue -->
<script >
  import { VNetworkGraph, VEdgeLabel } from "v-network-graph"
  import { invoke } from '@tauri-apps/api/tauri';
  import { save } from '@tauri-apps/api/dialog';
  import * as vNG from "v-network-graph"
  import {
    ForceLayout,
  } from "v-network-graph/lib/force-layout"

  export default {
  components: {
    VNetworkGraph,
    VEdgeLabel
    
  },
  data() {
    return {
      position: { left: "0", top: "0" },

      graphData: {
        nodes: [],
        edges: [],
      },
      selectedNode: null,

      viewMenu: null, // Utilisez les refs pour les éléments de menu
      nodeMenu: null,
      edgeMenu: null,
      menuTargetNode: '', // Pour stocker le nœud ciblé par le menu contextuel
      menuTargetEdges: [], // Pour stocker les arêtes ciblées par le menu contextuel
      
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
            visible: true,
            color: "#E0E0E0",
            fontSize: 18,
            directionAutoAdjustment: true,
            text: "mac",
          },   // Same as node color for consistency
        },
        edge: {
          selectable: true,
          hoverable: true,
          label: {
            fontFamily: undefined,
            fontSize: 11,
            lineHeight: 1.1,
            color: "#E0E0E0",
            margin: 4,
            background: {
              visible: true,
              color: "#000000",
              padding: {
                vertical: 1,
                horizontal: 4,
              },
              borderRadius: 2,
            },
          },
        }
      })
    };
  },
  computed: {
    processedPackets() {
      return this.processData(this.packets);
    },
  
  },
  mounted() {
    this.intervalId = setInterval(this.fetchPacketInfos, 1000);
    this.viewMenu = this.$refs.viewMenu;
    this.nodeMenu = this.$refs.nodeMenu;
    this.edgeMenu = this.$refs.edgeMenu;
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
    async downloadSvg() {
    if (this.$refs.graphnodes && this.$refs.graphnodes.exportAsSvgText) {
      try {
        const svgContent = await this.$refs.graphnodes.exportAsSvgText();
        
        // Use Tauri's dialog API to open a save file dialog
        save({
          filters: [{
            name: 'SVG File',
            extensions: ['svg']
          }],
          defaultPath: 'network-graph.svg'
        }).then((filePath) => {
          if (filePath) {
            // Use Tauri's fs API to write the file
            invoke('write_file', { path: filePath, contents: svgContent })
              .then(() => console.log('SVG successfully saved'))
              .catch((error) => console.error('Error saving SVG:', error));
          }
        });
      } catch (error) {
        console.error('Error exporting SVG:', error);
      }
    } else {
      console.error('SVG export function not available or graph component not loaded.');
    }
    },
    handleNodeClick(node) {
      // Supposons que `selectedNode` est une propriété de données que vous utiliserez pour stocker les informations du node sélectionné
      this.selectedNode = node;
      console.log('selected node:',this.selectedNode)
    },
    showContextMenu(element, event) {
      console.log('element', element)
      console.log('event', event)

      element.style.left = event.x + "px"
      element.style.top = event.y + "px"
      element.style.visibility = "visible"
      const handler = (event) => {
        if (!event.target || !element.contains(event.target)) {
          element.style.visibility = "hidden"
          document.removeEventListener("pointerdown", handler, { capture: true })
        }
      }
      document.addEventListener("pointerdown", handler, { passive: true, capture: true })
    },

    showEdgeContextMenu({ edge, event }) {
      // Utilisation de showContextMenu pour le menu de l'arête
      if (this.edgeMenu) {
        const edgeData = this.graphData.edges[edge];
        // Formattez ou choisissez les informations de l'arête à afficher
        this.menuTargetEdges = [
          `Adresse Mac Source: ${edgeData.source}, 
          Adresse Mac Destination: ${edgeData.target}, 
          Protocole: ${edgeData.label}`];
        this.showContextMenu(this.edgeMenu, event);
      }
    },
    showNodeContextMenu({ node, event }) {
      // Utilisation de showContextMenu pour le menu du nœud
      if (this.nodeMenu) {
        //console.log("node: " + node)
        const nodeData = this.graphData.nodes[node];
      // Formattez ou choisissez les informations du nœud à afficher
      this.menuTargetNode = `Adresse Mac: ${nodeData.mac}`;
        this.showContextMenu(this.nodeMenu, event);
      }
    },
  }
}
</script>

<template>
  <button class="download-button" @click="downloadSvg">Télécharger l'image</button>
  <v-network-graph
    class="graph"
    ref="graphnodes"
    :nodes="graphData.nodes"
    :edges="graphData.edges"
    :layouts="graphData.layouts"
    :configs="configs"
    :event-handlers="{
      'node:click': showNodeContextMenu,
      'edge:click': showEdgeContextMenu,
    }"
  >
    <template #edge-label="{ edge, ...slotProps }">
      <v-edge-label :text="edge.label" align="above" v-bind="slotProps" />
    </template>
  </v-network-graph>
  <div ref="nodeMenu" class="context-menu">
    Infos du noeud:
    <div class="contenue">{{ menuTargetNode }}</div>
  </div>
  <div ref="edgeMenu" class="context-menu">
    Infos de l'arête:
    <div class="contenue">{{ menuTargetEdges.join(", ") }}</div>
  </div>
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

.download-button {
  background-color: #0b1b25; /* Couleur de fond du bouton */
  color: #fff; /* Couleur du texte du bouton */
  padding: 10px 20px; /* Espacement intérieur du bouton */
  border: none; /* Supprimer la bordure du bouton */
  border-radius: 5px; /* Ajouter une bordure arrondie au bouton */
  cursor: pointer; /* Curseur de type pointeur au survol */
}

.context-menu {
  border-radius: 10px;
  width: 180px;
  background-color: #efefef;
  padding: 10px;
  position: fixed;
  visibility: hidden;
  font-size: 12px;
  border: 1px solid #aaaaaa;
  box-shadow: 2px 2px 2px #e7bf0c;
}
.contenue {
  border: 1px dashed #aaa;
    padding: 4px;
    margin-top: 8px;
}

</style>