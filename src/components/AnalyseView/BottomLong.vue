<template>
  <div>
    <table class="trames">
      <thead>
        <tr>
          <th>MAC Source</th>
          <th>MAC Destination</th>
          <th>Interface</th>
          <th>L3</th>
          <th>IP Source</th>
          <th>IP Destination</th>
          <th>L4</th>
          <th>Port Source</th>
          <th>Port Destination</th>
          <th>L7</th>
          <th>Taille (o)</th>
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
      frames: Array(4).fill({}).map(() => ({
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
      if (this.frames.length > 4) {
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
  position: fixed;
  bottom: 25px; /* Ajustez cette valeur selon la hauteur de votre barre de statut */
  left: 0;
  right: 0;
  height: 120px;
  width: 100%;
  background-color: #000; /* Black background */
  font-family: 'Courier New', Courier, monospace; /* Monospace font for terminal look */
}

table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

td, th {
  padding: 8px;
  text-align: center;
  color: rgb(132, 195, 247); /* Matrix green text */
  background-color: #000; /* Black background */
  overflow: hidden; /* Hides content that overflows */
  white-space: nowrap; /* Prevents text from wrapping to the next line */
  text-overflow: ellipsis; /* Truncates with an ellipsis */
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

