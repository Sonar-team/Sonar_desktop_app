// Construction des données affichées dans le bandeau « nœud sélectionné » à
// partir des attributs graphology : snapshot pour l'édition de label, et
// résumé texte (degré, trafic, protocoles…).
import Graph from "graphology"
import { NodeData } from "../../../types/capture"
import { formatBytes } from "./graphStyle"

/** Snapshot des attributs d'un nœud, utilisé par le bandeau bas et l'édition de label. */
export function nodeSnapshot(graph: Graph, nodeId: string): NodeData | null {
  if (!graph.hasNode(nodeId)) return null
  const attrs = graph.getNodeAttributes(nodeId)
  return {
    id: nodeId,
    name: attrs.name,
    mac: attrs.mac,
    ip: attrs.ip,
    color: attrs.color,
    label: attrs.rawLabel,
  }
}

/** Résumé texte des infos d'un nœud (degré, trafic, protocoles…) pour le bandeau bas. */
export function buildNodeInfos(graph: Graph, nodeId: string): string[] {
  if (!graph.hasNode(nodeId)) return ["Nœud introuvable"]
  const n = graph.getNodeAttributes(nodeId)

  const protos = new Set<string>()
  let degree = 0
  let packets = 0
  let bytes = 0
  graph.forEachEdge(nodeId, (_edge, attrs) => {
    degree++
    if (attrs.protocol) protos.add(String(attrs.protocol))
    packets += attrs.count || 0
    bytes += attrs.total_bytes || 0
  })

  // Plusieurs MAC observées pour cette IP : anomalie mise en évidence.
  const macInfo = n.macConflict
    ? `⚠ MACs multiples (${n.macs.length}): ${n.macs.join(", ")}`
    : `MAC: ${n.mac ?? ""}`

  return [
    `ID: ${nodeId}`,
    `Nom: ${n.name ?? ""}`,
    `Label: ${n.rawLabel || "N/A"}`,
    macInfo,
    `IP: ${n.ip ?? ""}`,
    `Couleur: ${n.color}`,
    `Degré: ${degree}`,
    `Trafic: ${formatBytes(bytes)} (${packets} paquets)`,
    `Protocoles: ${[...protos].join(", ") || "—"}`,
  ]
}
