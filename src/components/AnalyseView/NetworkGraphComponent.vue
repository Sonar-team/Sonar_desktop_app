<!-- YourVueComponent.vue -->
<script >
  import { VNetworkGraph, VEdgeLabel } from "v-network-graph"
  import * as vNG from "v-network-graph"
  import {ForceLayout} from "v-network-graph/lib/force-layout"
  import html2canvas from 'html2canvas';

  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { info, error } from '@tauri-apps/plugin-log';

  import { getCurrentDate, padZero } from '../../utils/time';

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
    menuTargetNode: [], // Pour stocker le nœud ciblé par le menu contextuel
    menuTargetEdges: [], // Pour stocker les arêtes ciblées par le menu contextuel
    packets: [],
    intervalId: null,
    configs: vNG.defineConfigs({
      view: {
        maxZoomLevel: 5,
        minZoomLevel: 0.1,
        
        layoutHandler: new ForceLayout({}),
      },
      node: {
        selectable: true,
        normal: { 
          radius: 20,
          color: node => node.color
         }, // Light grey for visibility on dark background
        label: { 
          visible: true,
          color: "#E0E0E0",
          fontSize: 18,
          directionAutoAdjustment: true,
        }, // Same as node color for consistency
      },
      edge: {
        gap: 50,
        type: "curve",
        selectable: true,
        hoverable: true,
        normal: {
          width: 2, // Ou toute autre largeur par défaut que vous souhaitez
          color: edge => { // Ici, vous définissez dynamiquement la couleur de l'arête
            switch(edge.label) {
              case 'Arp':
                return 'yellow';
              case 'Ipv4':
                return 'orange';
              case 'Ipv6':
                return 'violet';
              case 'Profinet_rt':
                return 'green';
              // Ajoutez d'autres cas selon vos besoins
              default:
                return 'white'; // Couleur par défaut
            }
          },
        },
        label: { // Configuration du label conservée telle quelle
          fontFamily: undefined,
          fontSize: 21,
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
      },
    }),
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

    this.installationName = this.$route.params.installationName;
    this.niveauConfidentialite = this.$route.params.confidentialite;
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
          info('Attempting to export SVG...');
          const svgContent = await this.$refs.graphnodes.exportAsSvgText();
          save({
            filters: [{
              name: 'SVG File',
              extensions: ['svg']
            }],
            defaultPath: getCurrentDate()+ '_' + this.niveauConfidentialite  + '_' + this.installationName+ '.svg' // Set the default file name here
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
    async downloadPng() {
      // Ouvre une boîte de dialogue pour choisir l'emplacement du fichier
      info("save png")
      const filePath = await save({
        filters: [{
          name: 'PNG File',
          extensions: ['png']
        }],
        defaultPath: getCurrentDate() + '_' + this.niveauConfidentialite + '_' + this.installationName + '.png' // Nom de fichier par défaut
      });

      if (filePath) {
        // Capture tout le document
        info("file path: ",filePath)
        html2canvas(document.body, { scale: 2, useCORS: true }).then(canvas => {
          // Convertit le canvas en une chaîne Base64
          const base64Data = canvas.toDataURL('image/png'); // Format PNG

          // Supprime l'en-tête "data:image/png;base64,"
          const base64WithoutHeader = base64Data.replace(/^data:image\/png;base64,/, '');

          // Envoie les données au backend
          invoke('write_png_file', { path: filePath, contents: base64WithoutHeader })
            .then(() => console.log('PNG successfully saved at:', filePath))
            .catch((error) => console.error('Error saving PNG:', error));
        });
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
      if (this.edgeMenu) {
        const edgeData = this.graphData.edges[edge];
        this.menuTargetEdges = [
          `Adresse Mac Source: ${edgeData.source}, 
          Adresse Mac Destination: ${edgeData.target}, 
          Protocole: ${edgeData.label}`
        ];
        this.showContextMenu(this.edgeMenu, event);
      }
    },
    showNodeContextMenu({ node, event }) {
      if (this.nodeMenu) {
        const nodeData = this.graphData.nodes[node];
        this.menuTargetNode = [
          `Adresse IP: ${nodeData.name},
          Mac_address: ${nodeData.mac}`
        ];
        this.showContextMenu(this.nodeMenu, event);
      }
    },
  }
}
</script>

<template>
  <div class="graph-container">
    
    <button class="download-button" @click="downloadPng">PNG</button>
    <button class="download-button" @click="downloadSvg" style="left: 100px;">SVG</button>
    
    <v-network-graph
      class="graph"
      ref="graphnodes"
      zoom-level=3
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
  </div>
  <div ref="nodeMenu" class="context-menu">
    Infos du noeud:
    <ul class="contenu">
      <li v-for="(info, index) in menuTargetNode" :key="index">{{ info }}</li>
    </ul>
  </div>
  <div ref="edgeMenu" class="context-menu">
    Infos de l'arête:
    <div class="contenue">{{ menuTargetEdges.join(", ") }}</div>
  </div>
</template>

<style scoped>
.graph-container {
  position: fixed; /* Establishes a relative positioning context */
  bottom: 225px; /* Ajustez cette valeur selon la hauteur de votre barre de statut */
  left: 0;
  right: 0;
  height: 735px; /* Adjust height as needed */
  width: 100%; /* Container takes full width */
}

.graph {
  height: 100%; /* Graph takes full height of the container */
  border: 2px solid #3a3a3a;
  border-radius: 10px;
  width: 100%;
  text-align: center;
  color: #FFF;
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  background-color: #1a1a1a;
}

.download-button {
  position: absolute; /* Absolutely positioned relative to its nearest positioned ancestor */
  top: 10px; /* Distance from the top of the container */
  left: 10px; /* Distance from the left of the container */
  background-color: #0b1b25;
  color: #fff;
  padding: 10px 20px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  z-index: 10; /* Ensure the button is above the graph */
}

.context-menu {
  color: #0b1b25;
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
  color: #0b1b25;
  border: 1px dashed #aaa;
  padding: 4px;
  margin-top: 8px;
}

</style>
