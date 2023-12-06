<template>
  <div>
    <table class="matrice">
      <thead>
        <tr>
          <th>MAC Source</th>
          <th>MAC Destination</th>
          <th>Interface</th>
          <th>IP Source</th>
          <th>IP Destination</th>
          <th>Protocol</th>
          <th>Port Source</th>
          <th>Port Destination</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(frame, index) in matrices" :key="index">
          <td>{{ frame.mac_address_source }}</td>
          <td>{{ frame.mac_address_destination }}</td>
          <td>{{ frame.interface }}</td>
          <td>{{ frame.layer_3_infos.ip_source }}</td>
          <td>{{ frame.layer_3_infos.ip_destination }}</td>
          <td>{{ frame.layer_3_infos.l_4_protocol }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.port_source }}</td>
          <td>{{ frame.layer_3_infos.layer_4_infos.port_destination }}</td>
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
      matrices: []
    }
  },
  async mounted() {
    console.log('mounted top right matrice')
    await listen('matrice', (packet_info) => {
      //console.log('Received event:', packet_info);      // Push the new counter to the array
      this.matrices.push(packet_info.payload);

      // Keep only the last 5 elements
      if (this.matrices.length > 5) {
        this.matrices.shift();
      }
    });
  }
}
</script>

<style scoped>
  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  th, td {
    width: 120px; /* Example fixed width */
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
</style>

