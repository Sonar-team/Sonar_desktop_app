// Tests des helpers purs de style/mise en forme du graphe réseau
// (src/components/AnalyseView/graph/graphStyle.ts) : couleurs, tailles
// proportionnelles au trafic, formatage et courbure des arêtes parallèles.
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { DEFAULT_EDGE_CURVATURE } from "@sigma/edge-curve";

import {
  MAC_CONFLICT_BORDER_COLOR,
  NODE_SIZE_MIN,
  brighten,
  darken,
  edgeAttributes,
  edgeKey,
  edgeSizeFor,
  formatBytes,
  getCurvature,
  jitterAround,
  nodeAttributes,
  nodeSizeFor,
} from "../components/AnalyseView/graph/graphStyle.ts";
import type { Edge, EdgeData, Node } from "../types/capture.ts";

function node(overrides: Partial<Node> = {}): Node {
  return {
    id: "n",
    name: "n",
    color: "#2196F3",
    mac: "",
    macs: [],
    ip: "",
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

// --- Couleurs ----------------------------------------------------------------

Deno.test("darken - assombrit vers le noir au facteur donné", () => {
  assertEquals(darken("#2196F3", 0.25), "#1870b6");
});

Deno.test("darken - le noir reste noir", () => {
  assertEquals(darken("#000000"), "#000000");
});

Deno.test("darken - facteur par défaut (0.2) sur blanc", () => {
  assertEquals(darken("#ffffff"), "#cccccc");
});

Deno.test("brighten - éclaircit vers le blanc au facteur donné", () => {
  assertEquals(brighten("#2196F3"), "#47bcff");
});

Deno.test("brighten - clampe à blanc plutôt que de déborder", () => {
  assertEquals(brighten("#ffffff", 0.5), "#ffffff");
});

Deno.test("brighten - le noir s'éclaircit sans déborder", () => {
  assertEquals(brighten("#000000", 0.5), "#7f7f7f");
});

// --- Clé d'arête ---------------------------------------------------------------

Deno.test("edgeKey - concatène source, cible et protocole", () => {
  const e: EdgeData = { source: "a", target: "b", label: "TCP" };
  assertEquals(edgeKey(e), "a__b__TCP");
});

// --- Positionnement ------------------------------------------------------------

Deno.test("jitterAround - reste dans la couronne [0.4*r, r] autour du point", () => {
  for (let i = 0; i < 50; i++) {
    const { x, y } = jitterAround(100, 200, 120);
    const dist = Math.hypot(x - 100, y - 200);
    assertEquals(dist >= 120 * 0.4 - 1e-9 && dist <= 120 + 1e-9, true, `dist=${dist}`);
  }
});

// --- Tailles proportionnelles au trafic -----------------------------------------

Deno.test("nodeSizeFor - trafic nul -> taille minimale", () => {
  assertEquals(nodeSizeFor(0), NODE_SIZE_MIN);
});

Deno.test("nodeSizeFor - croît avec le trafic puis plafonne à 18", () => {
  const small = nodeSizeFor(1_000);
  const big = nodeSizeFor(1_000_000);
  const huge = nodeSizeFor(1e12);

  assertEquals(small < big, true, `small=${small} big=${big}`);
  assertEquals(huge, 18);
});

Deno.test("edgeSizeFor - trafic nul -> taille minimale, plafonne à 7", () => {
  assertEquals(edgeSizeFor(0), 1.2);
  assertEquals(edgeSizeFor(1e12), 7);
});

// --- Formatage des octets --------------------------------------------------------

Deno.test("formatBytes - sous 1024 octets -> unité 'o'", () => {
  assertEquals(formatBytes(0), "0 o");
  assertEquals(formatBytes(1023), "1023 o");
});

Deno.test("formatBytes - kilo-octets", () => {
  assertEquals(formatBytes(1024), "1.0 ko");
  assertEquals(formatBytes(2048), "2.0 ko");
});

Deno.test("formatBytes - méga-octets", () => {
  assertEquals(formatBytes(1024 * 1024), "1.0 Mo");
});

Deno.test("formatBytes - giga-octets, deux décimales", () => {
  assertEquals(formatBytes(1024 * 1024 * 1024 * 2.5), "2.50 Go");
});

// --- Courbure des arêtes parallèles ----------------------------------------------

Deno.test("getCurvature - une seule arête (maxIndex 0) -> courbure par défaut", () => {
  assertEquals(getCurvature(0, 0), DEFAULT_EDGE_CURVATURE);
});

Deno.test("getCurvature - croît avec l'index, reste sous la courbure max", () => {
  const c1 = getCurvature(1, 3);
  const c2 = getCurvature(2, 3);
  const c3 = getCurvature(3, 3);
  assertEquals(c1 < c2 && c2 < c3, true, `c1=${c1} c2=${c2} c3=${c3}`);
});

// --- Attributs graphology ---------------------------------------------------------

Deno.test("nodeAttributes - couleur par défaut si absente, label retombe sur le nom", () => {
  const attrs = nodeAttributes(node({ id: "n1", name: "hote-a", color: "", label: null }));
  assertEquals(attrs.color, "#2196F3");
  assertEquals(attrs.label, "hote-a");
  assertEquals(attrs.rawLabel, "");
  assertEquals(attrs.macConflict, false);
});

Deno.test("nodeAttributes - plusieurs MAC -> conflit signalé par la bordure d'alerte", () => {
  const attrs = nodeAttributes(node({ macs: ["aa:bb:cc:dd:ee:ff", "11:22:33:44:55:66"] }));
  assertEquals(attrs.macConflict, true);
  assertEquals(attrs.borderColor, MAC_CONFLICT_BORDER_COLOR);
});

Deno.test("edgeAttributes - champs par défaut sur une arête minimale", () => {
  const attrs = edgeAttributes(edge({ label: "TCP", totalBytes: 0 }));
  assertEquals(attrs.protocol, "TCP");
  assertEquals(attrs.ports, []);
  assertEquals(attrs.has_dynamic_ports, false);
  assertEquals(attrs.bidir, false);
  assertEquals(attrs.encapIds, []);
  assertEquals(attrs.size, edgeSizeFor(0));
});

Deno.test("edgeAttributes - transmet ports/tunnels tels quels", () => {
  const attrs = edgeAttributes(
    edge({ ports: [80, 443], hasDynamicPorts: true, encapIds: ["enc-1"], totalBytes: 12345 })
  );
  assertEquals(attrs.ports, [80, 443]);
  assertEquals(attrs.has_dynamic_ports, true);
  assertEquals(attrs.encapIds, ["enc-1"]);
  assertEquals(attrs.total_bytes, 12345);
});
