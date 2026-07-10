// Mutations du graphe graphology à partir des données du backend :
// upserts de nœuds/arêtes, tailles proportionnelles au trafic, courbure des
// arêtes parallèles et normalisation des updates reçues du canal.
import Graph from "graphology"
import { indexParallelEdgesIndex } from "@sigma/edge-curve"
import { GraphUpdate } from "../../../types/capture"
import {
  NODE_SIZE_MIN,
  edgeAttributes,
  edgeKey,
  getCurvature,
  jitterAround,
  nodeAttributes,
  nodeSizeFor,
} from "./graphStyle"

/** Barycentre des nœuds existants (point d'apparition des nouveaux). */
export function spawnAnchor(graph: Graph): { x: number; y: number } {
  if (graph.order === 0) return { x: 0, y: 0 }
  let sx = 0, sy = 0
  graph.forEachNode((_n, attrs) => { sx += attrs.x; sy += attrs.y })
  return { x: sx / graph.order, y: sy / graph.order }
}

/** Ajoute ou met à jour un nœud. Retourne true si un élément a été ajouté. */
export function upsertNode(graph: Graph, node: any): boolean {
  if (!node?.id) return false
  const attrs = nodeAttributes(node)
  if (graph.hasNode(node.id)) {
    graph.mergeNodeAttributes(node.id, attrs)
    return false
  }
  const anchor = spawnAnchor(graph)
  const pos = jitterAround(anchor.x, anchor.y, 200)
  graph.addNode(node.id, { ...attrs, ...pos, size: NODE_SIZE_MIN, _spawned: true })
  return true
}

/** Taille du nœud proportionnelle (log) au trafic cumulé de ses arêtes. */
export function updateNodeTrafficSize(graph: Graph, nodeId: string) {
  if (!graph.hasNode(nodeId)) return
  let bytes = 0
  graph.forEachEdge(nodeId, (_edge, attrs) => { bytes += attrs.total_bytes || 0 })
  graph.setNodeAttribute(nodeId, "size", nodeSizeFor(bytes))
}

/** Ajoute ou met à jour une arête. Retourne true si un élément a été ajouté. */
export function upsertEdge(graph: Graph, e: any): boolean {
  if (!e?.source || !e?.target) return false
  // Arête orpheline : les deux extrémités doivent exister
  if (!graph.hasNode(e.source) || !graph.hasNode(e.target)) return false

  const key = edgeKey(e)
  const attrs = edgeAttributes(e)
  if (graph.hasEdge(key)) {
    graph.mergeEdgeAttributes(key, attrs)
    updateNodeTrafficSize(graph, e.source)
    updateNodeTrafficSize(graph, e.target)
    return false
  }
  graph.addEdgeWithKey(key, e.source, e.target, attrs)
  updateNodeTrafficSize(graph, e.source)
  updateNodeTrafficSize(graph, e.target)

  // Un nœud fraîchement apparu est déplacé à côté de son premier voisin
  // déjà placé : ForceAtlas2 n'a plus qu'un ajustement local à faire.
  const srcSpawned = !!graph.getNodeAttribute(e.source, "_spawned")
  const dstSpawned = !!graph.getNodeAttribute(e.target, "_spawned")
  if (srcSpawned !== dstSpawned) {
    const fresh = srcSpawned ? e.source : e.target
    const settled = srcSpawned ? e.target : e.source
    const p = jitterAround(
      graph.getNodeAttribute(settled, "x"),
      graph.getNodeAttribute(settled, "y")
    )
    graph.mergeNodeAttributes(fresh, { x: p.x, y: p.y, _spawned: false })
  }
  return true
}

/** Recalcule type/courbure des arêtes parallèles (multi-protocoles). */
export function refreshParallelEdges(graph: Graph) {
  indexParallelEdgesIndex(graph, {
    edgeIndexAttribute: "parallelIndex",
    edgeMinIndexAttribute: "parallelMinIndex",
    edgeMaxIndexAttribute: "parallelMaxIndex",
  })
  graph.forEachEdge((edge, attrs) => {
    const { parallelIndex, parallelMinIndex, parallelMaxIndex } = attrs as any
    if (typeof parallelMinIndex === "number") {
      graph.mergeEdgeAttributes(edge, {
        type: parallelIndex ? "curved" : "straight",
        curvature: parallelIndex ? getCurvature(parallelIndex, parallelMaxIndex) : 0,
      })
    } else if (typeof parallelIndex === "number") {
      graph.mergeEdgeAttributes(edge, {
        type: "curved",
        curvature: getCurvature(parallelIndex, parallelMaxIndex),
      })
    } else {
      graph.setEdgeAttribute(edge, "type", "straight")
    }
  })
}

/** Ramène les différents formats d'update du backend à la forme typée. */
export function normalizeGraphUpdate(raw: any): GraphUpdate | null {
  const u = raw?.update ?? raw
  if (!u) return null
  if (u.type && "payload" in u) return u as GraphUpdate
  if (u.NewNode) return { type: "NodeAdded", payload: u.NewNode }
  if (u.NodeUpdated) return { type: "NodeUpdated", payload: u.NodeUpdated }
  if (u.NewEdge) return { type: "EdgeAdded", payload: u.NewEdge }
  if (u.EdgeUpdated) return { type: "EdgeUpdated", payload: u.EdgeUpdated }
  return null
}
