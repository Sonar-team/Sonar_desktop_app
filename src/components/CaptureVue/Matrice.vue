<template>
  <div class="matrice">
    <table >
      <thead>
        <tr>
          <th>MAC Source</th>
          <th>MAC Destination</th>
          <th>Interface</th>
          <th>Protocole</th>
          <th>IP Source</th>
          <th>IP Destination</th>
          <th>Protocole</th>
          <th>Port Source</th>
          <th>Port Destination</th>
          <th>Protocole</th>
          <th>Taille Total</th>
          <th>Occurences</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="packet in processedPackets" :key="packet.id">
          <td>{{ packet.info.mac_address_source }}</td>
          <td>{{ packet.info.mac_address_destination }}</td>
          <td>{{ packet.info.interface }}</td>
          <td>{{ packet.info.l_3_protocol }}</td>
          <td>{{ packet.info.layer_3_infos.ip_source }}</td>
          <td>{{ packet.info.layer_3_infos.ip_destination }}</td>
          <td>{{ packet.info.layer_3_infos.l_4_protocol }}</td>
          <td>{{ packet.info.layer_3_infos.layer_4_infos.port_source }}</td>
          <td>{{ packet.info.layer_3_infos.layer_4_infos.port_destination }}</td>
          <td>{{ packet.info.layer_3_infos.layer_4_infos.l_7_protocol }}</td>
          <td>{{ packet.info.packet_size}}</td>
          <td>{{ packet.count }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri';

export default {
  data() {
    return {
      packets: [],
      intervalId: null,
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
      // Assuming each item in 'data' is an array of [PacketInfos, count]
      return data.map(([packetInfo, count], index) => ({
        id: index,
        info: packetInfo,
        count: count
      }));
    }
  },
};
</script>

<style scoped>
.matrice {
  height: 620px;
  border: 2px solid #3a3a3a; /* Darker border */
  border-radius: 10px;
  width: 100%;
  text-align: center;
  color: #FFF; /* White text */
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  background-color: #1a1a1a; /* Darker background */
  overflow-y: auto; /* Add vertical scrollbar if content exceeds max height */
  padding: 10px; /* Optional padding for better appearance */
}
  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  td {
    width: 110px; /* Example fixed width */
    overflow: hidden; /* Hides content that overflows */
    white-space: nowrap; /* Prevents text from wrapping to the next line */
    text-overflow: ellipsis; /* Truncates with an ellipsis */
    border: 1px solid rgb(59, 81, 121);
    padding: 8px;
    text-align: center;
    color: rgb(0, 0, 0);
    background-color: #ffffff;
  }

  th {
    background-color: #000000;
    color: rgb(255, 255, 255);
  }

  tbody {
    display: block; /* Change display to block */
    max-height: 620px; /* Set a max height */
    overflow-y: auto; /* Add scrollbar if content exceeds max height */
  }

  thead, tbody tr {
    display: table; /* Enable tables to behave like normal */
    width: 100%; /* Set width to match table width */
    table-layout: fixed; /* Ensure layout is fixed */
  }
</style>