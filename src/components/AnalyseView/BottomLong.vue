<!--
  Tableau des trames reçues en direct (batchs de paquets du backend),
  affiché sous le graphe pendant une capture.
-->
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
import type { CapturedPacket, DataLink, NetworkProtocol } from '../../types/capture'

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
      offPacketBatch: null as null | (() => void),
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
    const pendingPackets: CapturedPacket[] = []

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

    const enqueuePackets = (packets: CapturedPacket[]) => {
      if (packets.length === 0) return
      pendingPackets.push(...packets.slice(-MAX_LOG_ROWS))
      if (pendingPackets.length > MAX_BUFFERED_ROWS) {
        pendingPackets.splice(0, pendingPackets.length - MAX_LOG_ROWS)
      }
      scheduleFlush()
    }

    // Uniquement `onPacketBatch` : le store redistribue aussi chaque batch
    // paquet par paquet à `onPacket` (pour les consommateurs qui le
    // préfèrent), donc s'abonner aux deux ferait apparaître chaque trame en
    // double dans le tableau.
    const onPacketBatch = (packets: CapturedPacket[] | undefined | null) => {
      if (!Array.isArray(packets)) return
      enqueuePackets(packets.filter((packet) => packet && typeof packet === 'object'))
    }

    const maybeOffBatch = this.captureStore.onPacketBatch(onPacketBatch)
    if (typeof maybeOffBatch === 'function') this.offPacketBatch = maybeOffBatch

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
    if (this.offPacketBatch) {
      try { this.offPacketBatch() } catch { /* already unsubscribed */ }
    }
    const bus = (this as unknown as ComponentWithResetBus).$bus
    if (this.resetHandler) {
      bus?.off?.('reset', this.resetHandler)
    } else {
      bus?.off?.('reset')
    }
  },
  methods: {
    createLogRow(packet: CapturedPacket): PacketLogRow {
      const flow = packet.flow
      const dataLink = flow.data_link

      return {
        sourceMac: this.formatValue(this.linkAddressFields(dataLink).sourceMac),
        destinationMac: this.formatValue(this.linkAddressFields(dataLink).destinationMac),
        vlan: this.formatValue(this.linkVlanId(dataLink)),
        ethertype: this.formatValue(this.linkProtocolLabel(dataLink)),
        source: this.formatValue(flow.source_ip),
        destination: this.formatValue(flow.destination_ip),
        transportProtocol: this.formatValue(
          flow.protocol_transport ? this.normalizeTransportProtocol(flow.protocol_transport) : undefined,
        ),
        sourcePort: this.formatValue(flow.source_port),
        destinationPort: this.formatValue(flow.destination_port),
        applicationProtocol: this.formatValue(flow.application_protocol),
        length: this.formatValue(packet.len),
        time: this.formatTimestamp(packet.tsSec, packet.tsUsec),
      }
    },
    // MAC pour Ethernet/802.11 ; adresse cooked (longueur variable) pour
    // SLL/SLL2 ; aucune adresse de liaison pour RAW IP — `link_kind`
    // discrimine le seul groupe de champs réellement présent (#142).
    linkAddressFields(dataLink: DataLink): { sourceMac?: string; destinationMac?: string } {
      switch (dataLink.link_kind) {
        case 'ethernet':
        case 'ieee80211':
          return {
            sourceMac: dataLink.link_details.source_mac,
            destinationMac: dataLink.link_details.destination_mac,
          }
        case 'linux_sll':
        case 'linux_sll2':
          return { sourceMac: this.formatHwAddress(dataLink.link_details.source_address) }
        case 'raw_ip':
          return {}
      }
    },
    // VLAN : seul Ethernet en porte un (absent, pas `null`, côté contrat).
    linkVlanId(dataLink: DataLink): number | undefined {
      return dataLink.link_kind === 'ethernet' ? dataLink.link_details.vlan?.id : undefined
    },
    // EtherType (Ethernet) / protocole SNAP (802.11) ; pour les liens sans
    // EtherType (RAW IP, SLL/SLL2), le protocole réseau générique du DLT.
    linkProtocolLabel(dataLink: DataLink): string | undefined {
      switch (dataLink.link_kind) {
        case 'ethernet': return dataLink.link_details.ethertype
        case 'ieee80211': return dataLink.link_details.snap_protocol
        case 'linux_sll':
        case 'linux_sll2':
        case 'raw_ip':
          return this.formatNetworkProtocol(dataLink.network_protocol)
      }
    },
    normalizeTransportProtocol(value: string): string {
      const protocol = value.trim().toLowerCase()
      if (protocol === 'tcp') return 'TCP'
      if (protocol === 'udp') return 'UDP'
      return value
    },
    formatValue(value: unknown): string {
      if (value === null || value === undefined || value === '') return '-'
      return String(value)
    },
    // Adresse matérielle SLL/SLL2 : sérialisée en tableau d'octets côté Rust.
    formatHwAddress(value: number[] | null | undefined): string | undefined {
      if (!value || value.length === 0) return undefined
      return value.map((byte) => byte.toString(16).padStart(2, '0')).join(':')
    },
    // Protocole réseau annoncé par la couche liaison quand elle n'a pas
    // d'EtherType (RAW IP, SLL) : sérialisé `{ kind, value }` côté Rust.
    formatNetworkProtocol(protocol: NetworkProtocol): string | undefined {
      switch (protocol.kind) {
        case 'ipv4': return 'IPv4'
        case 'ipv6': return 'IPv6'
        case 'arp': return 'ARP'
        case 'profinet': return 'Profinet'
        case 'other':
          return `0x${protocol.value.toString(16).toUpperCase().padStart(4, '0')}`
      }
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
