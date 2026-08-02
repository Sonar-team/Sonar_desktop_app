// Mutations du graphe graphology à partir des données du backend :
// upserts de nœuds/arêtes, tailles proportionnelles au trafic et courbure
// des arêtes parallèles.
import Graph from "graphology"
import { indexParallelEdgesIndex } from "@sigma/edge-curve"
import { Edge, Node } from "../../../types/capture"
import {
  NODE_SIZE_MIN,
  edgeAttributes,
  edgeKey,
  getCurvature,
  jitterAround,
  nodeAttributes,
  nodeSizeFor,
} from "./graphStyle"

// Attributs graphology d'une arête tels que laissés par
// `indexParallelEdgesIndex` (@sigma/edge-curve) : un sur-ensemble de ce que
// `edgeAttributes` produit, avec les trois champs que cette fonction ajoute.
interface ParallelEdgeAttributes extends Record<string, unknown> {
  parallelIndex?: number
  parallelMinIndex?: number
  parallelMaxIndex?: number
}

// Nombre de nœuds échantillonnés pour l'ancrage d'apparition : suffisant
// pour retomber « raisonnablement près » du cluster existant (ForceAtlas2
// corrige l'écart ensuite), sans coût O(n) par nouveau nœud — O(n²) cumulé
// sur une capture longue avec beaucoup d'hôtes.
const SPAWN_ANCHOR_SAMPLE_SIZE = 200

/** Barycentre approché des nœuds existants (point d'apparition des nouveaux),
 *  calculé sur un échantillon borné plutôt que sur tout le graphe. */
export function spawnAnchor(graph: Graph): { x: number; y: number } {
  if (graph.order === 0) return { x: 0, y: 0 }
  let sx = 0, sy = 0, n = 0
  graph.findNode((_node, attrs) => {
    sx += attrs.x; sy += attrs.y
    n += 1
    return n >= SPAWN_ANCHOR_SAMPLE_SIZE
  })
  return { x: sx / n, y: sy / n }
}

/** Ajoute ou met à jour un nœud. Retourne true si un élément a été ajouté. */
export function upsertNode(graph: Graph, node: Node | null | undefined): boolean {
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

/** Taille du nœud proportionnelle (log) au trafic cumulé de ses arêtes,
 *  maintenue par delta (attribut interne `_trafficBytes`) plutôt que par un
 *  rebalayage O(degré) de toutes les arêtes à chaque upsert — coûteux en
 *  O(degré²) cumulé sur un nœud « hub » à fort degré. Sûr : les arêtes ne
 *  sont jamais retirées individuellement (seul `graph.clear()` les efface,
 *  ce qui efface aussi les nœuds et donc `_trafficBytes` avec eux). */
function applyNodeTrafficDelta(graph: Graph, nodeId: string, deltaBytes: number) {
  if (!graph.hasNode(nodeId) || deltaBytes === 0) return
  const bytes = (Number(graph.getNodeAttribute(nodeId, "_trafficBytes")) || 0) + deltaBytes
  graph.setNodeAttribute(nodeId, "_trafficBytes", bytes)
  graph.setNodeAttribute(nodeId, "size", nodeSizeFor(bytes))
}

/** Ajoute ou met à jour une arête. Retourne true si un élément a été ajouté. */
export function upsertEdge(graph: Graph, e: Edge | null | undefined): boolean {
  if (!e?.source || !e?.target) return false
  // Arête orpheline : les deux extrémités doivent exister
  if (!graph.hasNode(e.source) || !graph.hasNode(e.target)) return false

  const key = edgeKey(e)
  const attrs = edgeAttributes(e)
  if (graph.hasEdge(key)) {
    // Le backend envoie un compteur cumulé par arête : le delta face à
    // l'ancienne valeur suffit à tenir la taille des nœuds à jour.
    const previousBytes = Number(graph.getEdgeAttribute(key, "total_bytes")) || 0
    graph.mergeEdgeAttributes(key, attrs)
    const delta = attrs.total_bytes - previousBytes
    applyNodeTrafficDelta(graph, e.source, delta)
    applyNodeTrafficDelta(graph, e.target, delta)
    return false
  }
  graph.addEdgeWithKey(key, e.source, e.target, attrs)
  applyNodeTrafficDelta(graph, e.source, attrs.total_bytes)
  applyNodeTrafficDelta(graph, e.target, attrs.total_bytes)

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
  graph.forEachEdge((edge, attrs: ParallelEdgeAttributes) => {
    const { parallelIndex, parallelMinIndex, parallelMaxIndex } = attrs
    if (typeof parallelMinIndex === "number") {
      graph.mergeEdgeAttributes(edge, {
        type: parallelIndex ? "curved" : "straight",
        curvature: parallelIndex ? getCurvature(parallelIndex, parallelMaxIndex ?? 0) : 0,
      })
    } else if (typeof parallelIndex === "number") {
      graph.mergeEdgeAttributes(edge, {
        type: "curved",
        curvature: getCurvature(parallelIndex, parallelMaxIndex ?? 0),
      })
    } else {
      graph.setEdgeAttribute(edge, "type", "straight")
    }
  })
}

/** Exhaustivité imposée par le compilateur sur `GraphUpdate["type"]` : toute
 * variante ajoutée sans branche dans le `switch` appelant casse le build
 * TypeScript. Journalise plutôt que de planter (même politique que
 * `logUnknownEvent` dans `store/capture.ts`, #142). */
export function logUnknownGraphUpdateType(update: never): null {
  console.warn("[graphSync] type de GraphUpdate inconnu ignoré :", update)
  return null
}
