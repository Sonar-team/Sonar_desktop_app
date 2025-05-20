<template>
  <div class="custom-table-container">
    <table class="custom-table">
      <tbody>
        <tr v-if="processedPackets.length === 0" v-for="n in 40" :key="'empty-' + n">
          <td v-for="header in headers" :key="header.key || header.value">&nbsp;</td>
        </tr>
        <tr v-else v-for="packet in processedPackets" :key="packet.id">
          <td>{{ packet.mac_address_source }}</td>
          <td>{{ packet.mac_address_destination }}</td>
          <td>{{ packet.l_3_protocol }}</td>
          <td>{{ packet.layer_3_infos?.ip_source }}</td>
          <td>{{ packet.layer_3_infos?.ip_destination_type }}</td>
          <td>{{ packet.layer_3_infos?.ip_destination }}</td>
          <td>{{ packet.layer_3_infos?.l_4_protocol }}</td>
          <td>{{ packet.layer_3_infos?.layer_4_infos?.port_source }}</td>
          <td>{{ packet.layer_3_infos?.layer_4_infos?.port_destination }}</td>
          <td>{{ packet.layer_3_infos?.layer_4_infos?.l_7_protocol }}</td>
          <td>{{ packet.packet_size_total }}</td>
          <td>{{ packet.count }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/core';

export default {
  data() {
    return {
      packets: [],
      intervalId: null,
      headers: [
        { title: 'MAC Source', value: 'mac_address_source' },
        { title: 'MAC Destination', value: 'mac_address_destination' },
        { title: 'L3', value: 'l_3_protocol' },
        { title: 'IP Source', value: 'layer_3_infos.ip_source' },
        { title: 'Type', value: 'layer_3_infos.ip_destination_type' },
        { title: 'IP Destination', value: 'layer_3_infos.ip_destination' },
        { title: 'L4', value: 'layer_3_infos.l_4_protocol' },
        { title: 'Port Source', value: 'layer_3_infos.layer_4_infos.port_source' },
        { title: 'Port Destination', value: 'layer_3_infos.layer_4_infos.port_destination' },
        { title: 'L7', value: 'layer_3_infos.layer_4_infos.l_7_protocol' },
        { title: 'Trame (o)', value: 'packet_size_total' },
        { title: 'Occ', value: 'count' },
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
  beforeUnmount() {
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
.custom-table-container {
  height: 100%;
  overflow-y: auto;
  background-color: #2A2A2A;

}

.custom-table {
  width: 100%;
  border-collapse: collapse;
  font-family: 'Courier New', Courier, monospace;
  font-size: 14px;
}

.custom-table thead {
  background-color: #1B1B1B;
}

.custom-table thead th {
  position: sticky;
  top: 0;
  z-index: 10;
  background-color: #1B1B1B;
  color: #ECF0F1;
  padding: 8px;
  font-weight: bold;
  text-align: left;
}


.custom-table tbody tr:nth-child(even) {
  background-color: rgba(255, 255, 255, 0.05);
}

.custom-table tbody td {
  color: rgb(132, 195, 247);
  padding: 6px 8px;
}

</style>
