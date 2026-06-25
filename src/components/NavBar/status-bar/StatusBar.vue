<template>
  <div class="status-bar">
    <div class="left-status-content">
      <InterfaceStatus />
      <div v-if="captureStore.activeFilter" class="filter-badge" :title="captureStore.activeFilter">
        <span class="filter-icon">⚗</span>
        <span class="filter-text">{{ captureStore.activeFilter }}</span>
        <button class="filter-clear" @click="clearFilter" title="Supprimer le filtre">✕</button>
      </div>
    </div>

    <div class="right-status-content">
      <Timer />
      <Cpu />

      <p title="Trames reçues 📥 par la carte réseau">
  📥: <span class="counter">{{ stats.received }}</span>
      </p>

      <p title="Trames analysées dans la matrice de flux 📊">
        <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />
        : <span class="counter">{{ stats.processed }}</span>
      </p>

      <p title="Trames ❌ perdues côté kernel">
        ❌: <span class="counter">{{ stats.dropped }}</span>
      </p>

      <p title="Trames 🚫 perdues au niveau de l’interface">
        🚫: <span class="counter">{{ stats.if_dropped }}</span>
      </p>

      <ChannelStatus />
    </div>
  </div>
</template>

<script>
import ChannelStatus from './ChannelStatus.vue';
import InterfaceStatus from './InterfaceStatus.vue';
import Timer from './Timer.vue';
import Cpu from './Cpu.vue';

import { useCaptureStore } from '../../../store/capture';
import { info } from '@tauri-apps/plugin-log';
import { invoke } from '@tauri-apps/api/core';

export default {
  name: 'StatusBar',
  components: { ChannelStatus, InterfaceStatus, Timer, Cpu },
  data() {
    return {
      stats: { received: 0, dropped: 0, if_dropped: 0, processed: 0 },
      _unsub: [], // pour garder les unsubscribe si nécessaires
      _resetHandler: null,
    };
  },
  computed: {
    captureStore() { return useCaptureStore(); },
  },
  mounted() {
    // Stats live de la capture
    this._unsub.push(this.captureStore.onStats((s) => {
      this.stats.received   = s.received ?? 0;
      this.stats.dropped    = s.dropped ?? 0;
      this.stats.if_dropped = s.if_dropped ?? 0;
      this.stats.processed  = s.processed ?? 0;
    }));
    this._unsub.push(this.captureStore.onFinished((f) => {
      this.stats.processed = f.matrix_total_count;
      this.stats.received = f.packet_total_count;
    }));

    // Reset global
    this._resetHandler = () => {
      this.stats = { received: 0, dropped: 0, if_dropped: 0, processed: 0 };
      this.matrice_len = 0;
    };
    this.$bus.on('reset', this._resetHandler);
  },
  methods: {
    async clearFilter() {
      try {
        await invoke('set_filter', { filter: '' });
        this.captureStore.setActiveFilter('');
      } catch (e) {
        console.error('clear filter failed:', e);
      }
    },
  },
  beforeUnmount() {
    if (this._resetHandler) this.$bus.off('reset', this._resetHandler);
    // si tes onXxx() renvoient une fonction d’unsubscribe, tu peux les stocker dans _unsub et les appeler ici
    for (const u of this._unsub) { try { u(); } catch {} }
    this._unsub = [];
  },
};
</script>

<style scoped>
.status-bar {
  height: 22px;
  position: fixed; bottom: 0; left: 0; width: 100%;
  background-color: #243452; color: #ffffff; font-size: 12px;
  display: flex; flex-direction: row; justify-content: space-between; align-items: center;
  padding: 0 10px; box-sizing: border-box;
}
.left-status-content { display: flex; align-items: center; }
.right-status-content { display: flex; align-items: center; gap: 12px; text-align: right; }
.icon-img { height: 16px; width: 16px; vertical-align: middle; margin-right: 5px; }

.counter {
  display: inline-block;
  width: 60px;
  text-align: right;
  font-family: monospace;
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
