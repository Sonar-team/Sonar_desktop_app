<script lang="ts">
import { VNetworkGraph, VEdgeLabel } from "v-network-graph"
import * as vNG from "v-network-graph"
import {ForceLayout} from "v-network-graph/lib/force-layout"
import html2canvas from 'html2canvas';
import ColorConvert from "color-convert"

import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { info } from '@tauri-apps/plugin-log';

import { getCurrentDate } from '../../utils/time';

// Constants for graph configuration
const GRAPH_CONFIGS = {
  view: {
    maxZoomLevel: 5,
    minZoomLevel: 0.1,
    layoutHandler: new ForceLayout({}),
  },
  node: {
    selectable: true,
    radius: 20,
    color: node => node.color,
    label: {
      fontSize: 16,
      color: "#ffffff",
      direction: "north",

    },
  },
  edge: {
    gap: 50,
    type: "curve",
    selectable: true,
    hoverable: true,
    normal: {
      width: 2, 
      color: edge => { 
        switch(edge.label) {
          case 'Arp':
            return 'yellow';
          case 'Ipv4':
            return 'orange';
          case 'Ipv6':
            return 'violet';
          case 'Profinet_rt':
            return 'green';
          case 'TLS':
            return 'blue';
          case 'DNS':
            return 'red';
          case 'NTP':
            return 'orange';
          default:
            return 'white'; 
        }
      },
    },
    label: { 
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
};

// Edge color mapping for different types of connections
const EDGE_COLORS = {
  'Arp': 'yellow',
  'Ipv4': 'orange',
  'Ipv6': 'violet',
  'Profinet_rt': 'green',
  'TLS': 'blue',
  'DNS': 'red',
  'NTP': 'orange',

};

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
        edges: []
      },
      selectedNode: null,
      viewMenu: null,
      nodeMenu: null,
      edgeMenu: null,
      menuTargetNode: [],
      menuTargetEdges: [],
      intervalId: null,
      configs: vNG.defineConfigs({
        view: GRAPH_CONFIGS.view,
        node: {
          ...GRAPH_CONFIGS.node,
          normal: {
            radius: GRAPH_CONFIGS.node.radius,
            color: node => node.color,
            strokeWidth: 3,
            strokeColor: node => this.darker(node.color, 20),
 
          }
        },
        edge: {
          ...GRAPH_CONFIGS.edge,
          normal: {
            ...GRAPH_CONFIGS.edge.normal,
            color: edge => EDGE_COLORS[edge.label] || '#ffffff'
          },
          marker: {
            source: {
              type: "none",
              width: 4,
              height: 4,
              margin: -1,
              offset: 0,
              units: "strokeWidth",
              color: null,
            },
            target: {
              type: "arrow",
              width: 6,
              height: 6,
              margin: 0,
              offset: 0,
              units: "strokeWidth",
              color: null,
            },
          },
          label: GRAPH_CONFIGS.edge.label
        },
      }),
      notifications: []
    };
  },
  computed: {
    // This computed property is unused, consider removing it
    processedPackets() {
      return this.processData(this.packets);
    }
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
    darker(hex: String, level: Number) {
      const hsv = ColorConvert.hex.hsv(hex)
      hsv[2] -= level
      return "#" + ColorConvert.hsv.hex(hsv)
    },
    // Fetches network packet information from backend
    async fetchPacketInfos() {
      try {
        const jsonString = await invoke('get_graph_state', {});
        this.graphData = JSON.parse(jsonString);

      } catch (error) {
        error("Error fetching packet infos:", error);
        this.showNotification('Error fetching network data', 'error');
      }
    },

    // Handles SVG export with error handling and notifications
    async downloadSvg() {
      if (!this.$refs.graphnodes || !this.$refs.graphnodes.exportAsSvgText) {
        this.showNotification('Graph export failed: Graph not initialized', 'error');
        return;
      }

      try {
        const svgContent = await this.$refs.graphnodes.exportAsSvgText();
        await this.saveFile(svgContent, 'svg');
      } catch (error) {
        console.error('Error exporting SVG:', error);
        this.showNotification('Failed to export SVG', 'error');
      }
    },

    // Handles PNG export with error handling and notifications
    async downloadPng() {
      if (!this.$refs.graphnodes) {
        this.showNotification('Graph export failed: Graph not initialized', 'error');
        return;
      }

      try {
        const canvas = await html2canvas(this.$refs.graphnodes.$el, { scale: 2, useCORS: true });
        const pngData = canvas.toDataURL('image/png');
        const blob = this.dataURLToBlob(pngData);
        await this.saveFile(blob, 'png');
      } catch (error) {
        console.error('Error exporting PNG:', error);
        this.showNotification('Failed to export PNG', 'error');
      }
    },

    // Helper function to save files of any format
    async saveFile(content, format) {
      const fileName = this.generateFileName(format);
      const filePath = await save({
        filters: [{
          name: format.toUpperCase(),
          extensions: [format]
        }],
        defaultPath: fileName
      });

      if (filePath) {
        try {
          await invoke('write_file', { path: filePath, contents: content });
          this.showNotification(`Successfully saved ${format.toUpperCase()} file`, 'success');
        } catch (error) {
          console.error('Error saving file:', error);
          this.showNotification('Failed to save file', 'error');
        }
      }
    },

    // Generates standardized file names for exports
    generateFileName(format) {
      return `${getCurrentDate()}_${this.niveauConfidentialite}_${this.installationName}.${format}`;
    },

    // Converts data URLs to Blob objects for PNG export
    dataURLToBlob(dataURL) {
      const arr = dataURL.split(',');
      const mime = arr[0].match(/:(.*?);/)[1];
      const bstr = atob(arr[1]);
      let n = bstr.length;
      const u8arr = new Uint8Array(n);
      while (n--) {
        u8arr[n] = bstr.charCodeAt(n);
      }
      return new Blob([u8arr], { type: mime });
    },

    // Shows a notification with automatic timeout
    showNotification(message, type = 'info') {
      const notification = {
        message,
        type,
        timestamp: Date.now()
      };
      this.notifications.push(notification);
      
      // Remove notification after 5 seconds
      setTimeout(() => {
        this.notifications = this.notifications.filter(n => n !== notification);
      }, 5000);
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
          `Adresse: ${nodeData.name},
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
      <template #edge-label="{ edge, scale, ...slotProps }">
        <!-- Ligne 1 : type de protocole -->
        <v-edge-label
          :text="edge.label"
          align="center"
          vertical-align="above"
          v-bind="slotProps"
          :font-size="18 * scale"
          fill="#FFFFFF"
        />
        <!-- Ligne 2 : ports source/destination -->
        <v-edge-label
          :text="`${edge.source_port ?? ''}`"
          align="source"
          vertical-align="below"
          v-bind="slotProps"
          :font-size="14 * scale"
          fill="#E0E0E0"
        />
        <v-edge-label
          :text="`${edge.destination_port ?? ''}`"
          align="target"
          vertical-align="below"
          v-bind="slotProps"
          :font-size="14 * scale"
          fill="#E0E0E0"
        />
      </template>
    </v-network-graph>
  </div>

  <!-- Context menu -->
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

  <div v-for="notification in notifications" :key="notification.timestamp" class="notification" :class="notification.type">
    {{ notification.message }}
  </div>
</template>

<style scoped>
.graph-container {
  position: relative; /* <-- indispensable */
  flex: 1;
  display: flex;
  flex-direction: column;
  width: 100%;
  overflow: hidden;
  background-color: #1a1a1a;
  height: 100%;
}

.graph {
  flex: 1;
  width: 100%;
  text-align: center;
  color: #FFF;
  background-color: #000000;
}

.download-button {
  position: absolute;
  top: 10px;
  left: 10px;
  background-color: #0b1b25;
  color: #fff;
  padding: 10px 20px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  z-index: 10;
}

.context-menu {
  color: #0b1b25;
  border-radius: 10px;
  width: 180px;
  background-color: #efefef;
  padding: 10px;
  position: absolute;
  visibility: hidden;
  font-size: 12px;
  border: 1px solid #aaaaaa;
  box-shadow: 2px 2px 2px #e7bf0c;
}

.contenue {
  color: #0b1b25;
  border: 1px dashed #aaa;
  margin-top: 8px;
}

.notification {
  position: absolute;
  top: 10px;
  right: 10px;
  padding: 10px;
  border-radius: 5px;
  font-size: 12px;
}

.notification.info {
  background-color: #2196f3;
  color: #fff;
}

.notification.success {
  background-color: #4caf50;
  color: #fff;
}

.notification.error {
  background-color: #f44336;
  color: #fff;
}
</style>
