<!--
  Barre de statut : compteurs de capture (reçus, traités, perdus), état du
  canal, interface active, CPU et durée de capture.
-->
<template>
  <div class="status-bar">
    <div class="left-status-content">
      <InterfaceStatus />
      <p
        v-if="captureStore.linkType"
        class="dlt-badge"
        title="Type de liaison (LINKTYPE/DLT) de la capture ou du fichier importé"
      >
        DLT : {{ captureStore.linkType }}
      </p>
      <div v-if="captureStore.activeFilter" class="filter-badge" :title="captureStore.activeFilter">
        <span class="filter-icon">⚗</span>
        <span class="filter-text">{{ captureStore.activeFilter }}</span>
        <button type="button" class="filter-clear" @click="clearFilter" title="Supprimer le filtre">✕</button>
      </div>
      <p
        v-if="lastImport"
        class="import-badge"
        :title="importReportDetail"
      >
        Import : {{ lastImport.fileName }} ({{ lastImport.totalCount }} lignes{{ rejectedTotal ? `, ${rejectedTotal} rejetée${rejectedTotal > 1 ? 's' : ''}` : '' }})
      </p>
      <p v-if="lastStopReason" class="stopped-badge" :title="lastStopReason">
        Arrêt : {{ lastStopReason }}
      </p>
    </div>

    <div class="right-status-content">
      <Timer />
      <Cpu />

      <p title="Trames reçues 📥 par la carte réseau">
        <span class="counter">{{ stats.received }} :📥</span>
      </p>

      <p title="Flux distincts dans la matrice 📊">
        
        <span class="counter">{{ stats.processed }} :<img src="../../../../src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" /></span>
      </p>

      <p title="Trames ❌ perdues côté kernel">
       <span class="counter">{{ stats.dropped }} :❌</span>
      </p>

      <p title="Trames 🚫 perdues au niveau de l’interface">
        <span class="counter">{{ stats.ifDropped }} :🚫</span>
      </p>

      <p title="Trames ⚠️ perdues côté application (pool ou canal saturé)">
        <span class="counter">{{ stats.appDropped }} :⚠️</span>
      </p>

      <p title="Trames 🧩 acceptées mais illisibles par le parseur">
        <span class="counter">{{ stats.parseErrors }} :🧩</span>
      </p>

      <ChannelStatus />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import ChannelStatus from './ChannelStatus.vue';
import InterfaceStatus from './InterfaceStatus.vue';
import Timer from './Timer.vue';
import Cpu from './Cpu.vue';

import { useCaptureStore } from '../../../store/capture';
import { invoke } from '@tauri-apps/api/core';
import { error } from '@tauri-apps/plugin-log';
import type { Stats } from '../../../types/capture';

type StatsView = Record<
  'received' | 'dropped' | 'ifDropped' | 'appDropped' | 'parseErrors' | 'processed',
  number
>;

const EMPTY_STATS: StatsView = {
  received: 0, dropped: 0, ifDropped: 0, appDropped: 0, parseErrors: 0, processed: 0,
};

interface LastImport {
  fileName: string;
  integratedCount: number;
  totalCount: number;
  // Catégories fines du bilan (#150) : chaque paquet lu appartient à
  // exactement une, le total est leur somme.
  truncatedCount: number;
  unsupportedLinkTypeCount: number;
  malformedCount: number;
}

