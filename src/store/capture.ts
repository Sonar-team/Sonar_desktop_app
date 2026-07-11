// Store Pinia central de la capture : reçoit les événements du backend via
// le Channel Tauri (stats, paquets, updates graphe, snapshots), tient l'état
// courant (running, filtre, compteurs) et redistribue les événements aux
// composants par des abonnements (onGraphUpdate, onGraphSnapshot, …).
import { defineStore } from "pinia";
import { markRaw } from "vue";
import type { Channel } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";
import type { CaptureEvent, GraphData, GraphUpdate } from "../types/capture";
import { DEFAULT_CAPTURE_CONFIG, type CaptureConfig } from "../types/config";

// ⚠️ Channel hors réactivité pour éviter le proxy de Vue
let __channel: Channel<CaptureEvent> | undefined;

function unsubscribe<T>(listeners: Array<(d: T) => void>, cb: (d: T) => void) {
  return () => {
    const index = listeners.indexOf(cb);
    if (index !== -1) listeners.splice(index, 1);
  };
}

export const useCaptureStore = defineStore("capture", {
  state: () => ({
    isRunning: false,
    // Démarrage en cours (invoke start_capture pas encore résolu) : bloque
    // un second start avant que isRunning ne soit à jour (double-clic).
    isStarting: false,
    // Session de capture live courante (0 = aucune). Les événements portant
    // un session_id différent viennent d'une session périmée et sont ignorés.
    sessionId: 0,
    showMatrice: true,
    isImporting: false,
    hasData: false,
    activeFilter: '' as string,
    pendingFilter: '' as string,

    // Listeners HMR-safe dans le state
    startedListeners: [] as Array<(d: any) => void>,
    finishedListeners: [] as Array<(d: any) => void>,
    stoppedListeners: [] as Array<(d: any) => void>,
    packetListeners: [] as Array<(p: any) => void>,
    packetBatchListeners: [] as Array<(packets: any[]) => void>,
    statsListeners: [] as Array<(d: any) => void>,
    lenFlowMatrixListeners: [] as Array<(d: any) => void>,
    channelCapacityPayloadListeners: [] as Array<(d: any) => void>,
    graphUpdateListeners: [] as Array<(u: GraphUpdate) => void>,
    graphSnapshotListeners: [] as Array<(d: GraphData) => void>,
  }),

  actions: {
    updateStatus(status: { is_running: boolean; session_id?: number }) {
      this.isRunning = status.is_running;
      if (typeof status.session_id === "number" && status.session_id > 0) {
        this.sessionId = status.session_id;
      }
    },
    toggleView() {
      this.showMatrice = !this.showMatrice;
    },
    setActiveFilter(filter: string) {
      this.activeFilter = filter;
    },
    setPendingFilter(filter: string) {
      this.pendingFilter = filter;
    },

    setChannel(channel: Channel<CaptureEvent>) {
      console.log("[CaptureStore] Channel attaché");

      // détacher l’ancien handler si besoin
      if (__channel) (__channel as any).onmessage = undefined;

      __channel = markRaw(channel);

      __channel.onmessage = (msg: any) => {
        // Filtrage de session : les événements de capture live portent le
        // session_id de leur pipeline (0 = hors session, ex. import). Un id
        // différent de la session courante signale un événement périmé.
        const sid = msg.data?.session_id;
        if (typeof sid === "number" && sid > 0) {
          if (msg.event === "started") {
            if (sid < this.sessionId) {
              console.warn(`[CaptureStore] started d'une session périmée ignoré (${sid} < ${this.sessionId})`);
              return;
            }
            this.sessionId = sid;
          } else if (sid !== this.sessionId) {
            console.warn(`[CaptureStore] ${msg.event} d'une session périmée ignoré (${sid} ≠ ${this.sessionId})`);
            return;
          }
        }
        switch (msg.event) {
          case "started":
            if (this.pendingFilter) {
              this.activeFilter = this.pendingFilter;
              this.pendingFilter = '';
            }
            for (const cb of this.startedListeners) cb(msg.data);
            break;
          case "finished":
            for (const cb of this.finishedListeners) cb(msg.data);
            break;
          case "stopped":
            console.log("[CaptureStore] Capture arrêtée :", msg.data?.reason);
            // Arrêt initié par le backend (ex. erreur pcap) : sans cette mise
            // à jour, l'UI croirait capturer indéfiniment.
            this.isRunning = false;
            for (const cb of this.stoppedListeners) cb(msg.data);
            break;
          case "packet":
            if (!this.hasData) this.hasData = true;
            for (const cb of this.packetListeners) cb(msg.data.packet);
            break;
          case "packetBatch": {
            const packets = Array.isArray(msg.data?.packets) ? msg.data.packets : [];
            // Le backend n'émet plus de `packet` unitaire : c'est ici que la
            // présence de données doit être détectée.
            if (packets.length > 0 && !this.hasData) this.hasData = true;

            for (const cb of this.packetBatchListeners) cb(packets);

            if (this.packetListeners.length > 0) {
              for (const packet of packets) {
                for (const cb of this.packetListeners) cb(packet);
              }
            }
            break;
          }
          case "stats":
            for (const cb of this.statsListeners) cb(msg.data);
            break;
          case "channelCapacityPayload":
            for (const cb of this.channelCapacityPayloadListeners) cb(msg.data);
            break;
          case "graph": {
            const update = msg.data.update as GraphUpdate;
            for (const cb of this.graphUpdateListeners) cb(update);
            break;
          }
          case "graphBatch": {
            const updates = Array.isArray(msg.data?.updates)
              ? (msg.data.updates as GraphUpdate[])
              : [];
            for (const update of updates) {
              for (const cb of this.graphUpdateListeners) cb(update);
            }
            break;
          }
          case "graphSnapshot": {
            const graphData = msg.data.graph_data as GraphData;
            console.log("[CaptureStore] GraphSnapshot reçu");
            for (const cb of this.graphSnapshotListeners) cb(graphData);
            break;
          }
        }
      };
    },

    getChannel(): Channel<CaptureEvent> | undefined {
      return __channel;
    },
    onStarted(cb: (d: any) => void) {
      this.startedListeners.push(cb);
      return unsubscribe(this.startedListeners, cb);
    },
    onStopped(cb: (d: any) => void) {
      this.stoppedListeners.push(cb);
      return unsubscribe(this.stoppedListeners, cb);
    },
    onFinished(cb: (d: any) => void) {
      this.finishedListeners.push(cb);
      return unsubscribe(this.finishedListeners, cb);
    },
    onPacket(cb: (p: any) => void) {
      this.packetListeners.push(cb);
      return unsubscribe(this.packetListeners, cb);
    },
    onPacketBatch(cb: (packets: any[]) => void) {
      this.packetBatchListeners.push(cb);
      return unsubscribe(this.packetBatchListeners, cb);
    },
    onStats(cb: (d: any) => void) {
      this.statsListeners.push(cb);
      return unsubscribe(this.statsListeners, cb);
    },
    onFlowMatrixLen(cb: (d: any) => void) {
      this.lenFlowMatrixListeners.push(cb);
      return unsubscribe(this.lenFlowMatrixListeners, cb);
    },
    onChannelCapacityPayload(cb: (d: any) => void) {
      this.channelCapacityPayloadListeners.push(cb);
      return unsubscribe(this.channelCapacityPayloadListeners, cb);
    },
    // Applique des updates graphe retournées par une commande (mutations de
    // labels #157) : mêmes abonnés que le flux du channel de capture, mais
    // sans dépendre de l'existence d'un channel (hors capture).
    applyGraphUpdates(updates: GraphUpdate[]) {
      for (const update of updates) {
        for (const cb of this.graphUpdateListeners) cb(update);
      }
    },
    onGraphUpdate(cb: (u: GraphUpdate) => void) {
      console.log("[CaptureStore] GraphUpdate abonné");
      this.graphUpdateListeners.push(cb);
      return unsubscribe(this.graphUpdateListeners, cb);
    },
    onGraphSnapshot(cb: (d: GraphData) => void) {
      console.log("[CaptureStore] GraphSnapshot abonné");
      this.graphSnapshotListeners.push(cb);
      return unsubscribe(this.graphSnapshotListeners, cb);
    },

    async refreshHasData() {
      const empty = await invoke<boolean>('is_matrix_empty');
      this.hasData = !empty;
    },
  },
});

export const useCaptureConfigStore = defineStore("captureConfig", {
  state: () => ({
    interface: "",
    buffer_size: DEFAULT_CAPTURE_CONFIG.buffer_size,
    chan_capacity: DEFAULT_CAPTURE_CONFIG.chan_capacity,
    timeout: DEFAULT_CAPTURE_CONFIG.timeout,
    snaplen: DEFAULT_CAPTURE_CONFIG.snaplen,
  }),
  actions: {
    updateConfig(config: CaptureConfig) {
      this.interface = config.device_name;
      this.buffer_size = config.buffer_size;
      this.chan_capacity = config.chan_capacity;
      this.timeout = config.timeout;
      this.snaplen = config.snaplen;
    },
    updateInterface(iface: string) {
      this.interface = iface;
    },
  },
});
