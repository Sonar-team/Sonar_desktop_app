<script lang="ts">
import { defineComponent, markRaw } from "vue"
import Graph from "graphology"
import Sigma from "sigma"
import { EdgeArrowProgram } from "sigma/rendering"
import forceAtlas2 from "graphology-layout-forceatlas2"
import FA2Layout from "graphology-layout-forceatlas2/worker"
import { EdgeCurvedArrowProgram, indexParallelEdgesIndex, DEFAULT_EDGE_CURVATURE } from "@sigma/edge-curve"
import { NodeBorderProgram } from "@sigma/node-border"
import { toBlob } from "@sigma/export-image"
import { useCaptureStore } from "../../store/capture"
import { save } from "@tauri-apps/plugin-dialog"
import { writeFile } from "@tauri-apps/plugin-fs"
import { EdgeData, EdgeId, GraphData, GraphUpdate, NodeData } from "../../types/capture"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentDate } from '../../utils/time';
import LegendComponent from './LegendComponent.vue';
import { colorForProtocol } from "../../utils/protocolColors"

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

// Jitter autour d'un point d'ancrage pour les nouveaux nœuds (évite l'empilement)
function jitterAround(x: number, y: number, radius = 120) {
  const angle = Math.random() * 2 * Math.PI
  const r = radius * (0.4 + 0.6 * Math.random())
  return { x: x + Math.cos(angle) * r, y: y + Math.sin(angle) * r }
}

// Seuils de zoom pour l'affichage des labels d'arêtes (zoom = 1 / camera.ratio)
const EDGE_LABEL_ZOOM = 1.2
const PORT_LABEL_ZOOM = 1.8

