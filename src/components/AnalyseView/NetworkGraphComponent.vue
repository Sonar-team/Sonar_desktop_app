<script lang="ts">
import { defineComponent, markRaw } from "vue"
import cytoscape from "cytoscape"
import cola from "cytoscape-cola"
import { useCaptureStore } from "../../store/capture"
import { save } from "@tauri-apps/plugin-dialog"
import { writeFile } from "@tauri-apps/plugin-fs"
import { EdgeData, EdgeId, GraphData, GraphUpdate, NodeData } from "../../types/capture"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentDate } from '../../utils/time';
import LegendComponent from './LegendComponent.vue';

cytoscape.use(cola)

// --- Colors ----------------------------------------------------------------
const EDGE_COLORS_LC: Record<string, string> = Object.freeze({
  arp: "#FFFF00",
  ipv4: "#FFA500",
  ipv6: "#EE82EE",
  profinet_rt: "#008000",
  tls: "#0000FF",
  dns: "#FF0000",
  ntp: "#FFA500",
})
const colorForLabel = (label: string) =>
  EDGE_COLORS_LC[label?.toLowerCase?.() ?? ""] || "#ffffff"

// --- Helpers ---------------------------------------------------------------
function clamp01(x: number) { return x < 0 ? 0 : x > 1 ? 1 : x }
function hexToRgb(hex: string) {
  const h = hex.startsWith("#") ? hex.slice(1) : hex
  const v = parseInt(h.length === 3 ? h.replace(/(.)/g, "$1$1") : h, 16)
  return { r: (v >> 16) & 255, g: (v >> 8) & 255, b: v & 255 }
}
function rgbToHex(r: number, g: number, b: number) {
  return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)
}
function darken(hex: string, factor = 0.2) {
  const { r, g, b } = hexToRgb(hex)
  return rgbToHex((r * (1 - factor)) | 0, (g * (1 - factor)) | 0, (b * (1 - factor)) | 0)
}
function brighten(hex: string, factor = 0.15) {
  const { r, g, b } = hexToRgb(hex)
  return rgbToHex(
    (clamp01(r / 255 + factor) * 255) | 0,
    (clamp01(g / 255 + factor) * 255) | 0,
    (clamp01(b / 255 + factor) * 255) | 0
  )
}
const EDGE_SEP = "__"
function edgeKey(e: EdgeData): EdgeId {
  return `${e.source}${EDGE_SEP}${e.target}${EDGE_SEP}${e.label}`
}

// Position de départ aléatoire pour les nouveaux nœuds (évite l'empilement en (0,0))
function randomPos() {
  return { x: (Math.random() - 0.5) * 600, y: (Math.random() - 0.5) * 600 }
}

// Seuils de zoom pour l'affichage des labels d'arêtes (identiques à l'ancien rendu)
const EDGE_LABEL_ZOOM = 1.2
const PORT_LABEL_ZOOM = 1.8

// Layout force continu (équivalent du ForceLayout d3-force de v-network-graph)
const FORCE_LAYOUT_OPTIONS = {
  name: "cola",
  animate: true,
  infinite: true,
  fit: false,
  nodeSpacing: 30,
  edgeLength: 150,
} as cytoscape.LayoutOptions

const GRAPH_STYLE = [
  {
    selector: "node",
    style: {
      width: 40,
      height: 40,
      "background-color": "data(color)",
      "border-width": 3,
      "border-color": "data(stroke)",
      label: "data(displayLabel)",
      "font-size": 20,
      color: "#ffffff",
      "text-valign": "top",
      "text-halign": "center",
      "text-margin-y": -6,
      "text-background-color": "#000000",
      "text-background-opacity": 1,
      "text-background-padding": "3px",
      "text-background-shape": "roundrectangle",
    },
  },
  {
    selector: "node.hover",
    style: { "background-color": "data(hover)" },
  },
  {
    selector: "node:selected",
    style: { "border-color": "#1de9b6" },
  },
  {
    selector: "edge",
    style: {
      width: 2,
      "curve-style": "bezier",
      "control-point-step-size": 20,
      "line-color": "data(color)",
      "target-arrow-shape": "triangle",
      "target-arrow-color": "data(color)",
      "arrow-scale": 1,
    },
  },
  // Label protocole au centre (visible à partir de EDGE_LABEL_ZOOM)
  {
    selector: "edge.lbl",
    style: {
      label: "data(label)",
      "font-size": 16,
      color: "#ffffff",
      "text-rotation": "autorotate",
      "text-background-color": "#000000",
      "text-background-opacity": 1,
      "text-background-padding": "2px",
      "text-background-shape": "roundrectangle",
    },
  },
  // Ports source/destination aux extrémités (visibles à partir de PORT_LABEL_ZOOM)
  {
    selector: "edge.ports",
    style: {
      "source-label": "data(sourcePortLabel)",
      "target-label": "data(targetPortLabel)",
      "source-text-offset": 30,
      "target-text-offset": 30,
    },
  },
] as unknown as cytoscape.StylesheetJson

