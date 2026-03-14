<script lang="ts">
import { defineComponent, shallowReactive, markRaw, reactive } from "vue"
import { VNetworkGraph, VEdgeLabel } from "v-network-graph"
import * as vNG from "v-network-graph"
import { ForceLayout } from "v-network-graph/lib/force-layout"
import { useCaptureStore } from "../../store/capture"
import { save } from "@tauri-apps/plugin-dialog"
import { writeTextFile, writeFile } from "@tauri-apps/plugin-fs"
import { EdgeData, EdgeId, GraphData, GraphUpdate, NodeData, NodeId } from "../../types/capture"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentDate } from '../../utils/time';
import LegendComponent from './LegendComponent.vue';

// --- Colors ----------------------------------------------------------------
const EDGE_COLORS_LC: Record<string, string> = Object.freeze({
  arp: "#a89040",
  ipv4: "#a07030",
  ipv6: "#8a5f8a",
  profinet_rt: "#3f7040",
  tls: "#3a5c9a",
  dns: "#8a3c3c",
  ntp: "#a07030",
})
const colorForLabel = (label: string) =>
  EDGE_COLORS_LC[label?.toLowerCase?.() ?? ""] || "#808095"

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

async function svgTextToPngBytes(svgText: string, opts?: { scale?: number; background?: "transparent" | "white" }) {
  const scale = opts?.scale ?? 1
  const background = opts?.background ?? "transparent"

  // 1) Base64-encode du SVG (safe)
  const svgBase64 = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(svgText)))

  // 2) Crée l'image
  const img = new Image()
  img.decoding = "async"
  img.src = svgBase64

  await new Promise<void>((resolve, reject) => {
    img.onload = () => resolve()
    img.onerror = (e) => reject(e)
  })

  // 3) Déterminer la taille via width/height OU viewBox
  let w = img.width
  let h = img.height

  if (!w || !h) {
    const m = svgText.match(/viewBox="([\d.\-eE]+)\s+([\d.\-eE]+)\s+([\d.\-eE]+)\s+([\d.\-eE]+)"/i)
    if (m) {
      const vbW = parseFloat(m[3]), vbH = parseFloat(m[4])
      w = Math.max(1, Math.round(vbW))
      h = Math.max(1, Math.round(vbH))
    } else {
      // fallback
      w = 1920; h = 1080
    }
  }

  const canvas = document.createElement("canvas")
  canvas.width = Math.max(1, Math.floor(w * scale))
  canvas.height = Math.max(1, Math.floor(h * scale))
  const ctx = canvas.getContext("2d")!

  if (background === "white") {
    ctx.fillStyle = "#ffffff"
    ctx.fillRect(0, 0, canvas.width, canvas.height)
  }

  // 4) Dessin
  ctx.drawImage(img, 0, 0, canvas.width, canvas.height)

  // 5) Canvas → PNG (Uint8Array)
  const blob: Blob = await new Promise((res) => canvas.toBlob(b => res(b!), "image/png"))
  const ab = await blob.arrayBuffer()
  return new Uint8Array(ab)
}

