<template>
  <div class="status-bar">
    <div class="left-status-content">
      <InterfaceStatus />
    </div>

    <div class="right-status-content">
      <Timer />
      <Cpu />

      <!-- RX: arrow down into tray -->
      <p title="Trames reçues par la carte réseau">
        <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
          <path d="M6 1v7M3.5 5.5L6 8l2.5-2.5"/>
          <path d="M2 10h8"/>
        </svg>
        <span class="counter">{{ stats.received }}</span>
      </p>

      <!-- PKT: grid/matrix -->
      <p title="Trames analysées dans la matrice de flux">
        <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
          <rect x="1.5" y="1.5" width="3" height="3" rx="0.5"/>
          <rect x="7.5" y="1.5" width="3" height="3" rx="0.5"/>
          <rect x="1.5" y="7.5" width="3" height="3" rx="0.5"/>
          <rect x="7.5" y="7.5" width="3" height="3" rx="0.5"/>
        </svg>
        <span class="counter">{{ stats.processed }}</span>
      </p>

      <!-- DRP: arrow dissolving -->
      <p title="Trames perdues côté kernel">
        <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
          <path d="M6 2v5M3.5 4.5L6 7l2.5-2.5"/>
          <path d="M2.5 10h2M7.5 10h2M5 10h2" stroke-dasharray="1.5 1.5"/>
        </svg>
        <span class="counter">{{ stats.dropped }}</span>
      </p>

      <!-- IFD: network plug disconnected -->
      <p title="Trames perdues au niveau de l’interface">
        <svg class="stat-icon" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
          <path d="M2 6h3.5M6.5 6H10"/>
          <circle cx="6" cy="6" r="1.2"/>
          <line x1="9.5" y1="2.5" x2="2.5" y2="9.5" stroke-width="1.1" opacity="0.5"/>
        </svg>
        <span class="counter">{{ stats.if_dropped }}</span>
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

export default {
  name: 'StatusBar',
  components: { ChannelStatus, InterfaceStatus, Timer, Cpu },
  data() {
    return {
      stats: { received: 0, dropped: 0, if_dropped: 0, processed: 0 },
      _unsub: [], // pour garder les unsubscribe si nécessaires
    };
  },
  computed: {
    captureStore() { return useCaptureStore(); },
  },
  mounted() {
    // Stats live de la capture
    this.captureStore.onStats((s) => {
      this.stats.received   = s.received ?? 0;
      this.stats.dropped    = s.dropped ?? 0;
      this.stats.if_dropped = s.if_dropped ?? 0;
      this.stats.processed  = s.processed ?? 0;
    });
    this.captureStore.onFinished((f) => {
      this.stats.processed = f.matrix_total_count;
      this.stats.received = f.packet_total_count;
    });

    // Reset global
    this.$bus.on('reset', () => {
      this.stats = { received: 0, dropped: 0, if_dropped: 0, processed: 0 };
      this.matrice_len = 0;
    });
  },
  beforeUnmount() {
    this.$bus.off('reset');
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
  background-color: #2c2c3a; color: #9a9aa8; font-size: 12px;
  border-top: 1px solid #383850;
  display: flex; flex-direction: row; justify-content: space-between; align-items: center;
  padding: 0 10px; box-sizing: border-box;
}
.left-status-content { display: flex; align-items: center; }
.right-status-content { display: flex; align-items: center; gap: 14px; }

p { display: flex; align-items: center; gap: 4px; margin: 0; }

.stat-icon {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
  color: #808095;
}

.counter {
  display: inline-block;
  width: 52px;
  text-align: right;
  font-family: monospace;
  color: #9a9aa8;
}


</style>
