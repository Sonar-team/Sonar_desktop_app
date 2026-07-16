// Store Pinia central de la capture : reçoit les événements du backend via
// le Channel Tauri (stats, paquets, updates graphe, snapshots), tient l'état
// courant (running, filtre, compteurs) et redistribue les événements aux
// composants par des abonnements (onGraphUpdate, onGraphSnapshot, …).
import { defineStore } from "pinia";
import { markRaw } from "vue";
import type { Channel } from "@tauri-apps/api/core";
import { invoke } from "@tauri-apps/api/core";
import type {
  CaptureEvent,
  GraphData,
  GraphUpdate,
  CapturedPacket,
  ImportProgress,
  Stats,
} from "../types/capture";
import { DEFAULT_CAPTURE_CONFIG, type CaptureConfig } from "../types/config";

// ⚠️ Channel hors réactivité pour éviter le proxy de Vue
let __channel: Channel<CaptureEvent> | undefined;

function unsubscribe<T>(listeners: Array<(d: T) => void>, cb: (d: T) => void) {
  return () => {
    const index = listeners.indexOf(cb);
    if (index !== -1) listeners.splice(index, 1);
  };
}

// Payload d'une variante nommée du contrat généré (#142) : évite de
// redéclarer à la main la forme de chaque événement.
type EventData<E extends CaptureEvent["event"]> = Extract<CaptureEvent, { event: E }>["data"];

// Version du contrat IPC attendue par ce build frontend, cf.
// `crate::events::CAPTURE_EVENT_PROTOCOL_VERSION` côté Rust. Comparée à la
// valeur reçue dans `started` (#142) : un écart signale un frontend et un
// backend compilés séparément (build de dev désynchronisé), jamais censé
// arriver sur un bundle Tauri livré atomiquement — d'où un avertissement,
// pas un blocage. Exportée pour que les channels locaux hors store (import
// de labels, `LabelsPanel.vue`) fassent la même vérification.
export const EXPECTED_PROTOCOL_VERSION = 1;

