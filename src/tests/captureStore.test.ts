// Tests du store de capture (src/store/capture.ts) avec l'environnement
// Tauri simulé : mockIPC intercepte les invoke, et un vrai Channel Tauri
// (créé sous mock) permet de rejouer les événements que le backend enverrait.
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { createPinia, setActivePinia } from "pinia";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { Channel } from "@tauri-apps/api/core";

import { useCaptureStore } from "../store/capture.ts";
import type { CaptureEvent } from "../types/capture.ts";

/** Store neuf + Channel attaché ; retourne de quoi simuler le backend. */
function setupStoreWithChannel() {
  setActivePinia(createPinia());
  const store = useCaptureStore();
  const channel = new Channel<CaptureEvent>();
  store.setChannel(channel);
  // Le store a remplacé onmessage par son dispatcher : l'appeler simule
  // l'arrivée d'un événement backend.
  const emitFromBackend = (msg: unknown) => (channel.onmessage as (m: unknown) => void)(msg);
  return { store, emitFromBackend };
}

Deno.test("refreshHasData - interroge is_matrix_empty via invoke", async () => {
  mockIPC((cmd) => {
    if (cmd === "is_matrix_empty") return false; // matrice non vide
  });
  try {
    setActivePinia(createPinia());
    const store = useCaptureStore();
    assertEquals(store.hasData, false);

    await store.refreshHasData();
    assertEquals(store.hasData, true);
  } finally {
    clearMocks();
  }
});

Deno.test("refreshHasData - matrice vide -> hasData reste faux", async () => {
  mockIPC((cmd) => {
    if (cmd === "is_matrix_empty") return true;
  });
  try {
    setActivePinia(createPinia());
    const store = useCaptureStore();
    await store.refreshHasData();
    assertEquals(store.hasData, false);
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - graphBatch redistribué update par update", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const received: unknown[] = [];
    store.onGraphUpdate((u) => received.push(u));

    emitFromBackend({
      event: "graphBatch",
      data: { updates: [{ NewNode: { id: "a" } }, { NewEdge: { source: "a", target: "b" } }] },
    });

    assertEquals(received.length, 2);
    assertEquals(received[0], { NewNode: { id: "a" } });
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - graphSnapshot atteint les abonnés", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const snapshots: unknown[] = [];
    store.onGraphSnapshot((g) => snapshots.push(g));

    const graphData = { nodes: {}, edges: {} };
    emitFromBackend({ event: "graphSnapshot", data: { graph_data: graphData } });

    assertEquals(snapshots, [graphData]);
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - started promeut le filtre en attente", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    store.setActiveFilter("tcp");
    store.setPendingFilter("udp");

    emitFromBackend({ event: "started", data: {} });

    assertEquals(store.activeFilter, "udp");
    assertEquals(store.pendingFilter, "");
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - packetBatch alimente les deux familles d'abonnés", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const batches: unknown[][] = [];
    const packets: unknown[] = [];
    store.onPacketBatch((b) => batches.push(b));
    store.onPacket((p) => packets.push(p));

    emitFromBackend({ event: "packetBatch", data: { packets: [{ len: 1 }, { len: 2 }] } });

    assertEquals(batches.length, 1);
    assertEquals(batches[0].length, 2);
    assertEquals(packets.length, 2, "chaque paquet du batch est aussi émis individuellement");
  } finally {
    clearMocks();
  }
});

Deno.test("onStats - le désabonnement retire bien le listener", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const seen: unknown[] = [];
    const unsub = store.onStats((s) => seen.push(s));

    emitFromBackend({ event: "stats", data: { received: 1 } });
    unsub();
    emitFromBackend({ event: "stats", data: { received: 2 } });

    assertEquals(seen, [{ received: 1 }]);
  } finally {
    clearMocks();
  }
});
