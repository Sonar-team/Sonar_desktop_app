// Tests des formateurs d'erreurs internes à src/errors/capture.ts :
// transforment un `*ErrorKind` du contrat IPC en message français lisible.
// `displayCaptureError` est testé sous mockIPC : il doit accepter toute
// valeur `unknown` sans lever d'exception secondaire (#161).
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";

import {
  capList,
  displayCaptureError,
  handleExportError,
  handleImportError,
  handleLabelerror,
} from "../errors/capture.ts";
import type { ExportErrorKind } from "../types/generated/ExportErrorKind.ts";
import type { LabelErrorKind } from "../types/generated/LabelErrorKind.ts";
import type { PcapImportErrorKind } from "../types/generated/PcapImportErrorKind.ts";

// --- handleExportError -----------------------------------------------------

Deno.test("handleExportError - chemin vide", () => {
  const msg = handleExportError({ kind: "emptyPath" });
  assertEquals(msg, "Aucun chemin de fichier fourni pour l'export.");
});

Deno.test("handleExportError - io/csv/poisonError incluent le message backend", () => {
  assertEquals(
    handleExportError({ kind: "io", message: "disque plein" } as ExportErrorKind),
    "Erreur d'écriture du fichier exporté : disque plein"
  );
  assertEquals(
    handleExportError({ kind: "csv", message: "champ invalide" } as ExportErrorKind),
    "Erreur d'écriture CSV : champ invalide"
  );
  assertEquals(
    handleExportError({ kind: "poisonError", message: "verrou empoisonné" } as ExportErrorKind),
    "Erreur verrou pendant l'export : verrou empoisonné"
  );
});

Deno.test("handleExportError - logNotFound", () => {
  assertEquals(handleExportError({ kind: "logNotFound" }), "Le dossier de logs est introuvable.");
});

Deno.test("handleExportError - forme inattendue -> message générique", () => {
  const msg = handleExportError(null as unknown as ExportErrorKind);
  assertEquals(msg.startsWith("Erreur d'export inconnue"), true, msg);
});

// --- handleImportError -------------------------------------------------------

Deno.test("handleImportError - entrée vide explique que le relevé est préservé", () => {
  const err: PcapImportErrorKind = { kind: "missingInput", message: "l'import PCAP" };
  const msg = handleImportError(err);
  assertEquals(msg.includes("Aucun fichier sélectionné"), true, msg);
  assertEquals(msg.includes("relevé courant est inchangé"), true, msg);
});

Deno.test("handleImportError - openFileError inclut fichier et détail", () => {
  const err: PcapImportErrorKind = { kind: "openFileError", message: ["capture.pcap", "permission refusée"] };
  assertEquals(handleImportError(err), "Impossible d'ouvrir le fichier capture.pcap : permission refusée");
});

Deno.test("handleImportError - readPacketError signale l'annulation de l'import", () => {
  const err: PcapImportErrorKind = { kind: "readPacketError", message: ["capture.pcap", "paquet tronqué"] };
  const msg = handleImportError(err);
  assertEquals(msg.includes("Erreur de lecture dans capture.pcap"), true, msg);
  assertEquals(msg.includes("matrice courante est inchangée"), true, msg);
});

Deno.test("handleImportError - unsupportedLinkType", () => {
  const err: PcapImportErrorKind = { kind: "unsupportedLinkType", message: ["capture.pcap", "DLT_RAW"] };
  const msg = handleImportError(err);
  assertEquals(msg.includes("Type de liaison non supporté dans capture.pcap : DLT_RAW"), true, msg);
});

Deno.test("handleImportError - forme inattendue -> message générique", () => {
  const msg = handleImportError(undefined as unknown as PcapImportErrorKind);
  assertEquals(msg.startsWith("Erreur d'import inconnue"), true, msg);
});

// --- capList -------------------------------------------------------------------

Deno.test("capList - sous le plafond : tout est affiché, pas de troncature", () => {
  const result = capList([1, 2, 3], (n) => `#${n}`, 8);
  assertEquals(result, "#1\n#2\n#3");
});

Deno.test("capList - exactement au plafond : pas de troncature", () => {
  const result = capList([1, 2, 3], (n) => `#${n}`, 3);
  assertEquals(result, "#1\n#2\n#3");
});

Deno.test("capList - au-delà du plafond : troncature avec singulier", () => {
  const result = capList([1, 2, 3], (n) => `#${n}`, 2);
  assertEquals(result, "#1\n#2\n… et 1 autre");
});

Deno.test("capList - au-delà du plafond : pluriel au-delà de 1 restant", () => {
  const result = capList([1, 2, 3, 4, 5], (n) => `#${n}`, 2);
  assertEquals(result, "#1\n#2\n… et 3 autres");
});