function nodeElementData(node: any) {
  const color = node.color || "#2196F3"
  const label = node.label || ""
  return {
    id: node.id,
    name: node.name || node.id,
    mac: node.mac || "",
    ip: node.ip || "",
    color,
    label,
    displayLabel: label || node.name || node.id,
    stroke: darken(color, 0.25),
    hover: brighten(color, 0.18),
  }
}

function edgeElementData(e: any) {
  return {
    label: e.label || "",
    source_port: e.source_port ?? null,
    destination_port: e.destination_port ?? null,
    sourcePortLabel: `${e.source_port ?? ""}`,
    targetPortLabel: `${e.destination_port ?? ""}`,
    bidir: !!e.bidir,
    color: e._color || colorForLabel(e.label || ""),
  }
}

// --- Component -------------------------------------------------------------
export default defineComponent({
  name: "NetworkGraphComponent",
  components: { LegendComponent },

  data() {
    return {
      cy: null as cytoscape.Core | null,
      layout: null as cytoscape.Layouts | null,

      forceEnabled: true,
      zoomLevel: 1,

      // Bandeau bas
      selectedNodeInfos: [] as string[],
      selectedNode: null as NodeData | null,
      selectedNodeId: null as string | null,
      editedLabel: "" as string,
      isSavingLabel: false as boolean,

      // Queue
      _queue: [] as GraphUpdate[],
      _raf: 0 as number,

      // Affichage des labels d'arêtes selon le zoom
      _edgeLabelsShown: false,
      _portLabelsShown: false,

      // Throttle du redémarrage du layout pendant une capture
      _layoutTimer: 0 as number,
      _layoutPending: false,

      // Handlers pour cleanup
      resetHandler: null as (() => void) | null,
      graphUnsubs: [] as Array<() => void>,
    }
  },

  computed: {
    captureStore() { return useCaptureStore() },
  },

  mounted() {
    this.initCy()

    this.graphUnsubs.push(this.captureStore.onGraphUpdate((update: GraphUpdate) => {
      this._queue.push(update)
      if (!this._raf) {
        this._raf = requestAnimationFrame(() => {
          this.flushQueue()
          this._raf = 0
        })
      }
    }))

    this.graphUnsubs.push(this.captureStore.onGraphSnapshot((graphData) => {
      console.log("[NetworkGraphComponent] GraphSnapshot reçu -> reload");
      this.loadFromGraphData(graphData);
    }));

    // Abonnement au reset via le bus global
    this.resetHandler = () => this.resetGraph()
    this.$bus?.on?.('reset', this.resetHandler)

    if (this.forceEnabled) this.restartLayout()
  },

  beforeUnmount() {
    for (const unsub of this.graphUnsubs) {
      try { unsub() } catch {}
    }
    this.graphUnsubs = []

    if (this._raf) {
      cancelAnimationFrame(this._raf)
      this._raf = 0
    }
    if (this._layoutTimer) {
      window.clearTimeout(this._layoutTimer)
      this._layoutTimer = 0
    }

    if (this.resetHandler) {
      this.$bus?.off?.('reset', this.resetHandler)
      this.resetHandler = null
    }

    if (this.layout) {
      try { this.layout.stop() } catch {}
      this.layout = null
    }
    if (this.cy) {
      this.cy.destroy()
      this.cy = null
    }
  },

  methods: {
    async printLabels() {
      await invoke('get_label_list').then((labels: any) => {
        console.log(labels)
      })
    },

    // === Initialisation Cytoscape ==========================================
    initCy() {
      const container = this.$refs.cyContainer as HTMLElement
      const cy = cytoscape({
        container,
        style: GRAPH_STYLE,
        minZoom: 0.1,
        maxZoom: 5,
      })
      this.cy = markRaw(cy)

      cy.on("tap", "node", (evt) => this.onNodeClick(evt.target.id()))
      cy.on("tap", (evt) => { if (evt.target === cy) this.clearNodeInfos() })
      cy.on("mouseover", "node", (evt) => evt.target.addClass("hover"))
      cy.on("mouseout", "node", (evt) => evt.target.removeClass("hover"))
      cy.on("zoom", () => this.onZoom())
    },

    onZoom() {
      if (!this.cy) return
      const z = this.cy.zoom()
      this.zoomLevel = z

      const showLabels = z >= EDGE_LABEL_ZOOM
      if (showLabels !== this._edgeLabelsShown) {
        this._edgeLabelsShown = showLabels
        this.cy.edges().toggleClass("lbl", showLabels)
      }
      const showPorts = z >= PORT_LABEL_ZOOM
      if (showPorts !== this._portLabelsShown) {
        this._portLabelsShown = showPorts
        this.cy.edges().toggleClass("ports", showPorts)
      }
    },

    // === Réinitialisation ==================================================
    resetGraph() {
      if (this.layout) {
        try { this.layout.stop() } catch {}
        this.layout = null
      }
      this.cy?.elements().remove()
      this.clearNodeInfos()
    },

    // === Upserts ===========================================================
    /** Ajoute ou met à jour un nœud. Retourne true si un élément a été ajouté. */
    upsertNode(node: any): boolean {
      if (!this.cy || !node?.id) return false
      const data = nodeElementData(node)
      const ele = this.cy.getElementById(data.id)
      if (ele.nonempty()) {
        ele.data(data)
        return false
      }
      this.cy.add({ group: "nodes", data, position: randomPos() })
      return true
    },

    /** Ajoute ou met à jour une arête. Retourne true si un élément a été ajouté. */
    upsertEdge(e: any): boolean {
      if (!this.cy || !e?.source || !e?.target) return false
      // Arête orpheline : les deux extrémités doivent exister
      if (this.cy.getElementById(e.source).empty() || this.cy.getElementById(e.target).empty()) return false

      const id = edgeKey(e)
      const data = edgeElementData(e)
      const ele = this.cy.getElementById(id)
      if (ele.nonempty()) {
        ele.data(data)
        return false
      }
      const added = this.cy.add({ group: "edges", data: { id, source: e.source, target: e.target, ...data } })
      added.toggleClass("lbl", this._edgeLabelsShown)
      added.toggleClass("ports", this._portLabelsShown)
      return true
    },

    /**
     * Recharge complètement le graphe à partir d'un snapshot complet
     * envoyé par le backend (GraphSnapshot).
     */
    loadFromGraphData(snapshot: GraphData | null | undefined) {
      console.log("[NetworkGraphComponent] GraphSnapshot reçu -> ", snapshot);

      try {
        if (!snapshot) {
          console.error("[NetworkGraphComponent] Aucune donnée reçue");
          return;
        }
        if (!snapshot.nodes || !snapshot.edges) {
          console.error("[NetworkGraphComponent] Données de graphe invalides:", {
            hasNodes: !!snapshot.nodes,
            hasEdges: !!snapshot.edges
          });
          return;
        }
        if (!this.cy) return

        this.resetGraph()

        const nodeEntries = Object.entries(snapshot.nodes || {});
        console.log(`[NetworkGraphComponent] Chargement de ${nodeEntries.length} nœuds`);

        const edgeEntries = Object.entries(snapshot.edges || {});
        console.log(`[NetworkGraphComponent] Chargement de ${edgeEntries.length} arêtes`);

        this.cy.batch(() => {
          for (const [nodeId, node] of nodeEntries) {
            if (!node) continue;
            this.upsertNode({ ...node, id: node.id || nodeId })
          }
          for (const [edgeId, edge] of edgeEntries) {
            if (!edge) continue;
            if (!edge.source || !edge.target) {
              console.warn(`[NetworkGraphComponent] Arête ${edgeId} invalide: source ou target manquante`);
              continue;
            }
            if (!this.upsertEdge(edge)) {
              console.warn(`[NetworkGraphComponent] Arête orpheline ignorée: ${edge.source} -> ${edge.target} (${edge.label})`);
            }
          }
        })

        this.cy.fit(undefined, 50)
        if (this.forceEnabled) this.restartLayout()

      } catch (error) {
        console.error("[NetworkGraphComponent] Erreur critique dans loadFromGraphData:", error);
      }
    },

    // === Gestion label =====================================================
    onNodeClick(nodeId: string) {
      const ele = this.cy?.getElementById(nodeId)
      if (!ele || ele.empty()) return
      this.selectedNodeId = nodeId
      this.selectedNode = { ...ele.data() } as NodeData
      this.editedLabel = ele.data("label") ?? ""
      this.selectedNodeInfos = this._buildNodeInfos(nodeId)
    },
    clearNodeInfos() {
      this.selectedNodeInfos = []
      this.selectedNode = null
      this.selectedNodeId = null
      this.editedLabel = ""
    },
    async editNodeLabel() {
      if (!this.selectedNode || !this.selectedNodeId || !this.cy) return
      const ele = this.cy.getElementById(this.selectedNodeId)
      if (ele.empty()) return
      const newLabel = String(this.editedLabel ?? "").trim()

      // MAJ UI immédiate
      ele.data({
        label: newLabel,
        displayLabel: newLabel || ele.data("name") || this.selectedNodeId,
      })
      this.selectedNode = { ...ele.data() } as NodeData
      this.selectedNodeInfos = this._buildNodeInfos(this.selectedNodeId)

      // Appel backend avec mac/ip/label
      try {
        this.isSavingLabel = true
        await invoke("add_label", {
          mac: ele.data("mac") ?? "",
          ip: ele.data("ip") ?? "",
          label: newLabel,
        })
      } catch (e) {
        console.error("Erreur add_label:", e)
      } finally {
        this.isSavingLabel = false
      }
    },
    onEditKeydown(e: KeyboardEvent) {
      if (e.key === "Enter") this.editNodeLabel()
      else if (e.key === "Escape") this.clearNodeInfos()
    },
    cancelEdit() {
      if (this.selectedNode && this.selectedNodeId) {
        this.editedLabel = this.selectedNode.label ?? ""
        this.selectedNodeInfos = this._buildNodeInfos(this.selectedNodeId)
      }
    },

    // === Bandeau infos =====================================================
    _buildNodeInfos(nodeId: string): string[] {
      const ele = this.cy?.getElementById(nodeId)
      if (!ele || ele.empty()) return ["Nœud introuvable"]
      const n = ele.data()

      const connected = ele.connectedEdges()
      const protos = new Set<string>()
      connected.forEach((e) => {
        const label = e.data("label")
        if (label) protos.add(String(label))
      })

      return [
        `ID: ${n.id}`,
        `Nom: ${n.name ?? ""}`,
        `Label: ${n.label || "N/A"}`,
        `MAC: ${n.mac ?? ""}`,
        `IP: ${n.ip ?? ""}`,
        `Couleur: ${n.color}`,
        `Degré: ${connected.length}`,
        `Protocoles: ${[...protos].join(", ") || "—"}`,
      ]
    },

    // === Force Layout ======================================================
    restartLayout() {
      if (!this.cy) return
      if (this.layout) {
        try { this.layout.stop() } catch {}
      }
      this.layout = markRaw(this.cy.layout(FORCE_LAYOUT_OPTIONS))
      this.layout.run()
    },
    /** Redémarre le layout au plus une fois toutes les 500 ms pendant une capture. */
    scheduleLayoutRestart() {
      if (!this.forceEnabled) return
      if (this._layoutTimer) {
        this._layoutPending = true
        return
      }
      this.restartLayout()
      this._layoutTimer = window.setTimeout(() => {
        this._layoutTimer = 0
        if (this._layoutPending) {
          this._layoutPending = false
          this.scheduleLayoutRestart()
        }
      }, 500)
    },
    toggleForce() {
      if (this.forceEnabled) {
        this.forceEnabled = false
        if (this.layout) {
          try { this.layout.stop() } catch {}
        }
      } else {
        this.forceEnabled = true
        this.restartLayout()
      }
    },

    // === Export PNG ========================================================
    async downloadPng() {
      if (!this.cy) return
      const filePath = await save({
        filters: [{ name: "PNG File", extensions: ["png"] }],
        defaultPath: getCurrentDate() + "_network_graph_DR_Matrice.png",
      })
      if (!filePath) return

      const blob = this.cy.png({ output: "blob", full: true, scale: 2, bg: "#ffffff" })
      const ab = await blob.arrayBuffer()
      await writeFile(filePath, new Uint8Array(ab))
      console.log(`PNG exporté dans ${filePath}`)
    },

    // === Queue & updates ===================================================
    normalizeGraphUpdate(raw: any): GraphUpdate | null {
      const u = raw?.update ?? raw
      if (!u) return null
      if (u.type && "payload" in u) return u as GraphUpdate
      if (u.NewNode) return { type: "NodeAdded", payload: u.NewNode }
      if (u.NodeUpdated) return { type: "NodeUpdated", payload: u.NodeUpdated }
      if (u.NewEdge) return { type: "EdgeAdded", payload: u.NewEdge }
      if (u.EdgeUpdated) return { type: "EdgeUpdated", payload: u.EdgeUpdated }
      return null
    },
    flushQueue() {
      const q = this._queue
      if (!q.length || !this.cy) return

      let added = false
      this.cy.batch(() => {
        for (let i = 0; i < q.length; i++) {
          if (this.applyUpdate(q[i])) added = true
        }
      })
      this._queue.length = 0

      if (added) this.scheduleLayoutRestart()
    },
    /** Retourne true si un élément a été ajouté au graphe. */
    applyUpdate(update: GraphUpdate | any): boolean {
      if (!update) return false
      const u = this.normalizeGraphUpdate(update)
      if (!u) return false

      switch (u.type) {
        case "NodeAdded":
          return this.upsertNode(u.payload)
        case "NodeUpdated": {
          const node = u.payload
          if (!node) return false
          const added = this.upsertNode(node)

          if (this.selectedNodeId === node.id) {
            const ele = this.cy!.getElementById(node.id)
            this.selectedNode = { ...ele.data() } as NodeData
            this.editedLabel = ele.data("label") ?? ""
            this.selectedNodeInfos = this._buildNodeInfos(node.id)
          }
          return added
        }
        case "EdgeAdded":
        case "EdgeUpdated":
          return this.upsertEdge(u.payload)
      }
      return false
    },
  },
})
</script>

