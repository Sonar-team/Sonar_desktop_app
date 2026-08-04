// Tests des mutations du graphe réseau (graph/graphSync.ts) sur une vraie
// instance graphology : upserts, clés d'arêtes, tailles proportionnelles au
// trafic et arêtes parallèles.
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import Graph from "graphology";

import {
  refreshParallelEdges,
  upsertEdge,
  upsertNode,
} from "../components/AnalyseView/graph/graphSync.ts";
import type { Edge, Node } from "../types/capture.ts";

function newGraph(): Graph {
  return new Graph({ multi: true, type: "directed" });
}

// Objets `Node`/`Edge` complets (contrat généré, #142) avec des défauts
// neutres : chaque test ne précise que les champs qui l'intéressent, sans
// retomber sur des littéraux partiels que `upsertNode`/`upsertEdge` — typés
// strictement, plus de `any` — rejetteraient à la compilation.
function node(overrides: Partial<Node> = {}): Node {
  return {
    id: "n",
    name: "n",
    color: "#2196F3",
    mac: "",
    macs: [],
    ip: "",
    vlan_id: null,
    duplicate_ip: false,
    label: null,
    ...overrides,
  };
}

function edge(overrides: Partial<Edge> = {}): Edge {
  return {
    id: "e",
    source: "a",
    target: "b",
    label: "TCP",
    sourcePort: null,
    destinationPort: null,
    ports: [],
    hasDynamicPorts: false,
    bidir: false,
    count: 0,
    totalBytes: 0,
    encapIds: [],
    ...overrides,
  };
}

Deno.test("upsertNode - ajoute un nœud avec ses attributs dérivés", () => {
  const g = newGraph();
  const added = upsertNode(g, node({ id: "192.168.1.1", name: "hote-a", mac: "aa:bb", ip: "192.168.1.1" }));

  assertEquals(added, true);
  assertEquals(g.order, 1);
  const attrs = g.getNodeAttributes("192.168.1.1");
  assertEquals(attrs.name, "hote-a");
  // Sans label utilisateur, le label affiché retombe sur le nom.
  assertEquals(attrs.label, "hote-a");
  assertEquals(typeof attrs.x, "number");
  assertEquals(typeof attrs.y, "number");
});

Deno.test("upsertNode - un nœud existant est fusionné, pas dupliqué", () => {
  const g = newGraph();
  upsertNode(g, node({ id: "n1", name: "avant" }));
  const added = upsertNode(g, node({ id: "n1", name: "avant", label: "mon-pc" }));

  assertEquals(added, false);
  assertEquals(g.order, 1);
  assertEquals(g.getNodeAttribute("n1", "label"), "mon-pc");
});

Deno.test("upsertNode - id manquant refusé", () => {
  const g = newGraph();
  assertEquals(upsertNode(g, node({ id: "" })), false);
  assertEquals(upsertNode(g, null), false);
  assertEquals(g.order, 0);
});

Deno.test("upsertEdge - refuse les arêtes orphelines", () => {
  const g = newGraph();
  upsertNode(g, node({ id: "a" }));
  const added = upsertEdge(g, edge({ source: "a", target: "fantome", label: "TCP" }));

  assertEquals(added, false);
  assertEquals(g.size, 0);
});

Deno.test("upsertEdge - ajoute puis fusionne sur la clé source__cible__protocole", () => {
  const g = newGraph();
  upsertNode(g, node({ id: "a" }));
  upsertNode(g, node({ id: "b" }));

  const first = upsertEdge(g, edge({ source: "a", target: "b", label: "TCP", count: 1, totalBytes: 100 }));
  const second = upsertEdge(g, edge({ source: "a", target: "b", label: "TCP", count: 5, totalBytes: 900 }));

  assertEquals(first, true);
  assertEquals(second, false, "même flux -> fusion, pas de doublon");
  assertEquals(g.size, 1);
  // Le backend envoie des compteurs cumulés : la fusion remplace les valeurs.
  assertEquals(g.getEdgeAttribute("a__b__TCP", "total_bytes"), 900);
  assertEquals(g.getEdgeAttribute("a__b__TCP", "count"), 5);
});

Deno.test("upsertEdge - deux protocoles entre les mêmes hôtes = deux arêtes", () => {
  const g = newGraph();
  upsertNode(g, node({ id: "a" }));
  upsertNode(g, node({ id: "b" }));
  upsertEdge(g, edge({ source: "a", target: "b", label: "TCP" }));
  upsertEdge(g, edge({ source: "a", target: "b", label: "DNS" }));

  assertEquals(g.size, 2);
});

Deno.test("upsertEdge - la taille des nœuds suit le trafic cumulé", () => {
  const g = newGraph();
  upsertNode(g, node({ id: "a" }));
  upsertNode(g, node({ id: "b" }));
  upsertEdge(g, edge({ source: "a", target: "b", label: "TCP", totalBytes: 1 }));
  const small = g.getNodeAttribute("a", "size");

  upsertEdge(g, edge({ source: "a", target: "b", label: "TCP", totalBytes: 50_000_000 }));
  const big = g.getNodeAttribute("a", "size");

  assertEquals(big > small, true, `taille ${big} devrait dépasser ${small}`);
});

Deno.test("refreshParallelEdges - les arêtes parallèles deviennent courbes", () => {
  const g = newGraph();
  upsertNode(g, node({ id: "a" }));
  upsertNode(g, node({ id: "b" }));
  upsertEdge(g, edge({ source: "a", target: "b", label: "TCP" }));
  upsertEdge(g, edge({ source: "a", target: "b", label: "DNS" }));

  refreshParallelEdges(g);

  const types = g.edges().map((e) => g.getEdgeAttribute(e, "type"));
  assertEquals(types.includes("curved"), true, `types: ${types}`);
});