export default defineComponent({
  name: 'StatusBar',
  components: { ChannelStatus, InterfaceStatus, Timer, Cpu },
  data() {
    return {
      stats: { ...EMPTY_STATS } as StatsView,
      lastImport: null as LastImport | null,
      lastStopReason: null as string | null,
      _unsub: [] as Array<() => void>,
      _resetHandler: null as null | (() => void),
    };
  },
  computed: {
    captureStore() { return useCaptureStore(); },
    rejectedTotal(): number {
      const imported = this.lastImport;
      if (!imported) return 0;
      return imported.truncatedCount + imported.unsupportedLinkTypeCount
        + imported.malformedCount;
    },
    /** Équation du bilan, lisible sans ouvrir un log (#150). */
    importReportDetail(): string {
      const imported = this.lastImport;
      if (!imported) return '';
      return `${imported.fileName} : ${imported.totalCount} lus = `
        + `${imported.integratedCount} intégrés + ${imported.truncatedCount} tronqués `
        + `+ ${imported.unsupportedLinkTypeCount} DLT non supportés `
        + `+ ${imported.malformedCount} malformés`;
    },
  },
  mounted() {
    // Stats live de la capture
    this._unsub.push(this.captureStore.onStats((s: Stats) => {
      this.stats.received    = s.received ?? 0;
      this.stats.dropped     = s.dropped ?? 0;
      this.stats.ifDropped   = s.ifDropped ?? 0;
      this.stats.appDropped  = s.appDropped ?? 0;
      this.stats.parseErrors = s.parseErrors ?? 0;
      this.stats.processed   = s.processed ?? 0;
    }));
    this._unsub.push(this.captureStore.onFinished((f) => {
      this.stats.processed = f.matrixTotalCount;
      this.stats.received = f.packetTotalCount;
      // Rapport qualité de l'import (#150) : les paquets rejetés du fichier
      // sont visibles au même titre que les pertes de capture, et leur cause
      // (troncature de capture, DLT inconnu, trame malformée) est détaillée
      // dans l'infobulle du badge.
      this.stats.parseErrors = (f.rejectedTruncatedCount ?? 0)
        + (f.rejectedUnsupportedLinkTypeCount ?? 0)
        + (f.rejectedMalformedCount ?? 0);
      this.lastImport = {
        fileName: f.fileName,
        integratedCount: f.integratedCount,
        totalCount: f.packetTotalCount,
        truncatedCount: f.rejectedTruncatedCount ?? 0,
        unsupportedLinkTypeCount: f.rejectedUnsupportedLinkTypeCount ?? 0,
        malformedCount: f.rejectedMalformedCount ?? 0,
      };
    }));
    // Raison d'arrêt (backend, ex. erreur pcap) : sans cet affichage, seule
    // la console la portait (#142) et l'utilisateur ne savait pas pourquoi
    // une capture s'était arrêtée d'elle-même.
    this._unsub.push(this.captureStore.onStopped((d) => {
      this.lastStopReason = d.reason;
    }));
    this._unsub.push(this.captureStore.onStarted(() => {
      this.lastStopReason = null;
    }));

    // Reset global
    this._resetHandler = () => {
      this.stats = { ...EMPTY_STATS };
      this.lastImport = null;
      this.lastStopReason = null;
    };
    this.$bus.on('reset', this._resetHandler);
  },
  methods: {
    async clearFilter() {
      try {
        await invoke('set_filter', { filter: '' });
        this.captureStore.setActiveFilter('');
      } catch (e) {
        error(`clear filter failed: ${e}`);
      }
    },
  },
  beforeUnmount() {
    if (this._resetHandler) this.$bus.off('reset', this._resetHandler);
    // si tes onXxx() renvoient une fonction d’unsubscribe, tu peux les stocker dans _unsub et les appeler ici
    for (const u of this._unsub) { try { u(); } catch { /* already unsubscribed */ } }
    this._unsub = [];
  },
});
</script>

<style scoped>
.status-bar {
  height: 22px;
  position: fixed; bottom: 0; left: 0; width: 100%;
  background-color: #243452; color: #ffffff; font-size: 12px;
  display: flex; flex-direction: row; justify-content: space-between; align-items: center;
  padding: 0 10px; box-sizing: border-box;
}
.left-status-content { display: flex; align-items: center; min-width: 0; overflow: hidden; }
.right-status-content { display: flex; align-items: center; gap: 4px 10px; text-align: right; min-width: 0; flex-shrink: 0; }
.status-bar p { margin: 0; line-height: 1; }
.right-status-content > p { flex: 0 0 auto; white-space: nowrap; }
.icon-img { height: 16px; width: 16px; vertical-align: middle; margin-right: 5px; }

.counter {
  display: inline-block;
  width: auto;
  min-width: 5ch;
  flex: 0 0 5ch;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
  text-align: right;
  font-family: monospace;
}

.dlt-badge {
  margin-left: 10px;
  font-family: monospace;
  font-size: 11px;
  color: #9fd3a8;
  white-space: nowrap;
  user-select: none;
}

.import-badge {
  margin-left: 10px;
  font-family: monospace;
  font-size: 11px;
  color: #9fd3a8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 260px;
  user-select: none;
}

.stopped-badge {
  margin-left: 10px;
  font-family: monospace;
  font-size: 11px;
  color: #e0a458;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 260px;
  user-select: none;
}

.filter-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 10px;
  padding: 0 7px;
  height: 16px;
  background: rgba(74, 124, 255, 0.15);
  border: 1px solid rgba(74, 124, 255, 0.4);
  border-radius: 4px;
  max-width: 320px;
  overflow: hidden;
}
.filter-icon {
  font-size: 11px;
  flex-shrink: 0;
  opacity: 0.8;
}
.filter-text {
  font-family: monospace;
  font-size: 11px;
  color: #8ab4ff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.filter-clear {
  flex-shrink: 0;
  background: transparent;
  border: none;
  color: #6080c0;
  font-size: 10px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
  opacity: 0.7;
  transition: opacity 0.15s, color 0.15s;
}
.filter-clear:hover {
  opacity: 1;
  color: #ff6060;
}


</style>
