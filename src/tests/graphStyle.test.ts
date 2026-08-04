// Tests des helpers purs de style/mise en forme du graphe réseau
// (src/components/AnalyseView/graph/graphStyle.ts) : couleurs, tailles
// proportionnelles au trafic, formatage et courbure des arêtes parallèles.
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { DEFAULT_EDGE_CURVATURE } from "@sigma/edge-curve";

import {
  DUPLICATE_IP_BORDER_COLOR,
  MAC_CONFLICT_BORDER_COLOR,
  NODE_SIZE_MAX,
  NODE_SIZE_MIN,
  brighten,
  darken,
  edgeAttributes,
  edgeKey,
  edgeLabelFor,
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

Deno.test("nodeSizeFor - distingue les gros volumes puis plafonne à la taille maximale", () => {
  const small = nodeSizeFor(1_000);
  const big = nodeSizeFor(1_000_000);
  const veryBig = nodeSizeFor(1_000_000_000);
  const huge = nodeSizeFor(1e18);

  assertEquals(small < big, true, `small=${small} big=${big}`);
  assertEquals(big < veryBig, true, `big=${big} veryBig=${veryBig}`);
  assertEquals(veryBig > 40, true, `veryBig=${veryBig}`);
  assertEquals(huge, NODE_SIZE_MAX);
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

Deno.test("nodeAttributes - VLAN affiché dans le libellé et exposé en attribut (#154)", () => {
  const attrs = nodeAttributes(node({ name: "192.168.1.10", vlan_id: 42 }));
  assertEquals(attrs.vlanId, 42);
  assertEquals(attrs.label, "192.168.1.10 (VLAN 42)");

  const untagged = nodeAttributes(node({ name: "192.168.1.10" }));
  assertEquals(untagged.vlanId, null);
  assertEquals(untagged.label, "192.168.1.10");
});

Deno.test("nodeAttributes - IP dupliquée -> bordure ambre, le conflit MAC garde priorité (#154)", () => {
  const attrs = nodeAttributes(node({ duplicate_ip: true }));
  assertEquals(attrs.duplicateIp, true);
  assertEquals(attrs.borderColor, DUPLICATE_IP_BORDER_COLOR);

  const both = nodeAttributes(node({
    duplicate_ip: true,
    macs: ["aa:bb:cc:dd:ee:ff", "11:22:33:44:55:66"],
  }));
  assertEquals(both.borderColor, MAC_CONFLICT_BORDER_COLOR);
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

Deno.test("edgeLabelFor - ports masqués -> seulement le protocole", () => {
  assertEquals(edgeLabelFor({ protocol: "TCP", ports: [80] }, false), "TCP");
});

Deno.test("edgeLabelFor - ports affichés, liste de ports -> protocole + ports triés", () => {
  assertEquals(edgeLabelFor({ protocol: "TCP", ports: [80, 443] }, true), "TCP :80,443");
});

Deno.test("edgeLabelFor - ports affichés, ports dynamiques mêlés à des ports listés -> suffixe …", () => {
  assertEquals(
    edgeLabelFor({ protocol: "TCP", ports: [80], has_dynamic_ports: true }, true),
    "TCP :80,…"
  );
});

Deno.test("edgeLabelFor - ports affichés, uniquement dynamiques -> pas de liste", () => {
  assertEquals(edgeLabelFor({ protocol: "TCP", ports: [], has_dynamic_ports: true }, true), "TCP :…");
});

Deno.test("edgeLabelFor - ports affichés, ports source/destination sans liste -> flèche", () => {
  assertEquals(
    edgeLabelFor({ protocol: "TCP", ports: [], source_port: 51234, destination_port: 443 }, true),
    "TCP 51234→443"
  );
});

Deno.test("edgeLabelFor - ports affichés, rien à montrer -> seulement le protocole", () => {
  assertEquals(edgeLabelFor({ protocol: "ICMP", ports: [] }, true), "ICMP");
});