<template>
  <div class="graph-container">
    <div class="top-buttons">
      <button class="download-button" @click="downloadPng" title="Exporter en PNG">⬇️ Export PNG</button>
      <button
        class="force-button"
        :class="{ on: forceEnabled }"
        @click="toggleForce"
        :title="forceEnabled ? 'Désactiver la gravité' : 'Activer la gravité'"
      >
        {{ forceEnabled ? "Gravité: ON" : "Gravité: OFF" }}
      </button>
    </div>

    <!-- Graph -->
    <div class="graph" ref="cyContainer"></div>

    <!-- Bandeau d'infos en bas -->
    <div class="bottom-info">
      <div class="zoom">Zoom: {{ zoomLevel.toPrecision(2) }}</div>
      <div class="sep" />
      <button class="download-button" @click="printLabels" title="afficher les labels">Afficher les labels</button>
      <div class="sep" />
      <div class="node-infos" v-if="selectedNodeInfos.length">
        <strong>Nœud sélectionné</strong>

        <!-- Édition du label -->
        <div class="edit-row">
          <label for="labelInput">Label :</label>
          <input
            id="labelInput"
            v-model="editedLabel"
            type="text"
            placeholder="Entrer un label…"
            @keydown="onEditKeydown"
          />
          <button
            class="primary"
            :disabled="isSavingLabel || !selectedNode"
            @click="editNodeLabel"
            title="Valider la modification"
          >
            {{ isSavingLabel ? "Enregistrement…" : "Enregistrer" }}
          </button>
          <button class="ghost" @click="cancelEdit" :disabled="isSavingLabel">Annuler</button>
        </div>

        <ul>
          infos :
          <li v-for="(info, idx) in selectedNodeInfos" :key="idx">{{ info }}</li>
        </ul>
      </div>
      <div class="node-infos hint" v-else>
        Clique sur un nœud pour afficher ses informations.
      </div>

    </div>
    <LegendComponent />
  </div>
