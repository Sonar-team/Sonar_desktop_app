<script lang="ts">
import { defineComponent, shallowReactive, markRaw, reactive } from "vue"
import { VNetworkGraph, VEdgeLabel } from "v-network-graph"
import * as vNG from "v-network-graph"
import { ForceLayout } from "v-network-graph/lib/force-layout"
import { useCaptureStore } from "../../store/capture"
import { save } from "@tauri-apps/plugin-dialog"
import { writeTextFile } from "@tauri-apps/plugin-fs"
import { GraphUpdate } from "../../types/capture"
import { invoke } from "@tauri-apps/api/core"

// --- Types -----------------------------------------------------------------
type NodeId = string
type EdgeId = string

interface NodeDataBase {
  id: string
  name: string
  mac?: string
  ip?: string      // ← NEW
  color: string
  label?: string
  _hover?: string
  _stroke?: string
}

interface EdgeData {
  source: NodeId
  target: NodeId
  label: string
  source_port?: string | number | null
  destination_port?: string | number | null
  bidir?: boolean
  _color?: string
}

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
function clearReactiveMap<T extends Record<string, any>>(obj: T) {
  for (const k of Object.keys(obj)) delete obj[k]
}
function isFn(x: any, name: string): x is (...a: any[]) => void {
  return x && typeof x[name] === "function"
}

