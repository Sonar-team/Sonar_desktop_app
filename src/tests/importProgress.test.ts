// Tests des calculs de progression d'import
// (src/components/AnalyseView/panels/import/importProgress.ts).
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import {
  progressFileNameOf,
  progressPercentOf,
} from "../components/AnalyseView/panels/import/importProgress.ts";
import type { ImportProgress } from "../types/capture.ts";

function progress(overrides: Partial<ImportProgress> = {}): ImportProgress {
  return {
    fileName: "capture.pcap",
    fileIndex: 1,
    filesTotal: 1,
    current: 0,
    total: 100,
    ...overrides,
  };
}

Deno.test("progressPercentOf - null -> 0", () => {
  assertEquals(progressPercentOf(null), 0);
});

Deno.test("progressPercentOf - filesTotal <= 0 -> 0", () => {
  assertEquals(progressPercentOf(progress({ filesTotal: 0 })), 0);
});

Deno.test("progressPercentOf - un seul fichier à mi-parcours", () => {
  assertEquals(progressPercentOf(progress({ current: 50, total: 100 })), 50);
});

Deno.test("progressPercentOf - fichier terminé sur un import multi-fichiers", () => {
  // Fichier 1/2 terminé : 50% du fichier courant + fichier déjà consommé.
  const result = progressPercentOf(
    progress({ fileIndex: 1, filesTotal: 2, current: 100, total: 100 })
  );
  assertEquals(result, 50);
});

Deno.test("progressPercentOf - total à 0 traité comme un fichier déjà complet", () => {
  // Évite une division par zéro : sans taille connue, on considère le fichier
  // courant comme consommé plutôt que de renvoyer NaN/Infinity.
  const result = progressPercentOf(progress({ fileIndex: 1, filesTotal: 2, current: 0, total: 0 }));
  assertEquals(result, 50);
});

Deno.test("progressPercentOf - borné entre 0 et 100", () => {
  assertEquals(progressPercentOf(progress({ current: -10, total: 100 })), 0);
  assertEquals(progressPercentOf(progress({ current: 1000, total: 100 })), 100);
});

Deno.test("progressFileNameOf - null -> chaîne vide", () => {
  assertEquals(progressFileNameOf(null), "");
});

Deno.test("progressFileNameOf - ne garde que le nom de fichier", () => {
  assertEquals(progressFileNameOf(progress({ fileName: "/home/user/capture.pcap" })), "capture.pcap");
  assertEquals(progressFileNameOf(progress({ fileName: "C:\\captures\\reseau.pcap" })), "reseau.pcap");
});

Deno.test("progressFileNameOf - sans séparateur, renvoie le nom tel quel", () => {
  assertEquals(progressFileNameOf(progress({ fileName: "capture.pcap" })), "capture.pcap");
});
