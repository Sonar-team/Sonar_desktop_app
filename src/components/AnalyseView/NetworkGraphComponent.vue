<script lang="ts">
import { defineComponent, markRaw, shallowReactive } from "vue"
import { VNetworkGraph, VEdgeLabel } from "v-network-graph"
import * as vNG from "v-network-graph"
import { ForceLayout } from "v-network-graph/lib/force-layout"
import { useCaptureStore } from "../../store/capture"

// --- Types -----------------------------------------------------------------

type NodeId = string
type EdgeId = string

interface NodeDataBase {
  id: string
  name: string
  mac?: string
  color: string
  _hover?: string
  _stroke?: string
}

interface EdgeData {
  source: NodeId
  target: NodeId
  label: string
  source_port?: string | number | null
  destination_port?: string | number | null
  bidir?: boolean           // deux sens observés ?
}

type GraphUpdate =
  | { type: "NodeAdded"; payload: any }
  | { type: "EdgeAdded"; payload: any }
  | { type: "EdgeUpdated"; payload: any }

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

// Small helpers --------------------------------------------------------------
function clamp01(x: number) { return x < 0 ? 0 : x > 1 ? 1 : x }
function hexToRgb(hex: string) {
  const h = hex.startsWith('#') ? hex.slice(1) : hex
  const v = parseInt(h.length === 3 ? h.replace(/(.)/g, '$1$1') : h, 16)
  return { r: (v >> 16) & 255, g: (v >> 8) & 255, b: v & 255 }
}
function rgbToHex(r: number, g: number, b: number) {
  return '#' + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)
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
    (clamp01(b / 255 + factor) * 255) | 0,
  )
}

// clé d'edge stable = (source,target,label)
function edgeKey(e: EdgeData): EdgeId {
  return `${e.source}\u001F${e.target}\u001F${e.label}`
}

function clearReactiveMap<T extends Record<string, any>>(obj: T) {
  for (const k of Object.keys(obj)) delete obj[k]
}
function isFn(x: any, name: string): x is (...a: any[]) => void {
  return x && typeof x[name] === "function"
}

export default defineComponent({
  name: "NetworkGraphComponent",
  components: { VNetworkGraph, VEdgeLabel },

  data() {
    const force = markRaw(new ForceLayout({}))

    return {
      graphData: {
        nodes: shallowReactive(Object.create(null) as Record<NodeId, NodeDataBase>),
        edges: shallowReactive(Object.create(null) as Record<EdgeId, EdgeData>),
        layouts: markRaw({}) as Record<string, unknown>,
      },

      menuTargetNode: [] as string[],
      menuTargetEdges: [] as string[],

      _queue: [] as GraphUpdate[],
      _pendingEdges: [] as GraphUpdate[],   // edges en attente des nœuds
      _raf: 0 as number,

      configs: markRaw(
        vNG.defineConfigs({
          view: { maxZoomLevel: 5, minZoomLevel: 0.1, layoutHandler: force },
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
            label: { fontSize: 16, color: "#ffffff", direction: "north" as const },
          },
          edge: {
            type: "curve",
            gap: 30,
            selectable: true,
            normal: {
              width: 2,
              color: (edge: EdgeData) => colorForLabel(edge.label),
            },
            // Markers: champs dynamiques (pas un objet retourné)
            marker: {
              source: {
                type: (edge: EdgeData) => (edge?.bidir ? "arrow" : "none"),
                width: 6, height: 6, margin: 0, offset: 0,
                units: "strokeWidth" as const, color: null,
              },
              target: {
                type: "arrow" as const,
                width: 6, height: 6, margin: 0, offset: 0,
                units: "strokeWidth" as const, color: null,
              },
            },
            label: {
              fontSize: 21,
              lineHeight: 1.1,
              color: "#E0E0E0",
              margin: 4,
              background: { visible: true, color: "#000000", padding: { vertical: 1, horizontal: 4 }, borderRadius: 2 },
            },
          },
        })
      ),
    }
  },

  computed: {
    captureStore() {
      return useCaptureStore()
    },
    graphNodes(): Record<NodeId, NodeDataBase> { return this.graphData.nodes },
    graphEdges(): Record<EdgeId, EdgeData> { return this.graphData.edges },
  },

  mounted() {
    clearReactiveMap(this.graphData.nodes)
    clearReactiveMap(this.graphData.edges)

    // abonnement aux updates backend (NodeAdded, EdgeAdded, EdgeUpdated)
    this.captureStore.onGraphUpdate((update: GraphUpdate) => {
      this._queue.push(update)
      if (!this._raf) {
        this._raf = requestAnimationFrame(() => {
          this.flushQueue()
          this._raf = 0
        })
      }
    })

    // reset containers (au cas où)
    this.graphData.nodes = shallowReactive(Object.create(null))
    this.graphData.edges = shallowReactive(Object.create(null))

    this.$bus.on('reset', () => this.resetGraph())
  },

  beforeUnmount() {
    if (this._raf) cancelAnimationFrame(this._raf)
  },

  methods: {
    resetGraph() {
      if (this._raf) { cancelAnimationFrame(this._raf); this._raf = 0 }
      this._queue.length = 0
      this._pendingEdges.length = 0

      clearReactiveMap(this.graphData.nodes)
      clearReactiveMap(this.graphData.edges)
      clearReactiveMap(this.graphData.layouts)

      const lh: any = this.configs.view?.layoutHandler
      if (isFn(lh, "stop")) lh.stop()
      if (isFn(lh, "reset")) lh.reset()
      if (isFn(lh, "start")) lh.start()
      if (isFn(this.$refs.graphnodes, "fitToContents")) (this.$refs as any).graphnodes.fitToContents()
    },

    // tolère { type, payload } ou { NewNode: {...} } / { NewEdge: {...} } / { EdgeUpdated: {...} }
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

      // essayer de vider les arêtes en attente
      if (this._pendingEdges.length) {
        const pend = this._pendingEdges.slice()
        this._pendingEdges.length = 0
        for (const u of pend) this.applyUpdate(u)
      }
    },

    applyUpdate(update: GraphUpdate | any) {
      if (!update) return
      const u = this.normalizeGraphUpdate(update)
      if (!u) {
        console.warn("[NetworkGraph] Unrecognized GraphUpdate shape:", update)
        return
      }

      switch (u.type) {
        case "NodeAdded": {
          const node = u.payload
          if (node) {
            const color = node.color || "#2196F3"
            this.graphData.nodes[node.id] = {
              id: node.id,
              name: node.name,
              mac: node.mac || "",
              color,
              _stroke: darken(color, 0.25),
              _hover: brighten(color, 0.18),
            }
          }
          break
        }

        case "EdgeAdded": {
          const e = u.payload
          if (e) {
            // garde-fou: attendre que les nœuds existent
            if (!this.graphData.nodes[e.source] || !this.graphData.nodes[e.target]) {
              this._pendingEdges.push(u)
              return
            }
            const key = edgeKey({
              source: e.source, target: e.target, label: e.label,
              source_port: e.source_port ?? null, destination_port: e.destination_port ?? null,
              bidir: !!e.bidir,
            })
            this.graphData.edges[key] = {
              source: e.source,
              target: e.target,
              label: e.label,
              source_port: e.source_port ?? null,
              destination_port: e.destination_port ?? null,
              bidir: !!e.bidir,
            }
          }
          break
        }

        case "EdgeUpdated": {
          const e = u.payload
          if (e) {
            if (!this.graphData.nodes[e.source] || !this.graphData.nodes[e.target]) {
              this._pendingEdges.push(u)
              return
            }
            const key = edgeKey({
              source: e.source, target: e.target, label: e.label,
              source_port: e.source_port ?? null, destination_port: e.destination_port ?? null,
              bidir: !!e.bidir,
            })
            const existing = this.graphData.edges[key]
            if (existing) {
              existing.bidir = !!e.bidir
              // si besoin de maj ports/label:
              // existing.source_port = e.source_port ?? existing.source_port
              // existing.destination_port = e.destination_port ?? existing.destination_port
              // existing.label = e.label ?? existing.label
            } else {
              this.graphData.edges[key] = {
                source: e.source,
                target: e.target,
                label: e.label,
                source_port: e.source_port ?? null,
                destination_port: e.destination_port ?? null,
                bidir: !!e.bidir,
              }
            }
          }
          break
        }

        default:
          console.warn("Unknown update type:", u)
          break
      }
    },
  },
})
</script>

