<template>
  <div>
    <table class="trames">
      <thead>
        <tr>
          <th>MAC S</th>
          <th>MAC D</th>
          <th>Protocol</th>
          <th>IP S</th>
          <th>IP D</th>
          <th>Transport</th>
          <th>Port S</th>
          <th>Port D</th>
          <th>Application</th>
          <th>Taille</th>
          <th>Heure</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(packet, index) in safePackets" :key="index">
          <td>{{ packet?.flow?.source_mac ?? '-' }}</td>
          <td>{{ packet?.flow?.destination_mac ?? '-' }}</td>
          <td>{{ packet?.flow?.ethertype ?? '-' }}</td>
          <td>{{ packet?.flow?.source ?? '-' }}</td>
          <td>{{ packet?.flow?.destination ?? '-' }}</td>
          <td>{{ packet?.flow?.protocol ?? '-' }}</td>
          <td>{{ packet?.flow?.source_port ?? '-' }}</td>
          <td>{{ packet?.flow?.destination_port ?? '-' }}</td>
          <td>{{ packet?.flow?.application_protocol ?? '-' }}</td>
          <td>{{ packet?.len ?? '-' }}</td>
          <td>{{ formatTimestamp(packet?.ts_sec, packet?.ts_usec) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useCaptureStore } from '../../store/capture'
import type { PacketMinimal } from '../../types/capture'

export default defineComponent({
  data() {
    return {
      packets: [] as PacketMinimal[],
      offPacket: null as null | (() => void),
      resetHandler: null as null | (() => void),
      MAX_ROWS: 5, // ajuste si besoin
    }
  },
  computed: {
    captureStore() {
      return useCaptureStore()
    },
    // Ne conserve que des objets valides et limite l'affichage
    safePackets(): PacketMinimal[] {
      const arr = this.packets.filter((p) => !!p && typeof p === 'object')
      return arr.slice(Math.max(0, arr.length - this.MAX_ROWS))
    },
  },
  mounted() {
    // Abonnement aux paquets
    const onPacket = (packet: PacketMinimal | undefined | null) => {
      if (!packet || typeof packet !== 'object') return
      this.packets.push(packet)
      // garde une taille raisonnable même en interne
      if (this.packets.length > 200) this.packets.shift()
    }
    // Si ton store renvoie une fonction d’unsubscribe, garde-la
    const maybeOff = this.captureStore.onPacket(onPacket)
    if (typeof maybeOff === 'function') this.offPacket = maybeOff

    // Handler reset conservé pour off() symétrique si nécessaire
    const reset = () => { this.packets = [] }
    this.resetHandler = reset
    this.$bus?.on?.('reset', reset)
  },
  beforeUnmount() {
    if (this.offPacket) {
      try { this.offPacket() } catch {}
    }
    if (this.resetHandler) {
      this.$bus?.off?.('reset', this.resetHandler)
    } else {
      // fallback si ton bus accepte off(event) sans callback
      this.$bus?.off?.('reset')
    }
  },
  methods: {
    formatTimestamp(sec?: number, usec?: number): string {
      if (typeof sec !== 'number' || typeof usec !== 'number') return '-'
      const date = new Date(sec * 1000 + Math.floor(usec / 1000))
      try {
        return date.toLocaleTimeString('fr-FR', {
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
          fractionalSecondDigits: 3,
        })
      } catch {
        // older runtimes sans fractionalSecondDigits
        const ms = String(Math.floor((usec % 1_000_000) / 1000)).padStart(3, '0')
        const base = date.toLocaleTimeString('fr-FR', { hour12: false })
        return `${base}.${ms}`
      }
    },
  },
})
</script>

<style scoped>
.trames {
  display: block;
  height: 190px;
  flex-shrink: 0;
  background-color: #000;
  font-family: 'Courier New', Courier, monospace;
}
table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}
td, th {
  padding: 8px;
  text-align: center;
  color: rgb(132, 195, 247);
  background-color: #000;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
tbody {
  display: block;
  overflow-y: auto;
}
thead, tbody tr {
  display: table;
  width: 100%;
  table-layout: fixed;
}
</style>
