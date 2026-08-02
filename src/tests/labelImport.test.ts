// Tests de la classification des erreurs d'import de labels
// (src/utils/labelImport.ts) : transformation des erreurs brutes du backend
// en cas typés pour le dialogue de conflits.
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import { classifyLabelImportError } from "../utils/labelImport.ts";

Deno.test("classifyLabelImportError - erreur non label -> null", () => {
  assertEquals(classifyLabelImportError({ kind: "io", message: "disque plein" }), null);
  assertEquals(classifyLabelImportError(null), null);
  assertEquals(classifyLabelImportError(undefined), null);
  assertEquals(classifyLabelImportError("boom"), null);
});

Deno.test("classifyLabelImportError - formats MAC/IP invalides", () => {
  const invalidMac = [[3, "aa:bb", "aa:bb,1.2.3.4,pc"]];
  const invalidIp = [[5, "999.1.1.1", "aa:bb:cc:dd:ee:ff,999.1.1.1,pc"]];
  const result = classifyLabelImportError({
    kind: "label",
    message: { kind: "invalidMacIpFormat", message: [invalidMac, invalidIp] },
  });

  assertEquals(result, { kind: "invalidMacIpFormat", invalidMac, invalidIp });
});

Deno.test("classifyLabelImportError - conflits entre lignes", () => {
  const conflict = [[1, 2, "1.2.3.4", "pc-a", "pc-b", "ligne1", "ligne2"]];
  const result = classifyLabelImportError({
    kind: "label",
    message: { kind: "labelLinesConflicts", message: [[], conflict] },
  });

  assertEquals(result, {
    kind: "labelLinesConflicts",
    sameIpDiffMac: [],
    sameIpDiffLabel: conflict,
  });
});

Deno.test("classifyLabelImportError - lignes malformées", () => {
  const invalidLines = [[4, "une,seule"]];
  const result = classifyLabelImportError({
    kind: "label",
    message: { kind: "invalidRowsFormat", message: invalidLines },
  });

  assertEquals(result, { kind: "invalidRowsFormat", invalidLines });
});

Deno.test("classifyLabelImportError - sous-type de label inconnu -> null", () => {
  const result = classifyLabelImportError({
    kind: "label",
    message: { kind: "autreChose", message: [] },
  });
  assertEquals(result, null);
});

Deno.test("classifyLabelImportError - payload label imbriqué non conforme (null) -> null, ne lève pas (régression)", () => {
  assertEquals(classifyLabelImportError({ kind: "label", message: null }), null);
  assertEquals(classifyLabelImportError({ kind: "label", message: "boom" }), null);
});
