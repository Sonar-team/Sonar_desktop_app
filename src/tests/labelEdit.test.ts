// Tests de la mise à jour optimiste des labels et de son rollback (#161)
// (src/components/AnalyseView/graph/labelEdit.ts).
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import Graph from "graphology";

import { applyOptimisticLabel } from "../components/AnalyseView/graph/labelEdit.ts";

function newGraph(): Graph {
  const g = new Graph({ multi: true, type: "directed" });
  g.addNode("192.168.1.1", {
    name: "hote-a",
    ip: "192.168.1.1",
    rawLabel: "Ancien label",
    label: "Ancien label",
  });
  return g;
}

Deno.test("applyOptimisticLabel - applique le nouveau label immédiatement", () => {
  const g = newGraph();
  applyOptimisticLabel(g, "192.168.1.1", "Nouveau");

  assertEquals(g.getNodeAttribute("192.168.1.1", "rawLabel"), "Nouveau");
  assertEquals(g.getNodeAttribute("192.168.1.1", "label"), "Nouveau");
});

Deno.test("applyOptimisticLabel - label vide : retombe sur le nom du nœud", () => {
  const g = newGraph();
  applyOptimisticLabel(g, "192.168.1.1", "");

  assertEquals(g.getNodeAttribute("192.168.1.1", "rawLabel"), "");
  assertEquals(g.getNodeAttribute("192.168.1.1", "label"), "hote-a");
});

Deno.test("applyOptimisticLabel - rollback restaure les attributs d'origine", () => {
  const g = newGraph();
  const rollback = applyOptimisticLabel(g, "192.168.1.1", "Nouveau");

  rollback();

  assertEquals(g.getNodeAttribute("192.168.1.1", "rawLabel"), "Ancien label");
  assertEquals(g.getNodeAttribute("192.168.1.1", "label"), "Ancien label");
});

Deno.test("applyOptimisticLabel - rollback sans effet si le nœud a disparu", () => {
  const g = newGraph();
  const rollback = applyOptimisticLabel(g, "192.168.1.1", "Nouveau");

  g.dropNode("192.168.1.1");
  rollback();

  assertEquals(g.hasNode("192.168.1.1"), false);
});
