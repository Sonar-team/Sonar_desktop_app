// Tests de la construction des données du bandeau "nœud sélectionné"
// (src/components/AnalyseView/graph/nodeInfo.ts).
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import Graph from "graphology";

import { buildNodeInfos, nodeSnapshot } from "../components/AnalyseView/graph/nodeInfo.ts";

function newGraph(): Graph {
  const g = new Graph({ multi: true, type: "directed" });
  g.addNode("192.168.1.1", {
    name: "hote-a",
    mac: "aa:bb:cc:dd:ee:ff",
    macs: ["aa:bb:cc:dd:ee:ff"],
    macConflict: false,
    ip: "192.168.1.1",
    rawLabel: "PC Sonar",
    color: "#2196F3",
  });
  g.addNode("192.168.1.2", { name: "hote-b", mac: "", macs: [], macConflict: false, ip: "", rawLabel: "", color: "#000" });
  g.addNode("192.168.1.3", {
    name: "hote-c",
    mac: "11:11:11:11:11:11",
    macs: ["11:11:11:11:11:11", "22:22:22:22:22:22"],
    macConflict: true,
    ip: "192.168.1.3",
    rawLabel: "",
    color: "#f00",
  });
  return g;
}

Deno.test("nodeSnapshot - nœud inexistant -> null", () => {
  const g = newGraph();
  assertEquals(nodeSnapshot(g, "fantome"), null);
});

Deno.test("nodeSnapshot - mappe les attributs graphology vers NodeData", () => {
  const g = newGraph();
  assertEquals(nodeSnapshot(g, "192.168.1.1"), {
    id: "192.168.1.1",
    name: "hote-a",
    mac: "aa:bb:cc:dd:ee:ff",
    ip: "192.168.1.1",
    color: "#2196F3",
    label: "PC Sonar",
  });
});

Deno.test("buildNodeInfos - nœud inexistant -> message d'erreur", () => {
  const g = newGraph();
  assertEquals(buildNodeInfos(g, "fantome"), ["Nœud introuvable"]);
});

Deno.test("buildNodeInfos - agrège degré, trafic et protocoles des arêtes", () => {
  const g = newGraph();
  g.addEdgeWithKey("e1", "192.168.1.1", "192.168.1.2", {
    protocol: "TCP",
    count: 10,
    total_bytes: 1000,
  });
  g.addEdgeWithKey("e2", "192.168.1.2", "192.168.1.1", {
    protocol: "DNS",
    count: 2,
    total_bytes: 200,
  });

  const infos = buildNodeInfos(g, "192.168.1.1");

  assertEquals(infos.includes("Degré: 2"), true, infos.join(" | "));
  assertEquals(infos.some((line) => line.includes("Protocoles:") && line.includes("TCP") && line.includes("DNS")), true, infos.join(" | "));
  assertEquals(infos.some((line) => line.startsWith("Trafic:") && line.includes("12 paquets")), true, infos.join(" | "));
});

Deno.test("buildNodeInfos - nœud isolé : pas d'arête, pas de protocole", () => {
  const g = newGraph();
  const infos = buildNodeInfos(g, "192.168.1.2");

  assertEquals(infos.includes("Degré: 0"), true, infos.join(" | "));
  assertEquals(infos.some((line) => line === "Protocoles: —"), true, infos.join(" | "));
  assertEquals(infos.includes("Label: N/A"), true, infos.join(" | "));
});

Deno.test("buildNodeInfos - conflit de MAC signalé", () => {
  const g = newGraph();
  const infos = buildNodeInfos(g, "192.168.1.3");

  assertEquals(
    infos.some((line) => line.startsWith("⚠ MACs multiples (2)")),
    true,
    infos.join(" | ")
  );
});