</template>

<style scoped>
.graph-container {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: #111;
  overflow: hidden;
}
.graph { flex: 1; min-height: 0; background: #000; }

/* Boutons */
.top-buttons {
  position: absolute;
  top: 10px;
  left: 10px;
  display: flex;
  gap: 10px;
  z-index: 10;
}
.download-button, .force-button {
  background: #0b1b25;
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 8px 14px;
  cursor: pointer;
}
.force-button.on { box-shadow: 0 0 0 2px #1de9b6 inset; }

/* Bandeau bas */
.bottom-info {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 200px; /* ajuste si besoin */
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: #0f0f0fcc;
  color: #eaeaea;
  border-top: 1px solid #333;
  backdrop-filter: blur(4px);
  z-index: 20;
}
.bottom-info .zoom { font-variant-numeric: tabular-nums; }
.bottom-info .sep {
  width: 1px;
  height: 20px;
  background: #333;
}
.node-infos {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.node-infos ul {
  list-style: none;
  margin: 4px 0 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.node-infos li { opacity: 0.95; }
.node-infos.hint { opacity: 0.7; font-style: italic; }

.edit-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0 10px;
}
.edit-row input {
  background: #0b0b0b;
  color: #eaeaea;
  border: 1px solid #333;
  border-radius: 6px;
  padding: 6px 8px;
  min-width: 220px;
}
button.primary {
  background: #116466;
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 6px 10px;
  cursor: pointer;
}
button.primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
button.ghost {
  background: transparent;
  color: #bbb;
  border: 1px solid #444;
  border-radius: 6px;
  padding: 6px 10px;
  cursor: pointer;
}
</style>
