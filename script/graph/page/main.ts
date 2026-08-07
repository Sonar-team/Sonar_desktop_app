// Rendu hors-ligne d'une matrice de flux avec sigma.js, exécuté dans un
// navigateur headless par ../render-sigma.mjs.
//
// Le graphe, les styles et l'export PNG sont ceux de l'application desktop
// (src/components/AnalyseView/graph/) : l'image produite par le script est la
// même que celle du bouton « Exporter en PNG », layout figé en plus.
import Graph from "graphology"
import forceAtlas2 from "graphology-layout-forceatlas2"
import Sigma from "sigma"
import { EdgeArrowProgram } from "sigma/rendering"
import { EdgeCurvedArrowProgram } from "@sigma/edge-curve"
import { NodeBorderProgram } from "@sigma/node-border"
import { toBlob } from "@sigma/export-image"
import type { Edge, GraphData, Node } from "../../../src/types/capture"
import {
  drawNodeLabel,
  nodeAttributes,
} from "../../../src/components/AnalyseView/graph/graphStyle"
import {
  refreshParallelEdges,
  upsertEdge,
} from "../../../src/components/AnalyseView/graph/graphSync"

export interface RenderOptions {
  width: number
  height: number
  scale: number
  iterations: number
  background: string
  edgeLabels: boolean
  seed: number
}

declare global {
  interface Window {
    __SONAR_GRAPH__: GraphData
    __SONAR_OPTIONS__: RenderOptions
  }
}

/** PRNG déterministe (mulberry32) : deux rendus de la même matrice donnent
 * la même image, alors que le placement initial des nœuds et le jitter de
 * `graphStyle` tirent au sort. Math.random est remplacé pour toute la durée
 * du rendu — la page ne fait rien d'autre. */
function seedRandom(seed: number) {
  let state = seed >>> 0
  Math.random = () => {
    state = (state + 0x6d2b79f5) >>> 0
    let t = state
    t = Math.imul(t ^ (t >>> 15), t | 1)
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

function buildGraph(snapshot: GraphData, options: RenderOptions): Graph {
  const graph = new Graph({ multi: true, type: "directed" })

  // Mêmes positions initiales en couronne que le chargement d'un snapshot
  // dans l'application : ForceAtlas2 démêle ensuite.
  const nodeEntries = Object.entries(snapshot.nodes ?? {}) as Array<[string, Node]>
  const total = Math.max(nodeEntries.length, 1)
  let index = 0
  for (const [nodeId, node] of nodeEntries) {
    if (!node) continue
    const id = node.id || nodeId
    const angle = (2 * Math.PI * index++) / total
    const radius = 100 + Math.sqrt(total) * 40
    graph.addNode(id, {
      ...nodeAttributes({ ...node, id }),
      x: Math.cos(angle) * radius + (Math.random() - 0.5) * 30,
      y: Math.sin(angle) * radius + (Math.random() - 0.5) * 30,
    })
  }

  const edgeEntries = Object.entries(snapshot.edges ?? {}) as Array<[string, Edge]>
  let skipped = 0
  for (const [, edge] of edgeEntries) {
    if (!edge?.source || !edge?.target) {
      skipped++
      continue
    }
    if (!upsertEdge(graph, edge)) skipped++
  }
  if (skipped) console.warn(`[sonar] ${skipped} arête(s) ignorée(s) (orpheline ou en double)`)

  refreshParallelEdges(graph)

  if (graph.order > 0 && options.iterations > 0) {
    forceAtlas2.assign(graph, {
      iterations: options.iterations,
      settings: forceAtlas2.inferSettings(graph),
    })
  }
  return graph
}

function render(graph: Graph, options: RenderOptions): Sigma {
  const container = document.getElementById("sigma") as HTMLElement
  container.style.width = `${options.width}px`
  container.style.height = `${options.height}px`

  // Réglages du rendu de l'application (NetworkGraphComponent.vue), moins
  // l'interactivité : pas de survol ni de sélection dans une image fixe.
  return new Sigma(graph, container, {
    allowInvalidContainer: true,
    defaultNodeType: "bordered",
    defaultEdgeType: "straight",
    nodeProgramClasses: { bordered: NodeBorderProgram },
    edgeProgramClasses: { straight: EdgeArrowProgram, curved: EdgeCurvedArrowProgram },
    renderEdgeLabels: options.edgeLabels,
    labelSize: 13,
    labelColor: { color: "#ffffff" },
    edgeLabelSize: 11,
    edgeLabelColor: { color: "#E0E0E0" },
    defaultDrawNodeLabel: drawNodeLabel,
    defaultDrawNodeHover: drawNodeLabel,
    // Les défauts de sigma sont conservés pour la grille de placement des
    // labels : `labelGridCellSize: 0` supprimait la grille et donc TOUS les
    // labels. Seul le seuil de taille est abaissé, pour qu'un petit nœud
    // garde son libellé — une image fixe n'a pas de zoom pour le révéler.
    labelRenderedSizeThreshold: 0,
    zIndex: true,
  })
}

async function toBase64(blob: Blob): Promise<string> {
  const buffer = new Uint8Array(await blob.arrayBuffer())
  let binary = ""
  const CHUNK = 0x8000
  for (let i = 0; i < buffer.length; i += CHUNK) {
    binary += String.fromCharCode(...buffer.subarray(i, i + CHUNK))
  }
  return btoa(binary)
}

/** Publie le résultat dans le DOM : le script Node le récupère via
 * `--dump-dom`, ce qui évite d'ajouter un pilote de navigateur (CDP) au
 * projet. */
function publish(kind: "png" | "error", payload: string) {
  const holder = document.getElementById(`sonar-${kind}`) as HTMLElement
  // Bornes explicites : le DOM sérialisé est parcouru par une recherche de
  // marqueurs, insensible à la façon dont le navigateur réordonne les
  // attributs ou échappe le texte.
  holder.textContent = `[SONAR:${kind}:begin]${payload}[SONAR:${kind}:end]`
  document.title = kind === "png" ? "SONAR-OK" : "SONAR-ERROR"
}

async function main() {
  const options = window.__SONAR_OPTIONS__
  seedRandom(options.seed)
  const graph = buildGraph(window.__SONAR_GRAPH__, options)
  if (graph.order === 0) throw new Error("graphe vide : aucune relation à rendre")

  const renderer = render(graph, options)
  renderer.refresh()

  const blob = await toBlob(renderer, {
    format: "png",
    backgroundColor: options.background,
    width: options.width * options.scale,
    height: options.height * options.scale,
  })
  publish("png", await toBase64(blob))
  console.log(`[sonar] ${graph.order} nœud(s), ${graph.size} arête(s)`)
}

main().catch((error: unknown) => {
  publish("error", error instanceof Error ? `${error.message}\n${error.stack}` : String(error))
})
