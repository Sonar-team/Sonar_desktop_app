<template>
  <div class="custom-table-container">
    <table class="custom-table">
      <thead>
        <tr>
          <th v-for="header in headers" :key="header.key || header.value" @click="sort(header.value)" :class="{ 'active': sortBy === header.value }">
            <button class="header-button">
              {{ header.title }}
              <span v-if="sortBy === header.value" class="sort-icon">
                <i v-if="sortDirection === 'asc'">↑</i>
                <i v-else>↓</i>
              </span>
            </button>
          </th>
        </tr>
      </thead>
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
import { info, error } from '@tauri-apps/plugin-log';

export default {
  data() {
    return {
      packets: [],
      intervalId: null,
      shouldFetch: true,
      sortBy: null,
      sortDirection: 'asc',
      headers: [
        { title: 'MAC Source', value: 'mac_address_source' },
        { title: 'MAC Destination', value: 'mac_address_destination' },
        { title: 'Internet', value: 'l_3_protocol' },
        { title: 'IP Source', value: 'layer_3_infos.ip_source' },
        { title: 'Type', value: 'layer_3_infos.ip_destination_type' },
        { title: 'IP Destination', value: 'layer_3_infos.ip_destination' },
        { title: 'Transport', value: 'layer_3_infos.l_4_protocol' },
        { title: 'Port Source', value: 'layer_3_infos.layer_4_infos.port_source' },
        { title: 'Port Destination', value: 'layer_3_infos.layer_4_infos.port_destination' },
        { title: 'Application', value: 'layer_3_infos.layer_4_infos.l_7_protocol' },
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
    // Listen for reset events from other components
    this.$bus.on('reset', () => {
      this.reset();
    });
  },
  beforeUnmount() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
    }
    // Remove the bus listener when component is destroyed
    this.$bus.off('reset');
  },
  methods: {
    async fetchPacketInfos() {
      try {
        if (!this.shouldFetch) return;
        
        const jsonString = await invoke('get_matrice', {});
        const newPackets = JSON.parse(jsonString);
        
        // Update packets only if we're still fetching
        if (this.shouldFetch) {
          this.packets = newPackets;
          this.$bus.emit('update-packet-count', this.packets.length);
          
          // Stop fetching if we have more than 25 packets
          if (this.packets.length > 25) {
            this.shouldFetch = false;
            info('Stopped fetching due to data size limit');
          }
        }
      } catch (error) {
        error("Error fetching packet infos:", error);
      }
    },
    async sort(headerValue) {
      try {
        const jsonString = await invoke('get_matrice', { headerValue });
        const newPackets = JSON.parse(jsonString);
        this.packets = newPackets;
      } catch (error) {
        error("Error sorting packets:", error);
      }
    },
    
    // Add a reset method to be called when needed
    async reset() {
      try {
        await invoke('reset');
        this.packets = [];
        this.shouldFetch = true;
        this.$bus.emit('update-packet-count', 0);
        info('Matrice reset completed');
      } catch (error) {
        error('Error resetting matrice:', error);
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
  table-layout: auto; /* <-- clé ici pour largeur auto */

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
  cursor: pointer;
}

.custom-table thead th.active {
  background-color: #3A3A3A;
}

.header-button {
  background: none;
  border: none;
  color: inherit;
  font: inherit;
  cursor: pointer;
  padding: 0;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  height: 100%;
}

.header-button:hover {
  color: rgb(132, 195, 247);
}

.sort-icon {
  font-size: 0.8em;
}

.custom-table tbody tr:nth-child(even) {
  background-color: rgba(255, 255, 255, 0.05);
}

.custom-table tbody td {
  color: rgb(132, 195, 247);
  padding: 6px 8px;
}


</style>
