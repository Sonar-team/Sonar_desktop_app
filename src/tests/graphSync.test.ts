// Tests des mutations du graphe réseau (graph/graphSync.ts) sur une vraie
// instance graphology : upserts, clés d'arêtes, tailles proportionnelles au
// trafic, arêtes parallèles et normalisation des updates du backend.
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import Graph from "graphology";

import {
  normalizeGraphUpdate,
  refreshParallelEdges,
  upsertEdge,
  upsertNode,
} from "../components/AnalyseView/graph/graphSync.ts";

function newGraph(): Graph {
  return new Graph({ multi: true, type: "directed" });
}

Deno.test("upsertNode - ajoute un nœud avec ses attributs dérivés", () => {
  const g = newGraph();
  const added = upsertNode(g, { id: "192.168.1.1", name: "hote-a", mac: "aa:bb", ip: "192.168.1.1" });

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
  upsertNode(g, { id: "n1", name: "avant" });
  const added = upsertNode(g, { id: "n1", name: "avant", label: "mon-pc" });

  assertEquals(added, false);
  assertEquals(g.order, 1);
  assertEquals(g.getNodeAttribute("n1", "label"), "mon-pc");
});

Deno.test("upsertNode - id manquant refusé", () => {
  const g = newGraph();
  assertEquals(upsertNode(g, {}), false);
  assertEquals(upsertNode(g, null), false);
  assertEquals(g.order, 0);
});

Deno.test("upsertEdge - refuse les arêtes orphelines", () => {
  const g = newGraph();
  upsertNode(g, { id: "a" });
  const added = upsertEdge(g, { source: "a", target: "fantome", label: "TCP" });

  assertEquals(added, false);
  assertEquals(g.size, 0);
});

Deno.test("upsertEdge - ajoute puis fusionne sur la clé source__cible__protocole", () => {
  const g = newGraph();
  upsertNode(g, { id: "a" });
  upsertNode(g, { id: "b" });

  const first = upsertEdge(g, { source: "a", target: "b", label: "TCP", count: 1, total_bytes: 100 });
  const second = upsertEdge(g, { source: "a", target: "b", label: "TCP", count: 5, total_bytes: 900 });

  assertEquals(first, true);
  assertEquals(second, false, "même flux -> fusion, pas de doublon");
  assertEquals(g.size, 1);
  // Le backend envoie des compteurs cumulés : la fusion remplace les valeurs.
  assertEquals(g.getEdgeAttribute("a__b__TCP", "total_bytes"), 900);
  assertEquals(g.getEdgeAttribute("a__b__TCP", "count"), 5);
});

Deno.test("upsertEdge - deux protocoles entre les mêmes hôtes = deux arêtes", () => {
  const g = newGraph();
  upsertNode(g, { id: "a" });
  upsertNode(g, { id: "b" });
  upsertEdge(g, { source: "a", target: "b", label: "TCP" });
  upsertEdge(g, { source: "a", target: "b", label: "DNS" });

  assertEquals(g.size, 2);
});

Deno.test("upsertEdge - la taille des nœuds suit le trafic cumulé", () => {
  const g = newGraph();
  upsertNode(g, { id: "a" });
  upsertNode(g, { id: "b" });
  upsertEdge(g, { source: "a", target: "b", label: "TCP", total_bytes: 1 });
  const small = g.getNodeAttribute("a", "size");

  upsertEdge(g, { source: "a", target: "b", label: "TCP", total_bytes: 50_000_000 });
  const big = g.getNodeAttribute("a", "size");

  assertEquals(big > small, true, `taille ${big} devrait dépasser ${small}`);
});

Deno.test("refreshParallelEdges - les arêtes parallèles deviennent courbes", () => {
  const g = newGraph();
  upsertNode(g, { id: "a" });
  upsertNode(g, { id: "b" });
  upsertEdge(g, { source: "a", target: "b", label: "TCP" });
  upsertEdge(g, { source: "a", target: "b", label: "DNS" });

  refreshParallelEdges(g);

  const types = g.edges().map((e) => g.getEdgeAttribute(e, "type"));
  assertEquals(types.includes("curved"), true, `types: ${types}`);
});

Deno.test("normalizeGraphUpdate - formats du backend ramenés à la forme typée", () => {
  assertEquals(
    normalizeGraphUpdate({ NewNode: { id: "n" } }),
    { type: "NodeAdded", payload: { id: "n" } },
  );
  assertEquals(
    normalizeGraphUpdate({ NodeUpdated: { id: "n" } }),
    { type: "NodeUpdated", payload: { id: "n" } },
  );
  assertEquals(
    normalizeGraphUpdate({ NewEdge: { source: "a" } }),
    { type: "EdgeAdded", payload: { source: "a" } },
  );
  assertEquals(
    normalizeGraphUpdate({ update: { EdgeUpdated: { source: "a" } } }),
    { type: "EdgeUpdated", payload: { source: "a" } },
  );
  // Déjà normalisé : passe tel quel.
  const typed = { type: "NodeAdded", payload: { id: "n" } };
  assertEquals(normalizeGraphUpdate(typed), typed);
  // Inconnu ou vide : null.
  assertEquals(normalizeGraphUpdate({ Autre: {} }), null);
  assertEquals(normalizeGraphUpdate(null), null);
});
