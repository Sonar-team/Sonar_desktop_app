<template>
  <div>
    <table class="trames">
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
          <th>Taille du paquet</th>
          <th>Horodatage</th> <!-- Nouvelle en-tête pour l'horodatage -->

        </tr>
      </thead>
      <tbody>
        <tr v-for="(frame, index) in frames" :key="index">
          <td>{{ frame.mac_address_source }}</td>
          <td>{{ frame.mac_address_destination }}</td>
          <td>{{ frame.interface }}</td>
          <td>{{ frame.l_3_protocol }}</td>
          <td>{{ frame.layer_3_infos.ip_source }}</td>
          <td>{{ frame.layer_3_infos.ip_destination }}</td>
          <td>{{ frame.layer_3_infos.l_4_protocol }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.port_source }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.port_destination }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.l_7_protocol }}</td>
          <td>{{ frame.packet_size }}</td>
          <td>{{ frame.timestamp }}</td> <!-- Nouvelle cellule pour l'horodatage -->

        </tr>
      </tbody>
    </table>
  </div>
</template>

<script>

import { listen } from '@tauri-apps/api/event';

export default {
  data() {
    return {
      frames: Array(5).fill({}).map(() => ({
        mac_address_source: ' ',
        mac_address_destination: ' ',
        interface: ' ',
        l_3_protocol: ' ',
        layer_3_infos: {
          ip_source: '',
          ip_destination: '',
          l_4_protocol: '',
          layer_4_infos: {
            port_source: '',
            port_destination: '',
            l_7_protocol: ''
          }
        },
        packet_size: '',
        timestamp: ''
      })),
      counter: 0
    }
  },

  async mounted() {

    await listen('frame', (packet_info) => {
      this.incrementAndEmit()
      const timestamp = new Date().toLocaleTimeString(); // Obtains the current time
      const frameWithTimestamp = { ...packet_info.payload, timestamp }; // Ajoute l'horodatage au packet

      this.frames.push(frameWithTimestamp);
      // Keep only the last 5 elements
      if (this.frames.length > 5) {
        this.frames.shift();
      }
    });
  },
  methods: {
    incrementAndEmit() {
      // Emit the custom event without specifying a value
      this.$bus.emit('increment-event');

    }
  }
}
</script>

<style scoped>
.trames {
  height: 200px;
  border: 2px solid #3a3a3a; /* Darker border */
  border-radius: 10px;
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.5);
  overflow-y: auto; /* Add vertical scrollbar if content exceeds max height */
}
  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    min-height: 50px;
  }

  th, td {

    width: 136px; /* Example fixed width */
    overflow: hidden; /* Hides content that overflows */
    white-space: nowrap; /* Prevents text from wrapping to the next line */
    text-overflow: ellipsis; /* Truncates with an ellipsis */
    border: 1px solid rgb(59, 81, 121);
    padding: 8px;
    text-align: center;
    color: rgb(255, 255, 255);
    background-color: #000000;
  }

  th {
    background-color: #000000;
  }
</style>
