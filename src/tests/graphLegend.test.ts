// La légende consomme les sorties Rust générées et les constantes du rendu,
// sans lire le dépôt à l'exécution (les tests Deno restent sans --allow-read).
import "./helpers/domShims.ts"

import { deepStrictEqual as assertEquals } from "node:assert/strict"

import {
  GRAPH_RENDERING_LEGEND,
  NODE_CATEGORY_LABELS,
  NODE_COLOR_LEGEND,
} from "../components/AnalyseView/graph/graphLegend.ts"
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
  nodeAttributes,
} from "../components/AnalyseView/graph/graphStyle.ts"
import { GRAPH_NODE_CATEGORIES } from "../types/generated/graphNodeCategories.ts"

Deno.test("légende nœuds - reprend toutes les catégories/couleurs générées par Rust", () => {
  assertEquals(
    NODE_COLOR_LEGEND.map(({ kind, color }) => ({ kind, color })),
    GRAPH_NODE_CATEGORIES.map(({ kind, color }) => ({ kind, color }))
  )
  assertEquals(
    NODE_COLOR_LEGEND.map(({ kind }) => kind).toSorted(),
    Object.keys(NODE_CATEGORY_LABELS).toSorted()
  )
})

Deno.test("légende nœuds - remplissage et bordure normale correspondent au rendu", () => {
  for (const item of NODE_COLOR_LEGEND) {
    const attrs = nodeAttributes({
      id: item.kind,
      name: item.kind,
      color: item.color,
      mac: "",
      macs: [],
      ip: "",
      vlan_id: null,
      duplicate_ip: false,
      label: null,
    })
    assertEquals(attrs.color, item.color)
    assertEquals(attrs.borderColor, item.borderColor)
  }
})

Deno.test("légende du rendu - tailles, zoom, conflit MAC et estompage partagent les constantes Sigma", () => {
  assertEquals(GRAPH_RENDERING_LEGEND, {
    nodeSize: { min: NODE_SIZE_MIN, max: NODE_SIZE_MAX },
    edgeSize: { min: EDGE_SIZE_MIN, max: EDGE_SIZE_MAX },
    edgeLabelZoom: EDGE_LABEL_ZOOM,
    portLabelZoom: PORT_LABEL_ZOOM,
    macConflictBorderColor: MAC_CONFLICT_BORDER_COLOR,
    duplicateIpBorderColor: DUPLICATE_IP_BORDER_COLOR,
    tunnelDimEdgeColor: DIM_EDGE_COLOR,
    tunnelDimNodeColor: DIM_NODE_COLOR,
  })
})
