// Données pures de la légende. Les couleurs de catégories viennent du graphe
// Rust (fichier généré) ; les autres valeurs viennent des helpers utilisés par
// Sigma, afin que la légende et le rendu partagent leurs sources.
import {
  GRAPH_NODE_CATEGORIES,
  type GraphNodeCategory,
} from "../../../types/generated/graphNodeCategories"
import {
  DIM_EDGE_COLOR,
  DIM_NODE_COLOR,
  DUPLICATE_IP_BORDER_COLOR,
  EDGE_LABEL_ZOOM,
  EDGE_SIZE_MAX,
  EDGE_SIZE_MIN,
  MAC_CONFLICT_BORDER_COLOR,
  NODE_SIZE_MAX,
  NODE_SIZE_MIN,
  PORT_LABEL_ZOOM,
  darken,
} from "./graphStyle"

export const NODE_CATEGORY_LABELS = Object.freeze({
  Private: "IP privée",
  Public: "IP publique",
  Multicast: "IP multicast",
  Loopback: "Boucle locale",
  Apipa: "APIPA",
  LinkLocal: "IPv6 lien-local",
  Ula: "IPv6 ULA",
  Documentation: "IP de documentation",
  Layer2: "Couche 2 (MAC)",
} satisfies Record<GraphNodeCategory, string>)

export const NODE_COLOR_LEGEND = Object.freeze(
  GRAPH_NODE_CATEGORIES.map(({ kind, color }) => ({
    kind,
    label: NODE_CATEGORY_LABELS[kind],
    color,
    borderColor: darken(color, 0.25),
  }))
)

export const GRAPH_RENDERING_LEGEND = Object.freeze({
  nodeSize: Object.freeze({ min: NODE_SIZE_MIN, max: NODE_SIZE_MAX }),
  edgeSize: Object.freeze({ min: EDGE_SIZE_MIN, max: EDGE_SIZE_MAX }),
  edgeLabelZoom: EDGE_LABEL_ZOOM,
  portLabelZoom: PORT_LABEL_ZOOM,
  macConflictBorderColor: MAC_CONFLICT_BORDER_COLOR,
  duplicateIpBorderColor: DUPLICATE_IP_BORDER_COLOR,
  tunnelDimEdgeColor: DIM_EDGE_COLOR,
  tunnelDimNodeColor: DIM_NODE_COLOR,
})
