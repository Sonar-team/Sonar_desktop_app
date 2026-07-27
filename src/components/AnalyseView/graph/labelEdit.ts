// Mise à jour optimiste du label d'un nœud : l'UI reflète immédiatement la
// saisie, mais si le backend refuse la mutation, le rollback retourné doit
// être appelé pour ne pas laisser affiché un label qui n'existe pas (#161).
import type Graph from "graphology"

export function applyOptimisticLabel(
  graph: Graph,
  nodeId: string,
  newLabel: string,
): () => void {
  const attrs = graph.getNodeAttributes(nodeId)
  const previous = { rawLabel: attrs.rawLabel, label: attrs.label }

  graph.mergeNodeAttributes(nodeId, {
    rawLabel: newLabel,
    label: newLabel || attrs.name || nodeId,
  })

  return () => {
    if (graph.hasNode(nodeId)) graph.mergeNodeAttributes(nodeId, previous)
  }
}
