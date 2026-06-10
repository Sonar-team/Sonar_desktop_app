import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { getCurrentDate, padZero } from "../utils/time.js";

// Test pour padZero
Deno.test("padZero ajoute un zéro devant les nombres inférieurs à 10", () => {
  assertEquals(padZero(5), "05");
  assertEquals(padZero(9), "09");
});

Deno.test("padZero ne modifie pas les nombres supérieurs ou égaux à 10", () => {
  assertEquals(padZero(10), "10");
  assertEquals(padZero(15), "15");
});

// Test pour getCurrentDate avec mock de Date
Deno.test("getCurrentDate retourne la date actuelle au format YYYYMMDD", () => {
  const RealDate = globalThis.Date;
  const mockDate = new Date(2024, 2, 10); // Mars (2 car indexé à 0), 10
  globalThis.Date = class extends Date {
    constructor(...args) {
      super(...(args.length ? args : [mockDate.getTime()])); // ✅ Appel correct de super()
    }
  };

  try {
    assertEquals(getCurrentDate(), "20240310");
  } finally {
    globalThis.Date = RealDate;
  }
});
