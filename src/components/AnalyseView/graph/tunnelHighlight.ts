// Calcul de la famille de tunnel(s) d'une arête : si elle participe à un ou
// plusieurs tunnels (encapIds non vide), la (les) arête(s) externe(s) du
// tunnel ET les flux internes qu'il transporte forment la famille à
// surligner. Fonctionne dans les deux sens : le CAPWAP montre son contenu,
// un flux interne montre par quel(s) tunnel(s) il est passé.
import Graph from "graphology"

export interface TunnelHighlight {
  edges: Set<string>
  nodes: Set<string>
  info: string
}

/** Retourne la famille de tunnel(s) de `edgeId`, ou `null` si elle n'en a pas. */
export function computeTunnelHighlight(
  graph: Graph,
  edgeId: string,
  pinnedEdgeId: string | null
): TunnelHighlight | null {
  if (!graph.hasEdge(edgeId)) return null
  const ids: string[] = graph.getEdgeAttribute(edgeId, "encapIds") || []
  if (!ids.length) return null

  const wanted = new Set(ids)
  const edges = new Set<string>()
  const nodes = new Set<string>()
  graph.forEachEdge((edge, attrs, source, target) => {
    const edgeIds: string[] = attrs.encapIds || []
    if (!edgeIds.some((id) => wanted.has(id))) return
    edges.add(edge)
    nodes.add(source)
    nodes.add(target)
  })

  const idLabel = ids.length === 1 ? `tunnel ${ids[0].slice(0, 8)}…` : `${ids.length} tunnels`
  const pin = pinnedEdgeId === edgeId ? "📌 " : ""
  return { edges, nodes, info: `${pin}${idLabel} — ${edges.size} flux liés` }
}

/** Une arête participe-t-elle à au moins un tunnel (donc épinglable) ? */
export function edgeHasTunnel(graph: Graph, edgeId: string): boolean {
  return graph.hasEdge(edgeId) && (graph.getEdgeAttribute(edgeId, "encapIds") || []).length > 0
}
