<template>
  <div class="status-bar">
    <div class="left-status-content">  
      <InterfaceStatus />
    </div>

    <div class="right-status-content">
      <Timer />
      
      <Cpu />
      <p title="Trames reçues 📥 par la carte réseau">📥: {{ stats.received }}</p>
      <p title="Trames analysées dans la matrice de flux 📊">
        <img src="/src-tauri/icons/StoreLogo.png" alt="Flux" class="icon-img" />: {{ matrice_len }}
      </p>
      <p title="Trames ❌ perdues côté kernel">❌: {{ stats.dropped }}</p>
      <p title="Trames 🚫 perdues au niveau de l’interface">🚫: {{ stats.if_dropped }}</p>
      <ChannelStatus />
    </div>
  </div>
</template>

<script>
import { listen } from '@tauri-apps/api/event'
import ChannelStatus from './ChannelStatus.vue'
import InterfaceStatus from './InterfaceStatus.vue'
import Timer from './Timer.vue'
import Cpu from './Cpu.vue'

export default {
  name: 'StatusBar',
  components: {
    ChannelStatus,
    InterfaceStatus,
    Timer,
    Cpu
  },
  data() {
    return {
      stats: {
        received: 0,
        dropped: 0,
        if_dropped: 0,
        processed: 0,
      },
      matrice_len: 0,
    }
  },
  mounted() {
    // Listen for stats updates
    listen('stats', (event) => {
      this.stats = event.payload
      this.matrice_len = event.payload.processed;
    });
    
    // Listen for reset events
    this.$bus.on('reset', () => {
      this.stats = {
        received: 0,
        dropped: 0,
        if_dropped: 0,
        processed: 0,
      };
      this.matrice_len = 0;
    });
    
    // Listen for matrice length updates
    listen('matrice_len', (event) => {
      this.matrice_len = event.payload;
    });
  },
  beforeUnmount() {
    this.$bus.off('reset');
  }
}
</script>
<style scoped>
.status-bar {
  height: 22px;
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  background-color: #243452;
  color: #ffffff;
  font-size: 12px;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 0 10px;
  box-sizing: border-box;
}

.left-status-content {
  display: flex;
  align-items: center;
}

.right-status-content {
  display: flex;
  align-items: center;
  gap: 12px;
  text-align: right;
}

.icon-img {
  height: 16px;
  width: 16px;
  vertical-align: middle;
  margin-right: 5px;
}
</style>
