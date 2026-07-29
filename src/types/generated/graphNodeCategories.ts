// Généré par `cargo test export_ipc_graph_node_categories` — NE PAS ÉDITER À LA MAIN.
// Couleurs observées en faisant passer chaque catégorie par GraphData.
import type { IpType } from "./IpType";

export type GraphNodeCategory = Exclude<IpType, "Unknown"> | "Layer2";

export const GRAPH_NODE_CATEGORIES = ([
  {
    "kind": "Private",
    "color": "#8BC34A"
  },
  {
    "kind": "Public",
    "color": "#2196F3"
  },
  {
    "kind": "Multicast",
    "color": "#FFC107"
  },
  {
    "kind": "Loopback",
    "color": "#E53935"
  },
  {
    "kind": "Apipa",
    "color": "#FF9800"
  },
  {
    "kind": "LinkLocal",
    "color": "#FF5722"
  },
  {
    "kind": "Ula",
    "color": "#9C27B0"
  },
  {
    "kind": "Documentation",
    "color": "#9E9E9E"
  },
  {
    "kind": "Layer2",
    "color": "#00BCD4"
  }
]) as const satisfies ReadonlyArray<{
kind: GraphNodeCategory;
color: string;
}>;
