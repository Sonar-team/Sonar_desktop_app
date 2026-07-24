// Tests du store de capture (src/store/capture.ts) avec l'environnement
// Tauri simulé : mockIPC intercepte les invoke, et un vrai Channel Tauri
// (créé sous mock) permet de rejouer les événements que le backend enverrait.
import "./helpers/domShims.ts";

import { deepStrictEqual as assertEquals } from "node:assert/strict";
import { createPinia, setActivePinia } from "pinia";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { Channel } from "@tauri-apps/api/core";

import { useCaptureStore } from "../store/capture.ts";
import type {
  CapturedPacket,
  CaptureEvent,
  GraphData,
  GraphUpdate,
  Stats,
} from "../types/capture.ts";
import {
  graphBatchFixture,
  graphSnapshotFixture,
  importProgressFixture,
  packetBatchMinimalFixture,
  startedFixture,
  statsFixture,
} from "../types/generated/captureEventFixtures.ts";

type EventData<E extends CaptureEvent["event"]> = Extract<CaptureEvent, { event: E }>["data"];

// Les fixtures ci-dessus sont du JSON réellement produit par `CaptureEvent`
// côté Rust (cargo test export_ipc_fixtures) : les réutiliser ici — plutôt
// que des littéraux à moitié remplis retapés à la main — garantit que ces
// tests exercent le store avec la forme *réelle* du contrat, `session_id`
// et champs de paquet compris, au lieu de passer par un `unknown` qui
// masquerait tout événement legacy ou incomplet (#142).
function clone<T>(value: T): T {
  return structuredClone(value);
}

/** Store neuf + Channel attaché ; retourne de quoi simuler le backend. */
function setupStoreWithChannel() {
  setActivePinia(createPinia());
  const store = useCaptureStore();
  const channel = new Channel<CaptureEvent>();
  store.setChannel(channel);
  // Le store a remplacé onmessage par son dispatcher : l'appeler simule
  // l'arrivée d'un événement backend.
  const emitFromBackend = (msg: CaptureEvent) => (channel.onmessage as (m: CaptureEvent) => void)(msg);
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
    const received: GraphUpdate[] = [];
    store.onGraphUpdate((u: GraphUpdate) => received.push(u));

    const batch = clone(graphBatchFixture);
    // sessionId 0 = hors session (cf. store/capture.ts) : passe le filtre
    // de session sans dépendre du sessionId courant du store, non pertinent
    // pour ce test de redistribution.
    batch.data.sessionId = 0;
    emitFromBackend(batch);

    assertEquals(received.length, batch.data.updates.length);
    assertEquals(received[0], batch.data.updates[0]);
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - graphSnapshot atteint les abonnés", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const snapshots: GraphData[] = [];
    store.onGraphSnapshot((g: GraphData) => snapshots.push(g));

    const snapshot = clone(graphSnapshotFixture);
    emitFromBackend(snapshot);

    assertEquals(snapshots, [snapshot.data.graphData]);
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - importProgress met à jour la progression typée", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const progress = clone(importProgressFixture);

    emitFromBackend(progress);

    assertEquals(store.importProgress, progress.data);
    store.clearImportProgress();
    assertEquals(store.importProgress, null);
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

    emitFromBackend(clone(startedFixture));

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
    const batches: CapturedPacket[][] = [];
    const packets: CapturedPacket[] = [];
    store.onPacketBatch((b: CapturedPacket[]) => batches.push(b));
    store.onPacket((p: CapturedPacket) => packets.push(p));

    const batch = clone(packetBatchMinimalFixture);
    batch.data.sessionId = 0; // hors session : indépendant du sessionId courant
    batch.data.packets.push(clone(packetBatchMinimalFixture.data.packets[0]));
    emitFromBackend(batch);

    assertEquals(batches.length, 1);
    assertEquals(batches[0].length, 2);
    assertEquals(packets.length, 2, "chaque paquet du batch est aussi émis individuellement");
  } finally {
    clearMocks();
  }
});

