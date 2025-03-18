import { assertEquals } from "https://deno.land/std@0.218.2/testing/asserts.ts";
import { SaveAsCsv, SaveAsXlsx, triggerSave } from "../utils/save.js";

// Mocking dependencies
Deno.test("SaveAsCsv - should save CSV file successfully", async () => {
  globalThis.save = async () => "mock/file.csv";
  globalThis.invoke = async () => {};

  const result = await SaveAsCsv(() => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, true);
});

Deno.test("SaveAsCsv - should return false if user cancels save dialog", async () => {
  globalThis.save = async () => null;
  globalThis.invoke = async () => {};

  const result = await SaveAsCsv(() => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, false);
});

Deno.test("SaveAsXlsx - should save XLSX file successfully", async () => {
  globalThis.save = async () => "mock/file.xlsx";
  globalThis.invoke = async () => {};

  const result = await SaveAsXlsx(() => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, true);
});

Deno.test("SaveAsXlsx - should return false if user cancels save dialog", async () => {
  globalThis.save = async () => null;
  globalThis.invoke = async () => {};

  const result = await SaveAsXlsx(() => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, false);
});

Deno.test("triggerSave - should trigger CSV save successfully", async () => {
  globalThis.save = async () => "mock/file.csv";
  globalThis.invoke = async () => {};
  globalThis.message = async () => {};

  const result = await triggerSave("csv", () => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, true);
});

Deno.test("triggerSave - should trigger XLSX save successfully", async () => {
  globalThis.save = async () => "mock/file.xlsx";
  globalThis.invoke = async () => {};
  globalThis.message = async () => {};

  const result = await triggerSave("xlsx", () => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, true);
});

Deno.test("triggerSave - should return false if user cancels save dialog", async () => {
  globalThis.save = async () => null;
  globalThis.invoke = async () => {};
  globalThis.message = async () => {};

  const result = await triggerSave("csv", () => "20240311", "CONFIDENTIAL", "INSTALL_001");
  assertEquals(result, false);
});