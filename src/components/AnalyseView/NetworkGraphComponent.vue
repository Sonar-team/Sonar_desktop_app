<!--
  Graphe réseau interactif (Sigma + graphology + ForceAtlas2) : rendu des
  nœuds/arêtes reçus du backend, surbrillance des familles de tunnels,
  sélection et édition de labels, export PNG. Les helpers de style et les
  mutations du graphe vivent dans ./graph/.
-->
<script lang="ts">
import { defineComponent, markRaw } from "vue"
import Graph from "graphology"
import Sigma from "sigma"
import { EdgeArrowProgram } from "sigma/rendering"
import { EdgeCurvedArrowProgram } from "@sigma/edge-curve"
import { NodeBorderProgram } from "@sigma/node-border"
import { useCaptureStore } from "../../store/capture"
import { save } from "@tauri-apps/plugin-dialog"
import { error, info, warn } from "@tauri-apps/plugin-log"
import { GraphData, GraphUpdate, NodeData } from "../../types/capture"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentDate } from '../../utils/time';
import LegendComponent from './LegendComponent.vue';
import MatrixLabelsPanel from './graph/MatrixLabelsPanel.vue';
import {
  DIM_EDGE_COLOR,
  DIM_NODE_COLOR,
  EDGE_LABEL_ZOOM,
  PORT_LABEL_ZOOM,
  drawNodeLabel,
  nodeAttributes,
} from "./graph/graphStyle"
import {
  logUnknownGraphUpdateType,
  refreshParallelEdges,
  upsertEdge,
  upsertNode,
} from "./graph/graphSync"
import { ForceLayoutController, createForceLayoutController } from "./graph/forceLayout"
import { computeTunnelHighlight, edgeHasTunnel } from "./graph/tunnelHighlight"
import { applyOptimisticLabel } from "./graph/labelEdit"
import { buildNodeInfos, nodeSnapshot } from "./graph/nodeInfo"
import { displayCaptureError } from "../../errors/capture"
import { exportGraphToPng } from "./graph/exportPng"

// Attributs graphology reçus/renvoyés par les reducers Sigma : un sur-
// ensemble de ce que `nodeAttributes`/`edgeAttributes` (graphStyle.ts)
// produisent, plus `x`/`y`/`size`/`highlighted`/`zIndex` ajoutés par
// graphology/Sigma. L'index signature couvre le reste de l'attribut bag
// (Sigma le type en `Attributes` générique côté framework) sans retomber
// sur `any` pour les champs que ces reducers lisent/écrivent réellement.
interface NodeRenderAttributes extends Record<string, unknown> {
  color?: string
  label?: string | null
  hoverColor?: string
  highlighted?: boolean
}

interface EdgeRenderAttributes extends Record<string, unknown> {
  color?: string
  label?: string | null
  size?: number
  zIndex?: number
  protocol?: string
  ports?: number[]
  has_dynamic_ports?: boolean
  source_port?: number | null
  destination_port?: number | null
}

