<template>
  <v-theme-provider theme="dark">
    <v-data-table
      :headers="headers"
      :items="processedPackets"
      item-key="id"
      items-per-page="19"
      density="compact"
    >
    </v-data-table>
  </v-theme-provider>
</template>

<script>
import { invoke } from '@tauri-apps/api/core';

export default {
  data() {
    return {
      packets: [],
      intervalId: null,
      headers: [
        { title: 'MAC Source', value: 'mac_address_source', key: 'mac_address_source', sortable: true },
        { title: 'MAC Destination', value: 'mac_address_destination', sortable: true },
        //{ title: 'Interface', value: 'interface', sortable: true },
        { title: 'L3', value: 'l_3_protocol', sortable: true },
        { title: 'IP Source', value: 'layer_3_infos.ip_source', sortable: true },
        // { title: 'Type Source', value: 'ip_source_type', sortable: true },
        { title: 'Type', value: 'layer_3_infos.ip_destination_type', sortable: true, removable: true },
        { title: 'IP Destination', value: 'layer_3_infos.ip_destination', sortable: true },
        { title: 'L4', value: 'layer_3_infos.l_4_protocol', sortable: true },
        { title: 'Port Source', value: 'layer_3_infos.layer_4_infos.port_source', sortable: true },
        { title: 'Port Destination', value: 'layer_3_infos.layer_4_infos.port_destination', sortable: true },
        { title: 'L7', value: 'layer_3_infos.layer_4_infos.l_7_protocol', sortable: true },
        { title: 'Trame (o)', value: 'packet_size_total', sortable: true },
        { title: 'Occ', value: 'count', sortable: true },
      ],
    };
  },
  computed: {
    processedPackets() {
      return this.processData(this.packets);
    }
  },
  mounted() {
    this.intervalId = setInterval(this.fetchPacketInfos, 1000);
  },
  beforeDestroy() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
  },
  methods: {
    async fetchPacketInfos() {
      try {
        const jsonString = await invoke('get_matrice', {});
        this.packets = JSON.parse(jsonString);
        this.$bus.emit('update-packet-count', this.packets.length);
      } catch (error) {
        console.error("Error fetching packet infos:", error);
      }
    },
    processData(data) {
      return data.map((packet, index) => ({
        id: index,
        ...packet.infos,
        count: packet.stats.count,
        packet_size_total: packet.stats.packet_size_total,
      }));
    }
  },
};
</script>

<style scoped>
.v-data-table {
  background-color: transparent !important;
}

.v-data-table :deep(.v-data-table__wrapper) {
  background-color: #2A2A2A;

  overflow-y: auto;
}

.v-data-table :deep(.v-data-table-header) {
  background-color: #1B1B1B;
}

.v-data-table :deep(.v-data-table-header tr th) {
  color: #ECF0F1;
  font-weight: bold;
}

.v-data-table :deep(.v-data-table-header tr th.sortable:hover) {
  background-color: #11212C !important;
}

.v-data-table :deep(tbody tr:nth-of-type(even)) {
  background-color: rgba(255, 255, 255, 0.05);
}

.v-data-table :deep(tbody tr:not(:last-child)) {
  border-bottom: none !important;
}

.v-data-table :deep(td) {
  color: rgb(132, 195, 247);
  font-family: 'Courier New', Courier, monospace;
  font-size: 14px;
}

/* Pour le défilement horizontal */
.v-data-table :deep(.v-data-table__wrapper)::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.v-data-table :deep(.v-data-table__wrapper)::-webkit-scrollbar-thumb {
  background-color: #11212C;
  border-radius: 4px;
}

.v-data-table :deep(.v-data-table__wrapper)::-webkit-scrollbar-track {
  background-color: #1B1B1B;
}
</style>