<template>
  <div class="graph-container">
    <v-network-graph
      class="graph"
      ref="graphnodes"
      :zoom-level="3"
      :nodes="graphNodes"
      :edges="graphEdges"
      :layouts="graphData.layouts"
      :configs="configs"
    >
      <template #edge-label="slotProps">
        <v-edge-label
          :text="slotProps.edge.label"
          align="center"
          vertical-align="above"
          v-bind="slotProps"
          :font-size="18 * slotProps.scale"
          fill="#FFFFFF"
        />
        <v-edge-label
          :text="`${slotProps.edge.source_port ?? ''}`"
          align="source"
          vertical-align="below"
          v-bind="slotProps"
          :font-size="14 * slotProps.scale"
          fill="#E0E0E0"
        />
        <v-edge-label
          :text="`${slotProps.edge.destination_port ?? ''}`"
          align="target"
          vertical-align="below"
          v-bind="slotProps"
          :font-size="14 * slotProps.scale"
          fill="#E0E0E0"
        />
      </template>
    </v-network-graph>
  </div>

  <!-- Context menus -->
  <div ref="nodeMenu" class="context-menu">
    Infos du noeud:
    <ul class="contenu">
      <li v-for="(info, index) in menuTargetNode" :key="index">{{ info }}</li>
    </ul>
  </div>
  <div ref="edgeMenu" class="context-menu">
    Infos de l'arête:
    <div class="contenu">{{ menuTargetEdges.join(", ") }}</div>
  </div>
</template>

<style scoped>
.graph-container { position: relative; flex: 1; display: flex; flex-direction: column; width: 100%; overflow: hidden; background-color: #1a1a1a; height: 100%; }
.graph { flex: 1; width: 100%; text-align: center; color: #FFF; background-color: #000000; }
.download-button { position: absolute; top: 10px; left: 10px; background-color: #0b1b25; color: #fff; padding: 10px 20px; border: none; border-radius: 5px; cursor: pointer; z-index: 10; }
.context-menu { color: #0b1b25; border-radius: 10px; width: 220px; background-color: #efefef; padding: 10px; position: absolute; visibility: hidden; font-size: 12px; border: 1px solid #aaaaaa; box-shadow: 2px 2px 2px #e7bf0c; z-index: 50; }
.contenu { color: #0b1b25; border: 1px dashed #aaa; margin-top: 8px; padding: 6px; word-break: break-word; }
</style>
