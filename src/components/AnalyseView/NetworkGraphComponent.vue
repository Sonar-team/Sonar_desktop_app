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
    color: e._color || colorForLabel(e.label || ""),
    size: edgeSizeFor(totalBytes),
  }
}

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

      // Taille du graphe lors du dernier inferSettings du layout
      _layoutOrder: 0,

      // Handlers pour cleanup
      resetHandler: null as (() => void) | null,
      graphUnsubs: [] as Array<() => void>,
    }
  },

  computed: {
    captureStore() { return useCaptureStore() },
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
      await invoke('get_label_list').then((labels: any) => {
        console.log(labels)
      })
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
        nodeReducer: (node, data) => {
          const res: any = { ...data }
          if (node === this.hoveredNode) res.color = data.hoverColor ?? res.color
          if (node === this.selectedNodeId) res.highlighted = true
          return res
        },
        edgeReducer: (_edge, data) => {
          const res: any = { ...data }
          if (!this._edgeLabelsShown) {
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
      renderer.on("clickStage", () => this.clearNodeInfos())
      renderer.on("enterNode", ({ node }) => { this.hoveredNode = node; renderer.refresh() })
      renderer.on("leaveNode", () => { this.hoveredNode = null; renderer.refresh() })

      renderer.getCamera().on("updated", (state) => this.onZoom(1 / state.ratio))
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