// --- handleLabelerror ----------------------------------------------------------

Deno.test("handleLabelerror - invalidMacIpFormat : MAC et IP invalides listées", () => {
  const err: LabelErrorKind = {
    kind: "invalidMacIpFormat",
    message: [
      [[2, "pas-une-mac", "pas-une-mac,1.2.3.4,pc"]],
      [[3, "999.1.1.1", "aa:bb:cc:dd:ee:ff,999.1.1.1,pc"]],
    ],
  };
  const msg = handleLabelerror(err);
  assertEquals(msg.includes("MAC invalides (1)"), true, msg);
  assertEquals(msg.includes("IP invalides (1)"), true, msg);
  assertEquals(msg.includes("ligne 2: pas-une-mac"), true, msg);
  assertEquals(msg.includes("ligne 3: 999.1.1.1"), true, msg);
});

Deno.test("handleLabelerror - invalidMacIpFormat : seule la section non vide apparaît", () => {
  const err: LabelErrorKind = {
    kind: "invalidMacIpFormat",
    message: [[], [[1, "1.2.3.4.5", "raw"]]],
  };
  const msg = handleLabelerror(err);
  assertEquals(msg.includes("MAC invalides"), false, msg);
  assertEquals(msg.includes("IP invalides (1)"), true, msg);
});

Deno.test("handleLabelerror - labelLinesConflicts : conflits mac/label listés", () => {
  const err: LabelErrorKind = {
    kind: "labelLinesConflicts",
    message: [[], [[1, 2, "1.2.3.4", "pc-a", "pc-b", "ligne1", "ligne2"]]],
  };
  const msg = handleLabelerror(err);
  assertEquals(msg.includes("même IP, label différent (1)"), true, msg);
  assertEquals(msg.includes("pc-a <-> pc-b"), true, msg);
  assertEquals(msg.includes("<Importation impossible>"), true, msg);
});

Deno.test("handleLabelerror - invalidRowsFormat", () => {
  const err: LabelErrorKind = { kind: "invalidRowsFormat", message: [[4, "une,seule"]] };
  const msg = handleLabelerror(err);
  assertEquals(msg.includes("ligne 4: une,seule"), true, msg);
});

Deno.test("handleLabelerror - editRejected", () => {
  const err: LabelErrorKind = { kind: "editRejected", message: "clé déjà utilisée" };
  assertEquals(handleLabelerror(err), "Édition refusée : clé déjà utilisée");
});

Deno.test("handleLabelerror - forme inattendue -> message générique", () => {
  const msg = handleLabelerror(null as unknown as LabelErrorKind);
  assertEquals(msg.startsWith("Erreur de label inconnue"), true, msg);
});

// --- displayCaptureError : enveloppes hors contrat (#161) -------------------

/** Intercepte les invokes Tauri (dialogue + log) et collecte les messages. */
function withMockedDialog(run: () => Promise<void>): Promise<string[]> {
  const shown: string[] = [];
  mockIPC((cmd, args) => {
    if (cmd === "plugin:dialog|message") {
      shown.push(String((args as { message?: unknown })?.message ?? ""));
    }
    return null;
  });
  return run().finally(() => clearMocks()).then(() => shown);
}

Deno.test("displayCaptureError - chaîne brute : affichée sans exception", async () => {
  const shown = await withMockedDialog(() => displayCaptureError("panne backend"));
  assertEquals(shown, ["Erreur inattendue : panne backend"]);
});

Deno.test("displayCaptureError - null : affiché sans exception", async () => {
  const shown = await withMockedDialog(() => displayCaptureError(null));
  assertEquals(shown, ["Erreur inattendue : null"]);
});

Deno.test("displayCaptureError - objet sans kind : sérialisé", async () => {
  const shown = await withMockedDialog(() => displayCaptureError({ code: 42 }));
  assertEquals(shown, ['Erreur inattendue : {"code":42}']);
});

Deno.test("displayCaptureError - objet cyclique : ne lève pas", async () => {
  const cyclic: Record<string, unknown> = {};
  cyclic.self = cyclic;
  const shown = await withMockedDialog(() => displayCaptureError(cyclic));
  assertEquals(shown, ["Erreur inattendue : [object Object]"]);
});

Deno.test("displayCaptureError - Error : message repris", async () => {
  const shown = await withMockedDialog(() => displayCaptureError(new Error("refus")));
  assertEquals(shown, ["Erreur inattendue : refus"]);
});

Deno.test("displayCaptureError - enveloppe du contrat : parcours nominal intact", async () => {
  const shown = await withMockedDialog(() =>
    displayCaptureError({ kind: "tauri", message: "canal fermé" })
  );
  assertEquals(shown, ["Erreur Tauri : canal fermé"]);
});
