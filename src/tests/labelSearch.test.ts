// Tests du filtrage plein-texte de la table des labels importés
// (src/components/AnalyseView/panels/import/labelSearch.ts).
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import { filterLabelRows } from "../components/AnalyseView/panels/import/labelSearch.ts";

const rows: [string, string, string][] = [
  ["aa:bb:cc:dd:ee:ff", "192.168.1.1", "PC Sonar"],
  ["11:22:33:44:55:66", "192.168.1.2", "Imprimante"],
  ["77:88:99:aa:bb:cc", "10.0.0.5", "Passerelle"],
];

Deno.test("filterLabelRows - recherche vide renvoie tout", () => {
  assertEquals(filterLabelRows(rows, ""), rows);
});

Deno.test("filterLabelRows - filtre sur le label", () => {
  assertEquals(filterLabelRows(rows, "sonar"), [rows[0]]);
});

Deno.test("filterLabelRows - filtre sur l'IP", () => {
  assertEquals(filterLabelRows(rows, "10.0.0"), [rows[2]]);
});

Deno.test("filterLabelRows - filtre sur la MAC, insensible à la casse", () => {
  assertEquals(filterLabelRows(rows, "AA:BB:CC:DD"), [rows[0]]);
});

Deno.test("filterLabelRows - aucune correspondance -> tableau vide", () => {
  assertEquals(filterLabelRows(rows, "introuvable"), []);
});