Deno.test("setChannel - les événements d'une session périmée sont ignorés", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const batches: CapturedPacket[][] = [];
    const stopped: EventData<"stopped">[] = [];
    store.onPacketBatch((b: CapturedPacket[]) => batches.push(b));
    store.onStopped((d: EventData<"stopped">) => stopped.push(d));

    // La session 2 est la session courante.
    const started = clone(startedFixture);
    started.data.sessionId = 2;
    emitFromBackend(started);
    assertEquals(store.sessionId, 2);
    store.isRunning = true;

    const packetBatchForSession = (sessionId: number) => {
      const batch = clone(packetBatchMinimalFixture);
      batch.data.sessionId = sessionId;
      return batch;
    };

    // Événements retardataires de la session 1 : filtrés.
    emitFromBackend(packetBatchForSession(1));
    emitFromBackend({ event: "stopped", data: { sessionId: 1, reason: "vieux pipeline" } });
    assertEquals(batches.length, 0, "packetBatch périmé ignoré");
    assertEquals(stopped.length, 0, "stopped périmé ignoré");
    assertEquals(store.isRunning, true, "un stopped périmé n'arrête pas la session courante");

    // Ceux de la session courante passent, ainsi que les événements hors
    // session (sessionId 0, ex. import).
    emitFromBackend(packetBatchForSession(2));
    emitFromBackend(packetBatchForSession(0));
    assertEquals(batches.length, 2);

    emitFromBackend({ event: "stopped", data: { sessionId: 2, reason: "arrêt demandé" } });
    assertEquals(store.isRunning, false);
  } finally {
    clearMocks();
  }
});

Deno.test("updateStatus - reprend le session_id du statut backend", () => {
  mockIPC(() => {});
  try {
    setActivePinia(createPinia());
    const store = useCaptureStore();

    store.updateStatus({ is_running: true, session_id: 7 });
    assertEquals(store.isRunning, true);
    assertEquals(store.sessionId, 7);

    // L'arrêt ne remet pas l'id à zéro : il sert encore à filtrer les
    // retardataires de la session close.
    store.updateStatus({ is_running: false });
    assertEquals(store.isRunning, false);
    assertEquals(store.sessionId, 7);
  } finally {
    clearMocks();
  }
});

Deno.test("updateStatus - un Running tardif ne ressuscite pas une session arrêtée", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const started = clone(startedFixture);
    started.data.sessionId = 7;
    emitFromBackend(started);

    emitFromBackend({
      event: "stopped",
      data: { sessionId: 7, reason: "erreur pcap" },
    });
    assertEquals(store.isRunning, false);
    assertEquals(store.lastStoppedSessionId, 7);

    // La Promise `start_capture` peut se résoudre après l'événement terminal,
    // car réponse invoke et Channel Tauri sont deux voies indépendantes.
    store.updateStatus({ is_running: true, session_id: 7 });
    assertEquals(
      store.isRunning,
      false,
      "la réponse Running périmée est ignorée",
    );
    assertEquals(store.sessionId, 7);

    // Une génération strictement plus récente reste autorisée.
    store.updateStatus({ is_running: true, session_id: 8 });
    assertEquals(store.isRunning, true);
    assertEquals(store.sessionId, 8);

    // Une réponse encore plus ancienne ne peut pas faire régresser l'id.
    store.updateStatus({ is_running: true, session_id: 6 });
    assertEquals(store.isRunning, true);
    assertEquals(store.sessionId, 8);
  } finally {
    clearMocks();
  }
});

Deno.test("onStats - le désabonnement retire bien le listener", () => {
  mockIPC(() => {});
  try {
    const { store, emitFromBackend } = setupStoreWithChannel();
    const seen: number[] = [];
    const unsub = store.onStats((s: Stats) => seen.push(s.received));

    const stats1 = clone(statsFixture);
    stats1.data.sessionId = 0; // hors session : indépendant du sessionId courant
    stats1.data.received = 1;
    emitFromBackend(stats1);
    unsub();

    const stats2 = clone(statsFixture);
    stats2.data.sessionId = 0;
    stats2.data.received = 2;
    emitFromBackend(stats2);

    assertEquals(seen, [1]);
  } finally {
    clearMocks();
  }
});