// Exhaustivité imposée par le compilateur (#142) : toute variante ajoutée
// côté Rust sans branche dans le switch ci-dessous casse le build
// TypeScript. À l'exécution, un tag que ce build ne connaît pas (donc
// jamais `never` en pratique) est journalisé plutôt que de faire planter le
// store : c'est justement le signe d'un contrat désynchronisé que
// `protocol_version` permet de diagnostiquer, pas une raison de plus pour
// perdre la session en cours.
function logUnknownEvent(event: never): void {
  console.warn("[CaptureStore] événement de capture inconnu ignoré (contrat IPC désynchronisé ?) :", event);
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
    importProgress: null as ImportProgress | null,
    hasData: false,
    activeFilter: '' as string,
    pendingFilter: '' as string,
    // Type de liaison (LINKTYPE/DLT) de la capture live ou des fichiers
    // importés, reçu dans l'événement `started` et affiché dans la barre
    // de statut ('' = inconnu).
    linkType: '' as string,

    // Listeners HMR-safe dans le state
    startedListeners: [] as Array<(d: EventData<"started">) => void>,
    finishedListeners: [] as Array<(d: EventData<"finished">) => void>,
    stoppedListeners: [] as Array<(d: EventData<"stopped">) => void>,
    packetListeners: [] as Array<(p: CapturedPacket) => void>,
    packetBatchListeners: [] as Array<(packets: CapturedPacket[]) => void>,
    statsListeners: [] as Array<(d: Stats) => void>,
    channelCapacityPayloadListeners: [] as Array<(d: EventData<"channelCapacityPayload">) => void>,
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
    clearImportProgress() {
      this.importProgress = null;
    },
    setPendingFilter(filter: string) {
      this.pendingFilter = filter;
    },

    setChannel(channel: Channel<CaptureEvent>) {
      console.log("[CaptureStore] Channel attaché");

      // détacher l’ancien handler si besoin
      if (__channel) __channel.onmessage = () => {};

      __channel = markRaw(channel);

      __channel.onmessage = (msg: CaptureEvent) => {
        // Filtrage de session : les événements de capture live portent le
        // sessionId de leur pipeline (0 = hors session, ex. import). Un id
        // différent de la session courante signale un événement périmé.
        const sid = "sessionId" in msg.data ? msg.data.sessionId : undefined;
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
            if (msg.data.protocolVersion !== EXPECTED_PROTOCOL_VERSION) {
              console.warn(
                `[CaptureStore] version du contrat IPC inattendue : reçu ${msg.data.protocolVersion}, attendu ${EXPECTED_PROTOCOL_VERSION} — frontend et backend probablement désynchronisés`
              );
            }
            if (this.pendingFilter) {
              this.activeFilter = this.pendingFilter;
              this.pendingFilter = '';
            }
            this.linkType = msg.data.linkType;
            for (const cb of this.startedListeners) cb(msg.data);
            break;
          case "finished":
            for (const cb of this.finishedListeners) cb(msg.data);
            break;
          case "stopped":
            console.log("[CaptureStore] Capture arrêtée :", msg.data.reason);
            // Arrêt initié par le backend (ex. erreur pcap) : sans cette mise
            // à jour, l'UI croirait capturer indéfiniment.
            this.isRunning = false;
            for (const cb of this.stoppedListeners) cb(msg.data);
            break;
          case "packetBatch": {
            const packets = msg.data.packets;
            // Il n'existe pas d'événement `packet` unitaire sur le fil (#142)
            // : c'est ici que la présence de données doit être détectée, et
            // que `packetListeners` (abonnés par paquet, cf. `onPacket`) est
            // nourri à partir des batches — un même consommateur ne doit
            // s'abonner qu'à l'un des deux (`onPacket` XOR `onPacketBatch`),
            // sous peine de recevoir chaque paquet deux fois.
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
          case "importProgress":
            this.importProgress = msg.data;
            break;
          case "channelCapacityPayload":
            for (const cb of this.channelCapacityPayloadListeners) cb(msg.data);
            break;
          case "graph":
            for (const cb of this.graphUpdateListeners) cb(msg.data.update);
            break;
          case "graphBatch":
            for (const update of msg.data.updates) {
              for (const cb of this.graphUpdateListeners) cb(update);
            }
            break;
          case "graphSnapshot":
            console.log("[CaptureStore] GraphSnapshot reçu");
            for (const cb of this.graphSnapshotListeners) cb(msg.data.graphData);
            break;
          default:
            logUnknownEvent(msg);
        }
      };
    },

    getChannel(): Channel<CaptureEvent> | undefined {
      return __channel;
    },
    onStarted(cb: (d: EventData<"started">) => void) {
      this.startedListeners.push(cb);
      return unsubscribe(this.startedListeners, cb);
    },
    onStopped(cb: (d: EventData<"stopped">) => void) {
      this.stoppedListeners.push(cb);
      return unsubscribe(this.stoppedListeners, cb);
    },
    onFinished(cb: (d: EventData<"finished">) => void) {
      this.finishedListeners.push(cb);
      return unsubscribe(this.finishedListeners, cb);
    },
    // Paquet par paquet, dérivé des `packetBatch` reçus (aucun événement
    // `packet` unitaire n'existe sur le fil). Ne pas combiner avec
    // `onPacketBatch` sur le même composant : chaque paquet serait délivré
    // deux fois (une fois par ce canal, une fois dans le batch).
    onPacket(cb: (p: CapturedPacket) => void) {
      this.packetListeners.push(cb);
      return unsubscribe(this.packetListeners, cb);
    },
    onPacketBatch(cb: (packets: CapturedPacket[]) => void) {
      this.packetBatchListeners.push(cb);
      return unsubscribe(this.packetBatchListeners, cb);
    },
    onStats(cb: (d: Stats) => void) {
      this.statsListeners.push(cb);
      return unsubscribe(this.statsListeners, cb);
    },
    onChannelCapacityPayload(cb: (d: EventData<"channelCapacityPayload">) => void) {
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
