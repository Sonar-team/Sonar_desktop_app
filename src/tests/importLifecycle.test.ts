import { deepStrictEqual as assertEquals } from "node:assert/strict";

import { finalizeImportState } from "../components/AnalyseView/panels/import/importLifecycle.ts";

Deno.test("finalizeImportState libère toujours l'UI avant un refresh en erreur", async () => {
  let busy = true;
  const calls: string[] = [];
  let reported: unknown;
  const failure = new Error("backend indisponible");

  await finalizeImportState(
    () => {
      busy = false;
      calls.push("released");
    },
    () => {
      calls.push(`refresh:${busy}`);
      return Promise.reject(failure);
    },
    (error) => {
      reported = error;
      calls.push(`reported:${busy}`);
    },
  );

  assertEquals(busy, false);
  assertEquals(reported, failure);
  assertEquals(calls, ["released", "refresh:false", "reported:false"]);
});

Deno.test("finalizeImportState rafraîchit aussi après un succès", async () => {
  let busy = true;
  let refreshed = false;

  await finalizeImportState(
    () => {
      busy = false;
    },
    () => {
      refreshed = true;
      return Promise.resolve();
    },
    () => {
      throw new Error("aucune erreur attendue");
    },
  );

  assertEquals(busy, false);
  assertEquals(refreshed, true);
});
