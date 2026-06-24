<template>
  <div>
    <table class="trames">
      <thead>
        <tr>
          <th>MAC S</th>
          <th>MAC D</th>
          <th>Vlan</th>
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
        <tr v-for="(row, index) in rows" :key="index">
          <td>{{ row.sourceMac }}</td>
          <td>{{ row.destinationMac }}</td>
          <td>{{ row.vlan }}</td>
          <td>{{ row.ethertype }}</td>
          <td>{{ row.source }}</td>
          <td>{{ row.destination }}</td>
          <td>{{ row.transportProtocol }}</td>
          <td>{{ row.sourcePort }}</td>
          <td>{{ row.destinationPort }}</td>
          <td>{{ row.applicationProtocol }}</td>
          <td>{{ row.length }}</td>
          <td>{{ row.time }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { useCaptureStore } from '../../store/capture'
import type { PacketMinimal } from '../../types/capture'

const LOG_FLUSH_INTERVAL_MS = 100
const MAX_LOG_ROWS = 5
const MAX_BUFFERED_ROWS = MAX_LOG_ROWS * 4

type PacketLogRow = {
  sourceMac: string
  destinationMac: string
  vlan: string
  ethertype: string
  source: string
  destination: string
  transportProtocol: string
  sourcePort: string
  destinationPort: string
  applicationProtocol: string
  length: string
  time: string
}

type PacketFlowLogFields = {
  source_mac?: unknown
  destination_mac?: unknown
  vlan?: { id?: unknown } | null
  ethertype?: unknown
  source_ip?: unknown
  destination_ip?: unknown
  source?: unknown
  destination?: unknown
  protocol_internet?: unknown
  protocol_transport?: unknown
  protocol?: unknown
  source_port?: unknown
  destination_port?: unknown
  application_protocol?: unknown
}

type ResetBus = {
  on?: (event: 'reset', handler: () => void) => void
  off?: (event: 'reset', handler?: () => void) => void
}

type ComponentWithResetBus = {
  $bus?: ResetBus
}

export default defineComponent({
  data() {
    return {
      rows: [] as PacketLogRow[],
      offPacket: null as null | (() => void),
      resetHandler: null as null | (() => void),
      flushTimer: null as number | null,
    }
  },
  computed: {
    captureStore() {
      return useCaptureStore()
    },
  },
  mounted() {
    const pendingPackets: PacketMinimal[] = []

    const flushPackets = () => {
      this.flushTimer = null
      if (pendingPackets.length === 0) return

      const nextPackets = pendingPackets.slice(-MAX_LOG_ROWS)
      pendingPackets.length = 0
      this.rows = [
        ...this.rows,
        ...nextPackets.map((packet) => this.createLogRow(packet)),
      ].slice(-MAX_LOG_ROWS)
    }

    const scheduleFlush = () => {
      if (this.flushTimer !== null) return
      this.flushTimer = window.setTimeout(flushPackets, LOG_FLUSH_INTERVAL_MS)
    }

    const onPacket = (packet: PacketMinimal | undefined | null) => {
      if (!packet || typeof packet !== 'object') return

      pendingPackets.push(packet)
      if (pendingPackets.length > MAX_BUFFERED_ROWS) {
        pendingPackets.splice(0, pendingPackets.length - MAX_LOG_ROWS)
      }
      scheduleFlush()
    }

    const maybeOff = this.captureStore.onPacket(onPacket)
    if (typeof maybeOff === 'function') this.offPacket = maybeOff

    const reset = () => {
      pendingPackets.length = 0
      this.rows = []
    }
    this.resetHandler = reset
    const bus = (this as unknown as ComponentWithResetBus).$bus
    bus?.on?.('reset', reset)
  },
  beforeUnmount() {
    if (this.flushTimer !== null) {
      window.clearTimeout(this.flushTimer)
      this.flushTimer = null
    }
    if (this.offPacket) {
      try { this.offPacket() } catch {}
    }
    const bus = (this as unknown as ComponentWithResetBus).$bus
    if (this.resetHandler) {
      bus?.off?.('reset', this.resetHandler)
    } else {
      bus?.off?.('reset')
    }
  },
  methods: {
    createLogRow(packet: PacketMinimal): PacketLogRow {
      const flow = packet.flow as PacketFlowLogFields | null

      return {
        sourceMac: this.formatValue(flow?.source_mac),
        destinationMac: this.formatValue(flow?.destination_mac),
        vlan: this.formatValue(flow?.vlan?.id),
        ethertype: this.formatValue(flow?.ethertype),
        source: this.formatValue(flow?.source_ip ?? flow?.source),
        destination: this.formatValue(flow?.destination_ip ?? flow?.destination),
        transportProtocol: this.formatTransportProtocol(flow),
        sourcePort: this.formatValue(flow?.source_port),
        destinationPort: this.formatValue(flow?.destination_port),
        applicationProtocol: this.formatApplicationProtocol(flow),
        length: this.formatValue(packet.len),
        time: this.formatTimestamp(packet.ts_sec, packet.ts_usec),
      }
    },
    formatTransportProtocol(flow: PacketFlowLogFields | null): string {
      const explicitProtocol = this.formatValue(flow?.protocol_transport)
      if (explicitProtocol !== '-') return this.normalizeTransportProtocol(explicitProtocol)

      const legacyProtocol = this.formatValue(flow?.protocol)
      if (this.isTcpOrUdp(legacyProtocol)) return this.normalizeTransportProtocol(legacyProtocol)

      return '-'
    },
    formatApplicationProtocol(flow: PacketFlowLogFields | null): string {
      const explicitProtocol = this.formatValue(flow?.application_protocol)
      if (explicitProtocol !== '-') return explicitProtocol

      const legacyProtocol = this.formatValue(flow?.protocol)
      if (legacyProtocol !== '-' && !this.isTcpOrUdp(legacyProtocol)) return legacyProtocol

      return '-'
    },
    normalizeTransportProtocol(value: string): string {
      const protocol = value.trim().toLowerCase()
      if (protocol === 'tcp') return 'TCP'
      if (protocol === 'udp') return 'UDP'
      return value
    },
    isTcpOrUdp(value: string): boolean {
      const protocol = value.trim().toLowerCase()
      return protocol === 'tcp' || protocol === 'udp'
    },
    formatValue(value: unknown): string {
      if (value === null || value === undefined || value === '') return '-'
      return String(value)
    },
    formatTimestamp(sec?: number, usec?: number): string {
      if (typeof sec !== 'number' || typeof usec !== 'number') return '-'
      const date = new Date(sec * 1000 + Math.floor(usec / 1000))
      const hours = String(date.getHours()).padStart(2, '0')
      const minutes = String(date.getMinutes()).padStart(2, '0')
      const seconds = String(date.getSeconds()).padStart(2, '0')
      const ms = String(Math.floor((usec % 1_000_000) / 1000)).padStart(3, '0')
      return `${hours}:${minutes}:${seconds}.${ms}`
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
