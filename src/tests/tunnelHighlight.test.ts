// Tests du calcul de la famille de tunnel(s) d'une arête survolée/épinglée
// (src/components/AnalyseView/graph/tunnelHighlight.ts).
import { deepStrictEqual as assertEquals } from "node:assert/strict";
import Graph from "graphology";

import {
  computeTunnelHighlight,
  edgeHasTunnel,
} from "../components/AnalyseView/graph/tunnelHighlight.ts";

function newGraph(): Graph {
  const g = new Graph({ multi: true, type: "directed" });
  g.addNode("capwapA");
  g.addNode("capwapB");
  g.addNode("client");
  g.addNode("server");
  g.addNode("autre");
  // Arête externe du tunnel (CAPWAP), et le flux interne qu'elle transporte,
  // partagent le même encap_id.
  g.addEdgeWithKey("tunnel", "capwapA", "capwapB", { encapIds: ["enc-1"] });
  g.addEdgeWithKey("inner", "client", "server", { encapIds: ["enc-1"] });
  g.addEdgeWithKey("hors-tunnel", "autre", "client", { encapIds: [] });
  return g;
}

Deno.test("computeTunnelHighlight - arête inexistante -> null", () => {
  const g = newGraph();
  assertEquals(computeTunnelHighlight(g, "fantome", null), null);
});

Deno.test("computeTunnelHighlight - arête sans tunnel -> null", () => {
  const g = newGraph();
  assertEquals(computeTunnelHighlight(g, "hors-tunnel", null), null);
});

Deno.test("computeTunnelHighlight - survol du CAPWAP montre le flux interne", () => {
  const g = newGraph();
  const result = computeTunnelHighlight(g, "tunnel", null);

  assertEquals(result?.edges, new Set(["tunnel", "inner"]));
  assertEquals(result?.nodes, new Set(["capwapA", "capwapB", "client", "server"]));
  assertEquals(result?.info, "tunnel enc-1… — 2 flux liés");
});

Deno.test("computeTunnelHighlight - survol du flux interne montre le tunnel (sens inverse)", () => {
  const g = newGraph();
  const result = computeTunnelHighlight(g, "inner", null);

  assertEquals(result?.edges, new Set(["tunnel", "inner"]));
});

Deno.test("computeTunnelHighlight - arête épinglée : marqueur 📌 dans le libellé", () => {
  const g = newGraph();
  const result = computeTunnelHighlight(g, "tunnel", "tunnel");

  assertEquals(result?.info.startsWith("📌 "), true, result?.info);
});

Deno.test("computeTunnelHighlight - plusieurs tunnels : libellé au pluriel", () => {
  const g = new Graph({ multi: true, type: "directed" });
  g.addNode("a");
  g.addNode("b");
  g.addEdgeWithKey("e", "a", "b", { encapIds: ["enc-1", "enc-2"] });

  const result = computeTunnelHighlight(g, "e", null);

  assertEquals(result?.info, "2 tunnels — 1 flux liés");
});

Deno.test("edgeHasTunnel - vrai si encapIds non vide", () => {
  const g = newGraph();
  assertEquals(edgeHasTunnel(g, "tunnel"), true);
});

Deno.test("edgeHasTunnel - faux si encapIds vide ou absent", () => {
  const g = newGraph();
  assertEquals(edgeHasTunnel(g, "hors-tunnel"), false);
});

Deno.test("edgeHasTunnel - faux si l'arête n'existe pas", () => {
  const g = newGraph();
  assertEquals(edgeHasTunnel(g, "fantome"), false);
});