// --- Component -------------------------------------------------------------
export default defineComponent({
  name: "NetworkGraphComponent",
  components: { VNetworkGraph, VEdgeLabel, LegendComponent },

  data() {
    const forceLayout = markRaw(new ForceLayout({}))
    const simpleLayout = markRaw(new vNG.SimpleLayout())

    const configs = reactive(
      vNG.defineConfigs({
        view: { 
          maxZoomLevel: 5, 
          minZoomLevel: 0.1, 
          layoutHandler: forceLayout, 
          scalingObjects: false 
        },
        node: {
          selectable: true,
          normal: {
            radius: 20,
            color: (node: NodeData) => node.color,
            strokeWidth: 3,
            strokeColor: (node: NodeData) => node._stroke ?? darken(node.color, 0.25),
          },
          hover: {
            radius: 20,
            color: (node: NodeData) => node._hover ?? brighten(node.color, 0.18),
          },
          label: {
            text: (node: NodeData) => node.label || node.name,
            fontSize: 10,
            color: "#c0c0c8",
            direction: "north" as const,
            lineHeight: 2.0,
            margin: 4,
            background: {
               visible: true,
               color: "#1e1e28",
               padding: { vertical: 3, horizontal: 6 },
               borderRadius: 2,
            },
          }
        },
        
        edge: {
          type: "straight",
          gap: 10,
          selectable: true,
          normal: {
            width: 2,
            color: (edge: any) => edge._color ?? colorForLabel(edge.label),
          },
          marker: {
            source: { type: "none", width: 5, height: 5, margin: 0, offset: 0, units: "strokeWidth" as const, color: null },
            target: { type: "arrow" as const, width: 5, height: 5, margin: 0, offset: 0, units: "strokeWidth" as const, color: null },
          },
          label: {
            fontSize: 18,
            lineHeight: 1.1,
            color: "#9a9aa8",
            margin: 4,
              background: {
                visible: true,
                color: "#1e1e28",
                padding: {
                  vertical: 1,
                  horizontal: 4
                },
                borderRadius: 2 },
          },
        },
      })
    )

    return {
      graphData: {
        nodes: shallowReactive(Object.create(null) as Record<NodeId, NodeData>),
        edges: shallowReactive(Object.create(null) as Record<EdgeId, EdgeData>),
        layouts: reactive({}) as Record<string, unknown>,
      },

      forceEnabled: true,
      zoomLevel: 1,

      forceLayout,
      simpleLayout,
      configs,

      // Bandeau bas
      selectedNodeInfos: [] as string[],
      selectedNode: null as NodeData | null,
      selectedNodeId: null as string | null,
      editedLabel: "" as string,
      isSavingLabel: false as boolean,

      // Queue
      _queue: [] as GraphUpdate[],
      _pendingEdges: [] as GraphUpdate[],
      _raf: 0 as number,

      // Handlers pour cleanup
      resetHandler: null as (() => void) | null,
    }
  },

  computed: {
    captureStore() { return useCaptureStore() },
    graphNodes(): Record<NodeId, NodeData> { return this.graphData.nodes },
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

    this.captureStore.onGraphSnapshot((graphData) => {
      console.log("[NetworkGraphComponent] GraphSnapshot reçu -> reload");
      this.loadFromGraphData(graphData);
    });

    // Abonnement au reset via le bus global
    this.resetHandler = () => this.resetGraph()
    this.$bus?.on?.('reset', this.resetHandler)

    if (this.forceEnabled && isFn(this.forceLayout, "start")) this.forceLayout.start()
  },

  methods: {
    async printLabels() {
      await invoke('get_label_list').then((labels: any) => {
        console.log(labels)
      })
    },
    // === Réinitialisation ==================================================
    resetGraph() {
      clearReactiveMap(this.graphData.nodes)
      clearReactiveMap(this.graphData.edges)
      this.clearNodeInfos()
    },
    /**
     * Recharge complètement le graphe à partir d'un snapshot complet
     * envoyé par le backend (GraphSnapshot).
     */
async loadFromGraphData(snapshot: GraphData | null | undefined) {
  console.log("[NetworkGraphComponent] GraphSnapshot reçu -> ", snapshot);

  try {
    // 1. Input validation
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

    // 2. Reset
    clearReactiveMap(this.graphData.nodes);
    clearReactiveMap(this.graphData.edges);
    this.clearNodeInfos();

    // 3. Process nodes
    const nodeEntries = Object.entries(snapshot.nodes || {});
    console.log(`[NetworkGraphComponent] Chargement de ${nodeEntries.length} nœuds`);

    for (const [nodeId, node] of nodeEntries) {
      if (!node) continue;

      try {
        const color = node.color || "#4a6aa0";
        const id = node.id || nodeId;

        this.graphData.nodes[id] = {
          id,
          name: node.name || id,
          mac: node.mac || "",
          ip: node.ip || "",
          color,
          label: node.label || "",
          _stroke: darken(color, 0.25),
          _hover: brighten(color, 0.18),
        };
      } catch (error) {
        console.error(`[NetworkGraphComponent] Erreur lors du chargement du nœud ${nodeId}:`, error);
      }
    }

    // 4. Process edges
    const edgeEntries = Object.entries(snapshot.edges || {});
    console.log(`[NetworkGraphComponent] Chargement de ${edgeEntries.length} arêtes`);

    for (const [edgeId, edge] of edgeEntries) {
      if (!edge) continue;

      try {
        const source = edge.source;
        const target = edge.target;
        const label = edge.label || "";

        if (!source || !target) {
          console.warn(`[NetworkGraphComponent] Arête ${edgeId} invalide: source ou target manquante`);
          continue;
        }

        if (!this.graphData.nodes[source] || !this.graphData.nodes[target]) {
          console.warn(`[NetworkGraphComponent] Arête orpheline ignorée: ${source} -> ${target} (${label})`);
          continue;
        }

        const _color = edge._color || colorForLabel(label);
        const key = edgeKey({
          source,
          target,
          label,
          source_port: edge.source_port,
          destination_port: edge.destination_port
        } as EdgeData);

        this.graphData.edges[key] = {
          source,
          target,
          label,
          source_port: edge.source_port ?? null,
          destination_port: edge.destination_port ?? null,
          bidir: edge.bidir || false,
          _color,
        };
      } catch (error) {
        console.error(`[NetworkGraphComponent] Erreur lors du chargement de l'arête ${edgeId}:`, error);
      }
    }

    // 5. Update layout if needed
    if (this.forceEnabled && this.forceLayout?.start) {
      try {
        this.forceLayout.start();
      } catch (error) {
        console.error("[NetworkGraphComponent] Erreur lors du démarrage du layout:", error);
      }
    }

  } catch (error) {
    console.error("[NetworkGraphComponent] Erreur critique dans loadFromGraphData:", error);
  }
}
,

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
      this.configs.node.label.text = (node: NodeData) => node.label || node.name
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
          defaultPath: getCurrentDate()+ "_network_graph_DR_Matrice.svg",
        })
        if (!filePath) return
        const vng = (this.$refs as any).graphnodes
        const text = await vng.exportAsSvgText({ embedImages: true })
        await writeTextFile(filePath, text)
        console.log(`SVG exporté dans ${filePath}`)
      },
      async downloadPng() {
      const filePath = await save({
        filters: [{ name: "PNG File", extensions: ["png"] }],
        defaultPath: getCurrentDate()+ "_network_graph_DR_Matrice.png",
      })
      if (!filePath) return

      const vng = (this.$refs as any).graphnodes
      const svgText = await vng.exportAsSvgText({ embedImages: true })

      // Options : scale x2 et fond blanc (change selon besoin)
      const pngBytes = await svgTextToPngBytes(svgText, { scale: 2, background: "white" })

      await writeFile(filePath, pngBytes)
      console.log(`PNG exporté dans ${filePath}`)
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
            const color = node.color || "#4a6aa0"
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
    <div class="graph-controls">
      <!-- Export PNG -->
      <button class="graph-btn" @click="downloadPng" title="Exporter en PNG">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
          <path d="M8 2v8M5 7l3 3 3-3"/>
          <path d="M3 12h10"/>
        </svg>
      </button>

      <!-- Gravité toggle -->
      <button
        class="graph-btn"
        :class="{ active: forceEnabled }"
        @click="toggleForce"
        :title="forceEnabled ? 'Désactiver la gravité' : 'Activer la gravité'"
      >
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="8" cy="5" r="2.5"/>
          <path d="M8 7.5v5"/>
          <path d="M5.5 10.5l2.5 2.5 2.5-2.5"/>
        </svg>
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
          fill="#9a9aa8"
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
      <button class="inline-btn" @click="printLabels" title="Afficher les labels">
        <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
          <circle cx="4" cy="7" r="1.5"/>
          <circle cx="7" cy="3.5" r="1.5"/>
          <circle cx="10" cy="7" r="1.5"/>
          <circle cx="7" cy="10.5" r="1.5"/>
        </svg>
        Labels
      </button>
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
          <button class="ghost" @click="clearNodeInfos" :disabled="isSavingLabel">Annuler</button>
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
  background: #282838;
  overflow: hidden;
}
.graph { flex: 1; background: #1e1e28; }

/* Contrôles flottants */
.graph-controls {
  position: absolute;
  top: 10px;
  left: 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 10;
}

.graph-btn {
  width: 28px;
  height: 28px;
  background: rgba(28, 28, 32, 0.75);
  border: 1px solid #3c3c50;
  border-radius: 5px;
  color: #585870;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.15s ease, background 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
  backdrop-filter: blur(4px);
}

.graph-btn svg {
  width: 14px;
  height: 14px;
}

.graph-btn:hover {
  color: #888894;
  background: rgba(30, 30, 34, 0.9);
  border-color: #484858;
  transform: translateY(-1px);
}
.graph-btn:active {
  transform: scale(0.9);
}

.graph-btn.active {
  color: #5a8a9a;
  border-color: #2a4a55;
  background: rgba(30, 50, 60, 0.6);
}

/* Bandeau bas */
.bottom-info {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 200px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: rgba(24, 24, 28, 0.92);
  color: #a0a0b4;
  border-top: 1px solid #484860;
  backdrop-filter: blur(4px);
  z-index: 20;
}
.bottom-info .zoom {
  font-variant-numeric: tabular-nums;
  font-family: monospace;
  font-size: 11px;
  color: #585870;
}
.bottom-info .sep {
  width: 1px;
  height: 16px;
  background: #3c3c50;
}

.inline-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  background: transparent;
  border: none;
  color: #585870;
  font-size: 11px;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 3px;
  transition: color 0.15s ease;
}
.inline-btn svg { width: 11px; height: 11px; }
.inline-btn:hover { color: #888894; }
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
.node-infos.hint { color: #8888a0; font-style: italic; font-size: 12px; }

.edit-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0 10px;
}
.edit-row input {
  background: #343448;
  color: #d4d4d8;
  border: 1px solid #545465;
  border-radius: 6px;
  padding: 6px 8px;
  min-width: 220px;
}
button.primary {
  background: #2e4a68;
  color: #d4d4d8;
  border: 1px solid #3a5a80;
  border-radius: 6px;
  padding: 6px 10px;
  cursor: pointer;
}
button.primary:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
button.ghost {
  background: transparent;
  color: #888894;
  border: 1px solid #484858;
  border-radius: 6px;
  padding: 6px 10px;
  cursor: pointer;
}
</style>