// --- Component -------------------------------------------------------------
export default defineComponent({
  name: "NetworkGraphComponent",
  components: { VNetworkGraph, VEdgeLabel },

  data() {
    const forceLayout = markRaw(new ForceLayout({}))
    const simpleLayout = markRaw(new vNG.SimpleLayout())

    const configs = reactive(
      vNG.defineConfigs({
        view: { maxZoomLevel: 5, minZoomLevel: 0.1, layoutHandler: simpleLayout },
        node: {
          selectable: true,
          normal: {
            radius: 20,
            color: (node: NodeDataBase) => node.color,
            strokeWidth: 3,
            strokeColor: (node: NodeDataBase) => node._stroke ?? darken(node.color, 0.25),
          },
          hover: {
            radius: 20,
            color: (node: NodeDataBase) => node._hover ?? brighten(node.color, 0.18),
          },
          label: { text: (node: NodeDataBase) => node.label || node.name, fontSize: 16, color: "#ffffff", direction: "north" as const },
        },
        edge: {
          type: "straight",
          gap: 10,
          selectable: false,
          normal: {
            width: 2,
            color: (edge: any) => edge._color ?? colorForLabel(edge.label),
          },
          marker: {
            source: { type: "none", width: 5, height: 5, margin: 0, offset: 0, units: "strokeWidth" as const, color: null },
            target: { type: "arrow" as const, width: 5, height: 5, margin: 0, offset: 0, units: "strokeWidth" as const, color: null },
          },
          label: {
            fontSize: 21, lineHeight: 1.1, color: "#E0E0E0", margin: 4,
            background: { visible: true, color: "#000000", padding: { vertical: 1, horizontal: 4 }, borderRadius: 2 },
          },
        },
      })
    )

    return {
      graphData: {
        nodes: shallowReactive(Object.create(null) as Record<NodeId, NodeDataBase>),
        edges: shallowReactive(Object.create(null) as Record<EdgeId, EdgeData>),
        layouts: reactive({}) as Record<string, unknown>,
      },

      forceEnabled: false,
      zoomLevel: 1,

      forceLayout,
      simpleLayout,
      configs,

      // Bandeau bas
      selectedNodeInfos: [] as string[],
      selectedNode: null as NodeDataBase | null,
      selectedNodeId: null as string | null,
      editedLabel: "" as string,
      isSavingLabel: false as boolean,

      // Queue
      _queue: [] as GraphUpdate[],
      _pendingEdges: [] as GraphUpdate[],
      _raf: 0 as number,
    }
  },

  computed: {
    captureStore() { return useCaptureStore() },
    graphNodes(): Record<NodeId, NodeDataBase> { return this.graphData.nodes },
    graphEdges(): Record<EdgeId, EdgeData> { return this.graphData.edges },

    eventHandlers(): vNG.EventHandlers {
      return {
        "node:click": this.onNodeClick,
        "view:click": this.clearNodeInfos,
      }
    },
  },

  mounted() {
    clearReactiveMap(this.graphData.nodes)
    clearReactiveMap(this.graphData.edges)

    this.captureStore.onGraphUpdate((update: GraphUpdate) => {
      this._queue.push(update)
      if (!this._raf) {
        this._raf = requestAnimationFrame(() => {
          this.flushQueue()
          this._raf = 0
        })
      }
    })

    if (this.forceEnabled && isFn(this.forceLayout, "start")) this.forceLayout.start()
  },

  methods: {
    // === Gestion label =====================================================
    onNodeClick({ node }: { node: string }) {
      const n = this.graphData.nodes[node]
      if (!n) return
      this.selectedNodeId = node
      this.selectedNode = n
      this.editedLabel = n.label ?? ""
      this.selectedNodeInfos = this._buildNodeInfos(node)
    },
    clearNodeInfos() {
      this.selectedNodeInfos = []
      this.selectedNode = null
      this.selectedNodeId = null
      this.editedLabel = ""
    },
    async editNodeLabel() {
      if (!this.selectedNode || !this.selectedNodeId) return
      const newLabel = String(this.editedLabel ?? "").trim()

      // MAJ UI immédiate
      this.selectedNode.label = newLabel
      this.configs.node.label.text = (node: NodeDataBase) => node.label || node.name
      this.selectedNodeInfos = this._buildNodeInfos(this.selectedNodeId)

      // Appel backend avec mac/ip/label
      try {
        this.isSavingLabel = true
        await invoke("add_label", {
          mac: this.selectedNode.mac ?? "",
          ip: this.selectedNode.ip ?? "",
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
      const n = this.graphData.nodes[nodeId] as any
      if (!n) return ["Nœud introuvable"]

      let degree = 0
      const protos = new Set<string>()
      for (const e of Object.values(this.graphData.edges) as any[]) {
        if (!e) continue
        if (e.source === nodeId || e.target === nodeId) {
          degree++
          if (e.label) protos.add(String(e.label))
        }
      }

      return [
        `ID: ${n.id}`,
        `Nom: ${n.name ?? ""}`,
        `Label: ${n.label ?? "N/A"}`,
        `MAC: ${n.mac ?? ""}`,
        `IP: ${n.ip ?? ""}`,            // ← NEW (affichage)
        `Couleur: ${n.color}`,
        `Degré: ${degree}`,
        `Protocoles: ${[...protos].join(", ") || "—"}`,
      ]
    },

    // === Force Layout ======================================================
    toggleForce() {
      if (this.forceEnabled) {
        const lh: any = (this.configs.view as any).layoutHandler
        if (isFn(lh, "stop")) lh.stop()
        ;(this.configs.view as any).layoutHandler = this.simpleLayout
      } else {
        (this.configs.view as any).layoutHandler = this.forceLayout
        if (isFn(this.forceLayout, "start")) this.forceLayout.start()
      }
      this.forceEnabled = !this.forceEnabled
    },

    // === Export SVG ========================================================
    async downloadSvg() {
      const filePath = await save({
        filters: [{ name: "SVG File", extensions: ["svg"] }],
        defaultPath: "network-graph.svg",
      })
      if (!filePath) return
      const vng = (this.$refs as any).graphnodes
      const text = await vng.exportAsSvgText({ embedImages: true })
      await writeTextFile(filePath, text)
      console.log(`SVG exporté dans ${filePath}`)
    },

    // === Queue & updates ===================================================
    normalizeGraphUpdate(raw: any): GraphUpdate | null {
      const u = raw?.update ?? raw
      if (!u) return null
      if (u.type && "payload" in u) return u as GraphUpdate
      if (u.NewNode) return { type: "NodeAdded", payload: u.NewNode }
      if (u.NewEdge) return { type: "EdgeAdded", payload: u.NewEdge }
      if (u.EdgeUpdated) return { type: "EdgeUpdated", payload: u.EdgeUpdated }
      return null
    },
    flushQueue() {
      const q = this._queue
      if (!q.length) return
      for (let i = 0; i < q.length; i++) this.applyUpdate(q[i])
      this._queue.length = 0

      if (this._pendingEdges.length) {
        const pend = this._pendingEdges.slice()
        this._pendingEdges.length = 0
        for (const u of pend) this.applyUpdate(u)
      }
    },
    applyUpdate(update: GraphUpdate | any) {
      if (!update) return
      const u = this.normalizeGraphUpdate(update)
      if (!u) return

      switch (u.type) {
        case "NodeAdded": {
          const node = u.payload
          if (node) {
            const color = node.color || "#2196F3"
            this.graphData.nodes[node.id] = {
              id: node.id,
              name: node.name,
              mac: node.mac || "",
              ip: node.ip || "",          // ← NEW (propagation)
              color,
              label: node.label || "",
              _stroke: darken(color, 0.25),
              _hover: brighten(color, 0.18),
            }
          }
          break
        }
        case "EdgeAdded": {
          const e = u.payload
          if (!this.graphData.nodes[e.source] || !this.graphData.nodes[e.target]) return
          const key = edgeKey(e)
          const _color = colorForLabel(e.label)
          this.graphData.edges[key] = { ...e, bidir: !!e.bidir, _color }
          break
        }
        case "EdgeUpdated": {
          const e = u.payload
          if (!this.graphData.nodes[e.source] || !this.graphData.nodes[e.target]) return
          const key = edgeKey(e)
          const existing = this.graphData.edges[key]
          const _color = colorForLabel(e.label)
          if (existing) {
            existing.bidir = !!e.bidir
            ;(existing as any)._color = _color
          } else {
            this.graphData.edges[key] = { ...e, bidir: !!e.bidir, _color }
          }
          break
        }
      }
    },
  },
})
</script>

<template>
  <div class="graph-container">
    <!-- Boutons d’action -->
    <div class="top-buttons">
      <button class="download-button" @click="downloadSvg" title="Exporter en SVG">⬇️ Export SVG</button>
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
    <v-network-graph
      class="graph"
      ref="graphnodes"
      v-model:zoom-level="zoomLevel"
      :nodes="graphNodes"
      :edges="graphEdges"
      :layouts="graphData.layouts"
      :configs="configs"
      :event-handlers="eventHandlers"
    >
      <template #edge-label="slotProps">
        <v-edge-label
          v-if="zoomLevel >= 1.2"
          :text="slotProps.edge.label"
          align="center"
          vertical-align="above"
          v-bind="slotProps"
          :font-size="18 * slotProps.scale"
          fill="#FFFFFF"
        />
        <v-edge-label
          v-if="zoomLevel >= 1.8"
          :text="`${slotProps.edge.source_port ?? ''}`"
          align="source"
          vertical-align="below"
          v-bind="slotProps"
          :font-size="14 * slotProps.scale"
          fill="#E0E0E0"
        />
        <v-edge-label
          v-if="zoomLevel >= 1.8"
          :text="`${slotProps.edge.destination_port ?? ''}`"
          align="target"
          vertical-align="below"
          v-bind="slotProps"
          :font-size="14 * slotProps.scale"
          fill="#E0E0E0"
        />
      </template>
    </v-network-graph>

    <!-- Bandeau d'infos en bas -->
    <div class="bottom-info">
      <div class="zoom">Zoom: {{ zoomLevel.toPrecision(2) }}</div>
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
          <li v-for="(info, idx) in selectedNodeInfos" :key="idx">{{ info }}</li>
        </ul>
      </div>
      <div class="node-infos hint" v-else>
        Clique un nœud pour afficher ses informations.
      </div>
    </div>
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
.graph { flex: 1; background: #000; }

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