export default defineComponent({
  name: "NetworkGraphComponent",
  components: { LegendComponent, MatrixLabelsPanel },

  data() {
    return {
      graph: null as Graph | null,
      renderer: null as Sigma | null,
      forceLayout: null as ForceLayoutController | null,

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

      // Queue
      _queue: [] as GraphUpdate[],
      _raf: 0 as number,

      // Affichage des labels d'arêtes selon le zoom
      _edgeLabelsShown: false,
      _portLabelsShown: false,

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
      this.loadFromGraphData(graphData);
    }));

    // Abonnement au reset via le bus global
    this.resetHandler = () => this.resetGraph()
    this.$bus?.on?.('reset', this.resetHandler)

    this.startLayout()
    if (!this.forceEnabled) this.forceLayout?.stop()
  },

  beforeUnmount() {
    for (const unsub of this.graphUnsubs) {
      try { unsub() } catch { /* already unsubscribed */ }
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

    this.forceLayout?.kill()
    this.forceLayout = null

    if (this.renderer) {
      this.renderer.kill()
      this.renderer = null
    }
    this.graph = null
  },

  methods: {
    // === Initialisation Sigma ==============================================
    initSigma() {
      const container = this.$refs.sigmaContainer as HTMLElement
      const graph = new Graph({ multi: true, type: "directed" })
      this.graph = markRaw(graph)
      this.forceLayout = markRaw(createForceLayoutController(graph))

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
        nodeReducer: (node, data) => this.nodeReducer(node, data),
        edgeReducer: (edge, data) => this.edgeReducer(edge, data),
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

    // === Reducers Sigma (état de survol/sélection -> rendu) ================
    nodeReducer(node: string, data: NodeRenderAttributes): NodeRenderAttributes {
      const res: NodeRenderAttributes = { ...data }
      const tunnelNodes = this.hoveredTunnelNodes
      if (tunnelNodes && !tunnelNodes.has(node)) {
        res.color = DIM_NODE_COLOR
        res.label = null
      }
      if (node === this.hoveredNode) res.color = data.hoverColor ?? res.color
      if (node === this.selectedNodeId) res.highlighted = true
      return res
    },

    edgeReducer(edge: string, data: EdgeRenderAttributes): EdgeRenderAttributes {
      const res: EdgeRenderAttributes = { ...data }
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
      if (this._portLabelsShown) {
        const ports: number[] = Array.isArray(data.ports) ? data.ports : []
        const hasDynamic = data.has_dynamic_ports === true
        if (ports.length > 0) {
          // Ports « service » de l'arête (les éphémères ne sont pas listés,
          // le backend les résume par has_dynamic_ports → « … »).
          label += ` :${ports.join(",")}${hasDynamic ? ",…" : ""}`
        } else if (hasDynamic) {
          // Uniquement du trafic sur ports dynamiques : signalé sans liste.
          label += " :…"
        } else if (data.source_port != null || data.destination_port != null) {
          label += ` ${data.source_port ?? ""}→${data.destination_port ?? ""}`
        }
      }
      res.label = label
      return res
    },

    // === Surbrillance des tunnels ==========================================
    /** Surligne la famille de tunnel(s) de `edgeId` (calcul délégué à
     * ./graph/tunnelHighlight.ts). Retourne true si une famille a été surlignée. */
    applyTunnelHighlight(edgeId: string): boolean {
      if (!this.graph) return false
      const result = computeTunnelHighlight(this.graph, edgeId, this.pinnedTunnelEdge)
      if (!result) return false

      this.hoveredTunnelEdges = result.edges
      this.hoveredTunnelNodes = result.nodes
      this.tunnelHoverInfo = result.info
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
      if (this.graph && edgeHasTunnel(this.graph, edgeId)) {
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
      this.forceLayout?.kill()
      this.graph?.clear()
      this.clearNodeInfos()
      this.unpinTunnelHighlight()
    },

    /**
     * Recharge complètement le graphe à partir d'un snapshot complet
     * envoyé par le backend (GraphSnapshot).
     */
    loadFromGraphData(snapshot: GraphData | null | undefined) {
      try {
        if (!snapshot) {
          error("[NetworkGraphComponent] GraphSnapshot sans donnée");
          return;
        }
        if (!snapshot.nodes || !snapshot.edges) {
          error(`[NetworkGraphComponent] Données de graphe invalides (nodes: ${!!snapshot.nodes}, edges: ${!!snapshot.edges})`);
          return;
        }
        if (!this.graph) return

        this.resetGraph()

        const nodeEntries = Object.entries(snapshot.nodes || {});
        const edgeEntries = Object.entries(snapshot.edges || {});
        info(`[NetworkGraphComponent] Chargement du snapshot : ${nodeEntries.length} nœuds, ${edgeEntries.length} arêtes`);

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
            warn(`[NetworkGraphComponent] Arête ${edgeId} invalide: source ou target manquante`);
            continue;
          }
          if (!upsertEdge(this.graph, edge)) {
            warn(`[NetworkGraphComponent] Arête orpheline ignorée: ${edge.source} -> ${edge.target} (${edge.label})`);
          }
        }

        refreshParallelEdges(this.graph)

        // Layout recréé avec des réglages adaptés à la taille du graphe.
        this.startLayout()
        if (!this.forceEnabled) this.forceLayout?.stop()

        this.renderer?.getCamera().animatedReset()

      } catch (e) {
        error(`[NetworkGraphComponent] Erreur critique dans loadFromGraphData: ${e}`);
      }
    },

    // === Gestion label =====================================================
    onNodeClick(nodeId: string) {
      if (!this.graph) return
      const snapshot = nodeSnapshot(this.graph, nodeId)
      if (!snapshot) return
      this.selectedNodeId = nodeId
      this.selectedNode = snapshot
      this.editedLabel = snapshot.label ?? ""
      this.selectedNodeInfos = buildNodeInfos(this.graph, nodeId)
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

      // MAJ UI immédiate, avec rollback si le backend refuse (#161).
      const nodeId = this.selectedNodeId
      const prevSelected = this.selectedNode
      const attrs = this.graph.getNodeAttributes(nodeId)
      const rollback = applyOptimisticLabel(this.graph, nodeId, newLabel)
      this.selectedNode = { ...this.selectedNode, label: newLabel }
      this.selectedNodeInfos = buildNodeInfos(this.graph, nodeId)

      // Appel backend avec mac/ip/label. La commande resynchronise tous les
      // labels (#157) et retourne les updates graphe : appliquées via le
      // store, sans dépendre du channel de capture.
      try {
        this.isSavingLabel = true
        const updates = await invoke<GraphUpdate[]>("add_label", {
          mac: attrs.mac ?? "",
          ip: attrs.ip ?? "",
          label: newLabel,
        })
        this.captureStore.applyGraphUpdates(updates)
      } catch (e) {
        rollback()
        if (this.selectedNodeId === nodeId) {
          this.selectedNode = prevSelected
          this.editedLabel = prevSelected.label ?? ""
          this.selectedNodeInfos = buildNodeInfos(this.graph, nodeId)
        }
        await displayCaptureError(e)
      } finally {
        this.isSavingLabel = false
      }
    },
    onEditKeydown(e: KeyboardEvent) {
      if (e.key === "Enter") this.editNodeLabel()
      else if (e.key === "Escape") this.clearNodeInfos()
    },
    cancelEdit() {
      if (this.selectedNode && this.selectedNodeId && this.graph) {
        this.editedLabel = this.selectedNode.label ?? ""
        this.selectedNodeInfos = buildNodeInfos(this.graph, this.selectedNodeId)
      }
    },

    // === Force Layout ======================================================
    // (Re)création, arrêt et adaptation du superviseur ForceAtlas2 délégués
    // au contrôleur ./graph/forceLayout.ts.
    startLayout() {
      this.forceLayout?.start()
    },
    ensureLayoutSettings() {
      if (!this.forceEnabled) return
      this.forceLayout?.ensureSettings()
    },
    toggleForce() {
      this.forceEnabled = !this.forceEnabled
      if (this.forceEnabled) this.forceLayout?.resume()
      else this.forceLayout?.stop()
    },

    // === Export PNG ========================================================
    async downloadPng() {
      if (!this.renderer) return
      const filePath = await save({
        filters: [{ name: "PNG File", extensions: ["png"] }],
        defaultPath: getCurrentDate() + "_network_graph_DR_Matrice.png",
      })
      if (!filePath) return

      // Le typage Options API de Vue « déballe » l'instance Sigma (markRaw) ;
      // le cast rend le type nominal attendu par exportGraphToPng.
      await exportGraphToPng(this.renderer as Sigma, filePath)
      info(`PNG exporté dans ${filePath}`)
    },

    // === Queue & updates ===================================================
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
      if (addedEdges) refreshParallelEdges(this.graph)
      this.ensureLayoutSettings()
    },
    /** Retourne "node" | "edge" si un élément a été ajouté au graphe. */
    applyUpdate(update: GraphUpdate): "node" | "edge" | null {
      if (!update || !this.graph) return null

      switch (update.type) {
        case "nodeAdded":
          return upsertNode(this.graph, update.payload) ? "node" : null
        case "nodeUpdated": {
          const node = update.payload
          const added = upsertNode(this.graph, node)
          this.refreshSelectedNode(node.id)
          return added ? "node" : null
        }
        case "edgeAdded":
        case "edgeUpdated":
          return upsertEdge(this.graph, update.payload) ? "edge" : null
        default:
          return logUnknownGraphUpdateType(update)
      }
    },
    /** Resynchronise le bandeau bas si le nœud mis à jour est sélectionné. */
    refreshSelectedNode(nodeId: string | undefined) {
      if (!nodeId || this.selectedNodeId !== nodeId || !this.graph) return
      const snapshot = nodeSnapshot(this.graph, nodeId)
      if (!snapshot) return
      this.selectedNode = snapshot
      this.editedLabel = snapshot.label ?? ""
      this.selectedNodeInfos = buildNodeInfos(this.graph, nodeId)
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
      <button class="download-button" @click="showLabelsPanel = true" title="afficher les labels">Afficher les labels</button>
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
    <MatrixLabelsPanel v-if="showLabelsPanel" @close="showLabelsPanel = false" />

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
</style>
