// Tests du routage des fichiers déposés (drag & drop) sur le panneau
// d'import (src/components/AnalyseView/panels/import/fileTypes.ts).
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import { appendUniquePaths, routeDroppedPaths } from "../components/AnalyseView/panels/import/fileTypes.ts";

Deno.test("routeDroppedPaths - mode csv : premier .csv retenu", () => {
  const result = routeDroppedPaths(["notes.txt", "labels.csv", "autre.csv"], "csv");
  assertEquals(result, { kind: "csv", path: "labels.csv" });
});

Deno.test("routeDroppedPaths - mode csv : aucun .csv -> ignoré", () => {
  assertEquals(routeDroppedPaths(["capture.pcap", "notes.txt"], "csv"), { kind: "ignored" });
});

Deno.test("routeDroppedPaths - mode csv : insensible à la casse", () => {
  const result = routeDroppedPaths(["Labels.CSV"], "csv");
  assertEquals(result, { kind: "csv", path: "Labels.CSV" });
});

Deno.test("routeDroppedPaths - mode pcap : sépare captures et matrices", () => {
  const result = routeDroppedPaths(
    ["a.pcap", "b.pcapng", "c.cap", "m1.csv", "m2.xlsx", "notes.txt"],
    "pcap"
  );
  assertEquals(result, {
    kind: "files",
    pcaps: ["a.pcap", "b.pcapng", "c.cap"],
    matrices: ["m1.csv", "m2.xlsx"],
  });
});

Deno.test("routeDroppedPaths - mode pcap : rien de pertinent -> ignoré", () => {
  assertEquals(routeDroppedPaths(["notes.txt", "readme.md"], "pcap"), { kind: "ignored" });
});

Deno.test("routeDroppedPaths - mode pcap : seulement des matrices", () => {
  const result = routeDroppedPaths(["m.csv"], "pcap");
  assertEquals(result, { kind: "files", pcaps: [], matrices: ["m.csv"] });
});

// --- appendUniquePaths (#161) ----------------------------------------------

Deno.test("appendUniquePaths - écarte les chemins déjà listés", () => {
  const result = appendUniquePaths(["/tmp/a.pcap", "/tmp/b.pcap"], ["/tmp/b.pcap", "/tmp/c.pcap"]);
  assertEquals(result, ["/tmp/a.pcap", "/tmp/b.pcap", "/tmp/c.pcap"]);
});

Deno.test("appendUniquePaths - dédoublonne aussi au sein de la sélection", () => {
  const result = appendUniquePaths([], ["/tmp/a.pcap", "/tmp/a.pcap"]);
  assertEquals(result, ["/tmp/a.pcap"]);
});

Deno.test("appendUniquePaths - préserve l'ordre d'arrivée", () => {
  const result = appendUniquePaths(["/x/1.csv"], ["/x/2.csv", "/x/1.csv", "/x/3.csv"]);
  assertEquals(result, ["/x/1.csv", "/x/2.csv", "/x/3.csv"]);
});

Deno.test("appendUniquePaths - chemins distincts conservés (pas de normalisation implicite)", () => {
  const result = appendUniquePaths(["/tmp/a.pcap"], ["/tmp/A.pcap"]);
  assertEquals(result, ["/tmp/a.pcap", "/tmp/A.pcap"]);
});