// Tailles proportionnelles au trafic, en échelle log : les gros parleurs
// ressortent sans écraser les hôtes discrets.
const NODE_SIZE_MIN = 7
const NODE_SIZE_MAX = 18
function nodeSizeFor(bytes: number) {
  return Math.min(NODE_SIZE_MAX, NODE_SIZE_MIN + 1.1 * Math.log2(1 + bytes / 2000))
}
const EDGE_SIZE_MIN = 1.2
const EDGE_SIZE_MAX = 7
function edgeSizeFor(bytes: number) {
  return Math.min(EDGE_SIZE_MAX, EDGE_SIZE_MIN + 0.55 * Math.log2(1 + bytes / 1500))
}
function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} o`
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} ko`
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} Mo`
  return `${(bytes / 1024 ** 3).toFixed(2)} Go`
}

// Courbure des arêtes parallèles (repris de l'exemple officiel @sigma/edge-curve)
function getCurvature(index: number, maxIndex: number): number {
  if (maxIndex <= 0) return DEFAULT_EDGE_CURVATURE
  const amplitude = 3.5
  const maxCurvature = amplitude * (1 - Math.exp(-maxIndex / amplitude)) * DEFAULT_EDGE_CURVATURE
  return (maxCurvature * index) / maxIndex
}

// Label de nœud : texte blanc sur fond noir, au-dessus du nœud.
// Sert aussi au rendu du hover (fond blanc par défaut, illisible sur fond noir).
function drawNodeLabel(context: CanvasRenderingContext2D, data: any, settings: any) {
  if (!data.label) return
  const size = settings.labelSize
  context.font = `${settings.labelWeight} ${size}px ${settings.labelFont}`
  const width = context.measureText(data.label).width + 10
  const x = data.x - width / 2
  const y = data.y - data.size - size - 8
  context.fillStyle = "#000000CC"
  context.fillRect(x, y, width, size + 6)
  context.fillStyle = "#ffffff"
  context.fillText(data.label, x + 5, y + size)
}

function nodeAttributes(node: any) {
  const color = node.color || "#2196F3"
  const rawLabel = node.label || ""
  return {
    name: node.name || node.id,
    mac: node.mac || "",
    ip: node.ip || "",
    rawLabel,
    label: rawLabel || node.name || node.id,
    color,
    borderColor: darken(color, 0.25),
    hoverColor: brighten(color, 0.18),
  }
}

function edgeAttributes(e: any) {
  const totalBytes = Number(e.total_bytes) || 0
  return {
    protocol: e.label || "",
    source_port: e.source_port ?? null,
    destination_port: e.destination_port ?? null,
    bidir: !!e.bidir,
    count: Number(e.count) || 0,
    total_bytes: totalBytes,
    // Tunnels (encap_id hex) auxquels ce flux participe : le backend envoie
    // la liste cumulative, on remplace donc simplement à chaque update.
    encapIds: Array.isArray(e.encap_ids) ? e.encap_ids : [],
    color: e._color || colorForProtocol(e.label || ""),
    size: edgeSizeFor(totalBytes),
  }
}

// Couleurs d'estompage pendant la surbrillance d'un tunnel (fond noir)
const DIM_EDGE_COLOR = "#2a2a2a"
const DIM_NODE_COLOR = "#3a3a3a"

// --- Component -------------------------------------------------------------
export default defineComponent({
  name: "NetworkGraphComponent",
  components: { LegendComponent },

  data() {
    return {
      graph: null as Graph | null,
      renderer: null as Sigma | null,
      layout: null as InstanceType<typeof FA2Layout> | null,

      forceEnabled: true,
      zoomLevel: 1,
      hoveredNode: null as string | null,

      // Surbrillance de la famille d'un tunnel (survol d'une arête) :
      // arêtes/nœuds partageant un encap_id avec l'arête survolée.
      hoveredTunnelEdges: null as Set<string> | null,
      hoveredTunnelNodes: null as Set<string> | null,
      tunnelHoverInfo: "" as string,
      // Arête épinglée au clic : la surbrillance reste après le survol
      // (re-clic sur l'arête ou clic sur le fond pour libérer).
      pinnedTunnelEdge: null as string | null,

      // Bandeau bas
      selectedNodeInfos: [] as string[],
      selectedNode: null as NodeData | null,
      selectedNodeId: null as string | null,
      editedLabel: "" as string,
      isSavingLabel: false as boolean,

      // Panneau "Afficher les labels"
      showLabelsPanel: false as boolean,
      matrixLabels: [] as [string, string, string][],
      labelsSearch: "" as string,

      // Queue
      _queue: [] as GraphUpdate[],
      _raf: 0 as number,

      // Affichage des labels d'arêtes selon le zoom
      _edgeLabelsShown: false,
      _portLabelsShown: false,

      // Taille du graphe lors du dernier inferSettings du layout
      _layoutOrder: 0,

      // Handlers pour cleanup
      resetHandler: null as (() => void) | null,
      graphUnsubs: [] as Array<() => void>,
    }
  },

  computed: {
    captureStore() { return useCaptureStore() },
    filteredMatrixLabels(): [string, string, string][] {
      const q = this.labelsSearch.trim().toLowerCase()
      if (!q) return this.matrixLabels
      return this.matrixLabels.filter((row) =>
        row.some((field) => field.toLowerCase().includes(q))
      )
    },
  },

  mounted() {
    this.initSigma()

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

    this.startLayout()
    if (!this.forceEnabled) this.layout?.stop()
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

    if (this.resetHandler) {
      this.$bus?.off?.('reset', this.resetHandler)
      this.resetHandler = null
    }

    if (this.layout) {
      try { this.layout.kill() } catch {}
      this.layout = null
    }
    if (this.renderer) {
      this.renderer.kill()
      this.renderer = null
    }
    this.graph = null
  },

  methods: {
    async printLabels() {
      try {
        this.matrixLabels = await invoke<[string, string, string][]>('get_matrix_labels')
      } catch (e) {
        console.error("Erreur get_matrix_labels:", e)
        this.matrixLabels = []
      }
      this.labelsSearch = ""
      this.showLabelsPanel = true
    },

    closeLabelsPanel() {
      this.showLabelsPanel = false
    },

    // === Initialisation Sigma ==============================================
    initSigma() {
      const container = this.$refs.sigmaContainer as HTMLElement
      const graph = new Graph({ multi: true, type: "directed" })
      this.graph = markRaw(graph)

      const renderer = new Sigma(graph, container, {
        allowInvalidContainer: true,
        // zoom = 1 / ratio : bornes équivalentes à l'ancien minZoom 0.1 / maxZoom 5
        minCameraRatio: 0.2,
        maxCameraRatio: 10,
        defaultNodeType: "bordered",
        defaultEdgeType: "straight",
        nodeProgramClasses: { bordered: NodeBorderProgram },
        edgeProgramClasses: { straight: EdgeArrowProgram, curved: EdgeCurvedArrowProgram },
        renderEdgeLabels: true,
        labelSize: 13,
        labelColor: { color: "#ffffff" },
        edgeLabelSize: 11,
        edgeLabelColor: { color: "#E0E0E0" },
        defaultDrawNodeLabel: drawNodeLabel,
        defaultDrawNodeHover: drawNodeLabel,
        // Survol d'arête (surbrillance des tunnels) + zIndex pour faire
        // passer la famille surlignée au-dessus des arêtes estompées.
        enableEdgeEvents: true,
        zIndex: true,
        nodeReducer: (node, data) => {
          const res: any = { ...data }
          const tunnelNodes = this.hoveredTunnelNodes
          if (tunnelNodes && !tunnelNodes.has(node)) {
            res.color = DIM_NODE_COLOR
            res.label = null
          }
          if (node === this.hoveredNode) res.color = data.hoverColor ?? res.color
          if (node === this.selectedNodeId) res.highlighted = true
          return res
        },
        edgeReducer: (edge, data) => {
          const res: any = { ...data }
          const tunnelEdges = this.hoveredTunnelEdges
          const dimmed = !!tunnelEdges && !tunnelEdges.has(edge)
          if (tunnelEdges) {
            if (dimmed) {
              res.color = DIM_EDGE_COLOR
              res.zIndex = 0
            } else {
              res.zIndex = 1
              res.size = (data.size || 1) + 1
            }
          }
          if (!this._edgeLabelsShown || dimmed) {
            res.label = null
            return res
          }
          let label = data.protocol ?? ""
          if (this._portLabelsShown && (data.source_port != null || data.destination_port != null)) {
            label += ` ${data.source_port ?? ""}→${data.destination_port ?? ""}`
          }
          res.label = label
          return res
        },
      })
      this.renderer = markRaw(renderer)

      renderer.on("clickNode", ({ node }) => this.onNodeClick(node))
      renderer.on("clickStage", () => { this.clearNodeInfos(); this.unpinTunnelHighlight() })
      renderer.on("enterNode", ({ node }) => { this.hoveredNode = node; renderer.refresh() })
      renderer.on("leaveNode", () => { this.hoveredNode = null; renderer.refresh() })
      renderer.on("enterEdge", ({ edge }) => this.onTunnelEdgeEnter(edge))
      renderer.on("leaveEdge", () => this.clearTunnelHighlight())
      renderer.on("clickEdge", ({ edge }) => this.onTunnelEdgeClick(edge))

      renderer.getCamera().on("updated", (state) => this.onZoom(1 / state.ratio))
    },

    // === Surbrillance des tunnels ==========================================
    /**
     * Surligne la famille de tunnel(s) d'une arête : si elle participe à un
     * ou plusieurs tunnels (encapIds non vide), la (les) arête(s) externe(s)
     * du tunnel ET les flux internes qu'il transporte passent au premier
     * plan, le reste du graphe est estompé. Fonctionne dans les deux sens :
     * le CAPWAP montre son contenu, un flux interne montre par quel(s)
     * tunnel(s) il est passé. Retourne true si une famille a été surlignée.
     */
    applyTunnelHighlight(edgeId: string): boolean {
      if (!this.graph?.hasEdge(edgeId)) return false
      const ids: string[] = this.graph.getEdgeAttribute(edgeId, "encapIds") || []
      if (!ids.length) return false

      const wanted = new Set(ids)
      const edges = new Set<string>()
      const nodes = new Set<string>()
      this.graph.forEachEdge((edge, attrs, source, target) => {
        const edgeIds: string[] = attrs.encapIds || []
        if (!edgeIds.some((id) => wanted.has(id))) return
        edges.add(edge)
        nodes.add(source)
        nodes.add(target)
      })

      this.hoveredTunnelEdges = edges
      this.hoveredTunnelNodes = nodes
      const idLabel = ids.length === 1 ? `tunnel ${ids[0].slice(0, 8)}…` : `${ids.length} tunnels`
      const pin = this.pinnedTunnelEdge === edgeId ? "📌 " : ""
      this.tunnelHoverInfo = `${pin}${idLabel} — ${edges.size} flux liés`
      this.renderer?.refresh()
      return true
    },

    onTunnelEdgeEnter(edgeId: string) {
      if (!this.applyTunnelHighlight(edgeId)) this.clearTunnelHighlight()
    },

    /** Clic sur une arête : épingle sa famille de tunnel (re-clic = libère). */
    onTunnelEdgeClick(edgeId: string) {
      if (this.pinnedTunnelEdge === edgeId) {
        this.pinnedTunnelEdge = null
        this.clearTunnelHighlight()
        return
      }
      if (this.graph?.hasEdge(edgeId) && (this.graph.getEdgeAttribute(edgeId, "encapIds") || []).length) {
        this.pinnedTunnelEdge = edgeId
        this.applyTunnelHighlight(edgeId)
      } else {
        this.pinnedTunnelEdge = null
        this.clearTunnelHighlight()
      }
    },

    /** Fin de survol : revient à la famille épinglée s'il y en a une, sinon efface. */
    clearTunnelHighlight() {
      if (this.pinnedTunnelEdge && this.applyTunnelHighlight(this.pinnedTunnelEdge)) return
      this.pinnedTunnelEdge = null
      if (!this.hoveredTunnelEdges) return
      this.hoveredTunnelEdges = null
      this.hoveredTunnelNodes = null
      this.tunnelHoverInfo = ""
      this.renderer?.refresh()
    },

    /** Efface tout, épinglage compris (clic sur le fond, reset du graphe). */
    unpinTunnelHighlight() {
      this.pinnedTunnelEdge = null
      this.clearTunnelHighlight()
    },

    onZoom(zoom: number) {
      this.zoomLevel = zoom

      let changed = false
      const showLabels = zoom >= EDGE_LABEL_ZOOM
      if (showLabels !== this._edgeLabelsShown) {
        this._edgeLabelsShown = showLabels
        changed = true
      }
      const showPorts = zoom >= PORT_LABEL_ZOOM
      if (showPorts !== this._portLabelsShown) {
        this._portLabelsShown = showPorts
        changed = true
      }
      if (changed) this.renderer?.refresh()
    },

    // === Réinitialisation ==================================================
    resetGraph() {
      if (this.layout) {
        try { this.layout.kill() } catch {}
        this.layout = null
      }
      this._layoutOrder = 0
      this.graph?.clear()
      this.clearNodeInfos()
      this.unpinTunnelHighlight()
    },

    // === Upserts ===========================================================
    /** Barycentre des nœuds existants (point d'apparition des nouveaux). */
    _spawnAnchor(): { x: number; y: number } {
      const g = this.graph
      if (!g || g.order === 0) return { x: 0, y: 0 }
      let sx = 0, sy = 0
      g.forEachNode((_n, attrs) => { sx += attrs.x; sy += attrs.y })
      return { x: sx / g.order, y: sy / g.order }
    },

    /** Ajoute ou met à jour un nœud. Retourne true si un élément a été ajouté. */
    upsertNode(node: any): boolean {
      if (!this.graph || !node?.id) return false
      const attrs = nodeAttributes(node)
      if (this.graph.hasNode(node.id)) {
        this.graph.mergeNodeAttributes(node.id, attrs)
        return false
      }
      const anchor = this._spawnAnchor()
      const pos = jitterAround(anchor.x, anchor.y, 200)
      this.graph.addNode(node.id, { ...attrs, ...pos, size: NODE_SIZE_MIN, _spawned: true })
      return true
    },

    /** Taille du nœud proportionnelle (log) au trafic cumulé de ses arêtes. */
    updateNodeTrafficSize(nodeId: string) {
      if (!this.graph?.hasNode(nodeId)) return
      let bytes = 0
      this.graph.forEachEdge(nodeId, (_edge, attrs) => { bytes += attrs.total_bytes || 0 })
      this.graph.setNodeAttribute(nodeId, "size", nodeSizeFor(bytes))
    },

    /** Ajoute ou met à jour une arête. Retourne true si un élément a été ajouté. */
    upsertEdge(e: any): boolean {
      if (!this.graph || !e?.source || !e?.target) return false
      // Arête orpheline : les deux extrémités doivent exister
      if (!this.graph.hasNode(e.source) || !this.graph.hasNode(e.target)) return false

      const key = edgeKey(e)
      const attrs = edgeAttributes(e)
      if (this.graph.hasEdge(key)) {
        this.graph.mergeEdgeAttributes(key, attrs)
        this.updateNodeTrafficSize(e.source)
        this.updateNodeTrafficSize(e.target)
        return false
      }
      this.graph.addEdgeWithKey(key, e.source, e.target, attrs)
      this.updateNodeTrafficSize(e.source)
      this.updateNodeTrafficSize(e.target)

      // Un nœud fraîchement apparu est déplacé à côté de son premier voisin
      // déjà placé : ForceAtlas2 n'a plus qu'un ajustement local à faire.
      const srcSpawned = !!this.graph.getNodeAttribute(e.source, "_spawned")
      const dstSpawned = !!this.graph.getNodeAttribute(e.target, "_spawned")
      if (srcSpawned !== dstSpawned) {
        const fresh = srcSpawned ? e.source : e.target
        const settled = srcSpawned ? e.target : e.source
        const p = jitterAround(
          this.graph.getNodeAttribute(settled, "x"),
          this.graph.getNodeAttribute(settled, "y")
        )
        this.graph.mergeNodeAttributes(fresh, { x: p.x, y: p.y, _spawned: false })
      }
      return true
    },

    /** Recalcule type/courbure des arêtes parallèles (multi-protocoles). */
    refreshParallelEdges() {
      if (!this.graph) return
      indexParallelEdgesIndex(this.graph, {
        edgeIndexAttribute: "parallelIndex",
        edgeMinIndexAttribute: "parallelMinIndex",
        edgeMaxIndexAttribute: "parallelMaxIndex",
      })
      this.graph.forEachEdge((edge, attrs) => {
        const { parallelIndex, parallelMinIndex, parallelMaxIndex } = attrs as any
        if (typeof parallelMinIndex === "number") {
          this.graph!.mergeEdgeAttributes(edge, {
            type: parallelIndex ? "curved" : "straight",
            curvature: parallelIndex ? getCurvature(parallelIndex, parallelMaxIndex) : 0,
          })
        } else if (typeof parallelIndex === "number") {
          this.graph!.mergeEdgeAttributes(edge, {
            type: "curved",
            curvature: getCurvature(parallelIndex, parallelMaxIndex),
          })
        } else {
          this.graph!.setEdgeAttribute(edge, "type", "straight")
        }
      })
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
        if (!this.graph) return

        this.resetGraph()

        const nodeEntries = Object.entries(snapshot.nodes || {});
        console.log(`[NetworkGraphComponent] Chargement de ${nodeEntries.length} nœuds`);

        const edgeEntries = Object.entries(snapshot.edges || {});
        console.log(`[NetworkGraphComponent] Chargement de ${edgeEntries.length} arêtes`);

        // Positions initiales en couronne : ForceAtlas2 démêle ensuite.
        let i = 0
        const n = Math.max(nodeEntries.length, 1)
        for (const [nodeId, node] of nodeEntries) {
          if (!node) continue;
          const id = node.id || nodeId
          const angle = (2 * Math.PI * i++) / n
          const radius = 100 + Math.sqrt(n) * 40
          this.graph.addNode(id, {
            ...nodeAttributes({ ...node, id }),
            x: Math.cos(angle) * radius + (Math.random() - 0.5) * 30,
            y: Math.sin(angle) * radius + (Math.random() - 0.5) * 30,
          })
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

        this.refreshParallelEdges()

        // Layout recréé avec des réglages adaptés à la taille du graphe.
        this.startLayout()
        if (!this.forceEnabled) this.layout?.stop()

        this.renderer?.getCamera().animatedReset()

      } catch (error) {
        console.error("[NetworkGraphComponent] Erreur critique dans loadFromGraphData:", error);
      }
    },

    // === Gestion label =====================================================
    onNodeClick(nodeId: string) {
      if (!this.graph?.hasNode(nodeId)) return
      const attrs = this.graph.getNodeAttributes(nodeId)
      this.selectedNodeId = nodeId
      this.selectedNode = {
        id: nodeId,
        name: attrs.name,
        mac: attrs.mac,
        ip: attrs.ip,
        color: attrs.color,
        label: attrs.rawLabel,
      }
      this.editedLabel = attrs.rawLabel ?? ""
      this.selectedNodeInfos = this._buildNodeInfos(nodeId)
      this.renderer?.refresh()
    },
    clearNodeInfos() {
      this.selectedNodeInfos = []
      this.selectedNode = null
      this.selectedNodeId = null
      this.editedLabel = ""
      this.renderer?.refresh()
    },
    async editNodeLabel() {
      if (!this.selectedNode || !this.selectedNodeId || !this.graph) return
      if (!this.graph.hasNode(this.selectedNodeId)) return
      const newLabel = String(this.editedLabel ?? "").trim()

      // MAJ UI immédiate
      const attrs = this.graph.getNodeAttributes(this.selectedNodeId)
      this.graph.mergeNodeAttributes(this.selectedNodeId, {
        rawLabel: newLabel,
        label: newLabel || attrs.name || this.selectedNodeId,
      })
      this.selectedNode = { ...this.selectedNode, label: newLabel }
      this.selectedNodeInfos = this._buildNodeInfos(this.selectedNodeId)

      // Appel backend avec mac/ip/label
      try {
        this.isSavingLabel = true
        await invoke("add_label", {
          mac: attrs.mac ?? "",
          ip: attrs.ip ?? "",
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
      if (!this.graph?.hasNode(nodeId)) return ["Nœud introuvable"]
      const n = this.graph.getNodeAttributes(nodeId)

      const protos = new Set<string>()
      let degree = 0
      let packets = 0
      let bytes = 0
      this.graph.forEachEdge(nodeId, (_edge, attrs) => {
        degree++
        if (attrs.protocol) protos.add(String(attrs.protocol))
        packets += attrs.count || 0
        bytes += attrs.total_bytes || 0
      })

      return [
        `ID: ${nodeId}`,
        `Nom: ${n.name ?? ""}`,
        `Label: ${n.rawLabel || "N/A"}`,
        `MAC: ${n.mac ?? ""}`,
        `IP: ${n.ip ?? ""}`,
        `Couleur: ${n.color}`,
        `Degré: ${degree}`,
        `Trafic: ${formatBytes(bytes)} (${packets} paquets)`,
        `Protocoles: ${[...protos].join(", ") || "—"}`,
      ]
    },

    // === Force Layout ======================================================
    /**
     * (Re)crée le superviseur ForceAtlas2 (worker) et le démarre.
     * ⚠️ Jamais sur un graphe vide : inferSettings y produit
     * slowDown = 1 + log(0) = -Infinity et fige la simulation pour toujours.
     */
    startLayout() {
      if (!this.graph || this.graph.order === 0) return
      if (this.layout) {
        try { this.layout.kill() } catch {}
        this.layout = null
      }
      this._layoutOrder = this.graph.order
      const settings = forceAtlas2.inferSettings(this.graph)
      this.layout = markRaw(new FA2Layout(this.graph, { settings }))
      this.layout.start()
    },
    /**
     * Recrée le layout quand le graphe vient de se peupler ou a beaucoup
     * grandi depuis le dernier inferSettings (slowDown/BarnesHut dépendent
     * de la taille). Le superviseur suit les ajouts intermédiaires tout seul.
     */
    ensureLayoutSettings() {
      if (!this.forceEnabled || !this.graph || this.graph.order === 0) return
      if (!this.layout || this._layoutOrder === 0 || this.graph.order >= this._layoutOrder * 4) {
        this.startLayout()
      }
    },
    toggleForce() {
      if (this.forceEnabled) {
        this.forceEnabled = false
        this.layout?.stop()
      } else {
        this.forceEnabled = true
        if (this.layout && this._layoutOrder > 0) this.layout.start()
        else this.startLayout()
      }
    },

    // === Export PNG ========================================================
    async downloadPng() {
      if (!this.renderer) return
      const filePath = await save({
        filters: [{ name: "PNG File", extensions: ["png"] }],
        defaultPath: getCurrentDate() + "_network_graph_DR_Matrice.png",
      })
      if (!filePath) return

      const { width, height } = this.renderer.getDimensions()
      const blob = await toBlob(this.renderer, {
        format: "png",
        backgroundColor: "#ffffff",
        width: width * 2,
        height: height * 2,
      })
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
      if (!q.length || !this.graph) return

      let addedEdges = false
      for (let i = 0; i < q.length; i++) {
        if (this.applyUpdate(q[i]) === "edge") addedEdges = true
      }
      this._queue.length = 0

      // Le superviseur ForceAtlas2 suit les ajouts tout seul ; il ne reste
      // qu'à recalculer la courbure des arêtes parallèles.
      if (addedEdges) this.refreshParallelEdges()
      this.ensureLayoutSettings()
    },
    /** Retourne "node" | "edge" si un élément a été ajouté au graphe. */
    applyUpdate(update: GraphUpdate | any): "node" | "edge" | null {
      if (!update) return null
      const u = this.normalizeGraphUpdate(update)
      if (!u) return null

      switch (u.type) {
        case "NodeAdded":
          return this.upsertNode(u.payload) ? "node" : null
        case "NodeUpdated": {
          const node = u.payload
          if (!node) return null
          const added = this.upsertNode(node)

          if (this.selectedNodeId === node.id && this.graph?.hasNode(node.id)) {
            const attrs = this.graph.getNodeAttributes(node.id)
            this.selectedNode = {
              id: node.id,
              name: attrs.name,
              mac: attrs.mac,
              ip: attrs.ip,
              color: attrs.color,
              label: attrs.rawLabel,
            }
            this.editedLabel = attrs.rawLabel ?? ""
            this.selectedNodeInfos = this._buildNodeInfos(node.id)
          }
          return added ? "node" : null
        }
        case "EdgeAdded":
        case "EdgeUpdated":
          return this.upsertEdge(u.payload) ? "edge" : null
      }
      return null
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
    <div class="graph" ref="sigmaContainer"></div>

    <!-- Bandeau d'infos en bas -->
    <div class="bottom-info">
      <div class="zoom">Zoom: {{ zoomLevel.toPrecision(2) }}</div>
      <template v-if="tunnelHoverInfo">
        <div class="sep" />
        <div class="tunnel-info">🚇 {{ tunnelHoverInfo }}</div>
      </template>
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

    <!-- Panneau : labels appliqués à la matrice -->
    <div v-if="showLabelsPanel" class="labels-overlay" @click.self="closeLabelsPanel">
      <div class="labels-modal">
        <div class="labels-modal-header">
          <h3>Labels appliqués à la matrice ({{ matrixLabels.length }})</h3>
          <button class="labels-close" @click="closeLabelsPanel" title="Fermer">✕</button>
        </div>

        <input
          v-model="labelsSearch"
          class="labels-search"
          type="text"
          placeholder="Rechercher (MAC, IP, label)…"
        />

        <p v-if="matrixLabels.length === 0" class="labels-empty">
          Aucun label appliqué à la matrice pour le moment.
        </p>

        <div v-else class="labels-table">
          <div class="labels-row labels-head">
            <div class="labels-col">Adresse MAC</div>
            <div class="labels-col">Adresse IP</div>
            <div class="labels-col">Label</div>
          </div>
          <div class="labels-body">
            <div
              v-for="([mac, ip, label], index) in filteredMatrixLabels"
              :key="index"
              class="labels-row"
            >
              <div class="labels-col">{{ mac || "-" }}</div>
              <div class="labels-col">{{ ip || "-" }}</div>
              <div class="labels-col">{{ label || "-" }}</div>
            </div>
          </div>
        </div>
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
.bottom-info .tunnel-info {
  color: #1de9b6;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
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

/* Panneau "Afficher les labels" */
.labels-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 50;
}
.labels-modal {
  display: flex;
  flex-direction: column;
  width: 90%;
  max-width: 640px;
  max-height: 80%;
  background: #1e1e2e;
  border: 1px solid #444;
  border-radius: 8px;
  padding: 1rem 1.25rem;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  color: whitesmoke;
}
.labels-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}
.labels-modal-header h3 { margin: 0; font-size: 1.1rem; }
.labels-close {
  background: transparent;
  color: #bbb;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  line-height: 1;
}
.labels-close:hover { color: #fff; }
.labels-search {
  width: 100%;
  box-sizing: border-box;
  margin-bottom: 0.75rem;
  padding: 0.4em 0.7em;
  border-radius: 6px;
  border: 1px solid #444;
  background: #2d3748;
  color: whitesmoke;
}
.labels-empty { color: rgba(245, 245, 245, 0.6); text-align: center; padding: 1rem 0; }
.labels-table {
  display: flex;
  flex-direction: column;
  min-height: 0;
  border: 1px solid #333;
  border-radius: 6px;
  overflow: hidden;
}
.labels-body { overflow-y: auto; }
.labels-row {
  display: flex;
  padding: 0.35rem 0.5rem;
  border-bottom: 1px solid #2a2a3a;
}
.labels-row:last-child { border-bottom: none; }
.labels-head {
  font-weight: 600;
  background: #262636;
  position: sticky;
  top: 0;
}
.labels-col {
  flex: 1;
  padding: 0 0.4rem;
  word-break: break-all;
  font-family: monospace;
  font-size: 0.85rem;
}
</style>
