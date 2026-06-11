import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { SaveAsCsv, SaveAsXlsx, triggerSave } from "../utils/save.js";

function mockTauri(savePath) {
  const calls = [];

  globalThis.window = globalThis;
  globalThis.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args, options) => {
      calls.push({ cmd, args, options });

      if (cmd === "plugin:dialog|save") {
        return savePath;
      }

      return null;
    },
  };

  return calls;
}

Deno.test("SaveAsCsv - should save CSV file successfully", async () => {
  const calls = mockTauri("mock/file.csv");

  const result = await SaveAsCsv(
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, true);
  assertEquals(calls.at(-1)?.cmd, "save_packets_to_csv");
});

Deno.test("SaveAsCsv - should return false if user cancels save dialog", async () => {
  const calls = mockTauri(null);

  const result = await SaveAsCsv(
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, false);
  assertEquals(calls.map(({ cmd }) => cmd), ["plugin:dialog|save"]);
});

Deno.test("SaveAsXlsx - should save XLSX file successfully", async () => {
  const calls = mockTauri("mock/file.xlsx");

  const result = await SaveAsXlsx(
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, true);
  assertEquals(calls.at(-1)?.cmd, "save_packets_to_excel");
});

Deno.test("SaveAsXlsx - should return false if user cancels save dialog", async () => {
  const calls = mockTauri(null);

  const result = await SaveAsXlsx(
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, false);
  assertEquals(calls.map(({ cmd }) => cmd), ["plugin:dialog|save"]);
});

Deno.test("triggerSave - should trigger CSV save successfully", async () => {
  const calls = mockTauri("mock/file.csv");

  const result = await triggerSave(
    "csv",
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, true);
  assertEquals(calls.map(({ cmd }) => cmd), [
    "plugin:dialog|save",
    "save_packets_to_csv",
    "plugin:dialog|message",
  ]);
});

Deno.test("triggerSave - should trigger XLSX save successfully", async () => {
  const calls = mockTauri("mock/file.xlsx");

  const result = await triggerSave(
    "xlsx",
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, true);
  assertEquals(calls.map(({ cmd }) => cmd), [
    "plugin:dialog|save",
    "save_packets_to_excel",
    "plugin:dialog|message",
  ]);
});

Deno.test("triggerSave - should return false if user cancels save dialog", async () => {
  const calls = mockTauri(null);

  const result = await triggerSave(
    "csv",
    () => "20240311",
    "CONFIDENTIAL",
    "INSTALL_001",
  );
  assertEquals(result, false);
  assertEquals(calls.map(({ cmd }) => cmd), [
    "plugin:dialog|save",
    "plugin:dialog|message",
  ]);
});
